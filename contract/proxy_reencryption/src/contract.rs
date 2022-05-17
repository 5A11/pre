use crate::msg::{
    ExecuteMsg, ExecuteMsgJSONResponse, GetAvailableProxiesResponse, GetContractStateResponse,
    GetDataIDResponse, GetDelegationStatusResponse, GetFragmentsResponse, GetProxyStatusResponse,
    GetProxyTasksResponse, GetStakingConfigResponse, InstantiateMsg, InstantiateMsgResponse,
    ProxyAvailabilityResponse, ProxyDelegationString, ProxyStakeResponse, ProxyStatusResponse,
    ProxyTaskResponse, QueryMsg,
};
use crate::proxies::{
    get_maximum_withdrawable_stake_amount, store_get_all_active_proxy_pubkeys,
    store_get_all_proxies, store_get_proxy_address, store_get_proxy_entry,
    store_remove_proxy_address, store_remove_proxy_entry, store_set_is_proxy_active,
    store_set_proxy_address, store_set_proxy_entry, Proxy, ProxyState,
};
use crate::state::{
    store_get_data_entry, store_get_delegator_address, store_get_staking_config, store_get_state,
    store_get_timeouts_config, store_set_data_entry, store_set_delegator_address,
    store_set_staking_config, store_set_state, store_set_timeouts_config, DataEntry, StakingConfig,
    State, TimeoutsConfig,
};

use crate::delegations::{
    get_delegation_state, get_n_available_proxies_from_delegation,
    get_n_minimum_proxies_for_refund, remove_proxy_from_delegations,
    store_add_per_proxy_delegation, store_get_all_proxies_from_delegation, store_get_delegation,
    store_get_proxy_delegation_id, store_is_proxy_delegation_empty, store_set_delegation,
    store_set_delegation_id, ProxyDelegation,
};
use crate::reencryption_requests::{
    abandon_all_proxy_tasks, abandon_proxy_task, check_and_resolve_all_timedout_tasks,
    get_all_fragments, get_reencryption_request_state, store_add_delegatee_proxy_task,
    store_add_proxy_task_to_queue, store_get_all_proxy_tasks_in_queue,
    store_get_delegatee_proxy_task, store_get_proxy_task,
    store_is_list_of_delegatee_proxy_tasks_empty, store_remove_proxy_task_from_queue,
    store_set_proxy_task, ProxyTask,
};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Attribute, BankMsg, Binary, Coin, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Storage, SubMsg, Uint128,
};
use std::collections::HashMap;

use crate::common::add_bank_msg;

//use umbral_pre::{Capsule, CapsuleFrag, DeserializableFromArray, PublicKey};

macro_rules! generic_err {
    ($val:expr) => {
        Err(StdError::generic_err($val))
    };
}

pub const DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT: u128 = 1000;
pub const DEFAULT_TASK_REWARD_AMOUNT: u128 = 100;
pub const DEFAULT_PER_TASK_SLASH_STAKE_AMOUNT: u128 = 100;
pub const DEFAULT_TIMEOUT_HEIGHT: u64 = 50;

pub const FRAGMENT_VERIFICATION_ERROR: &str = "Fragment verification failed: ";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        admin: msg.admin.unwrap_or(info.sender),
        threshold: msg.threshold.unwrap_or(1),
        next_proxy_task_id: 0,
        next_delegation_id: 0,
        proxy_whitelisting: msg.proxy_whitelisting.unwrap_or(false),
        terminated: false,
    };

    if state.threshold == 0 {
        return generic_err!("Threshold cannot be 0");
    }

    let staking_config = StakingConfig {
        stake_denom: msg.stake_denom,
        minimum_proxy_stake_amount: msg
            .minimum_proxy_stake_amount
            .unwrap_or_else(|| Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT)),
        per_proxy_task_reward_amount: msg
            .per_proxy_task_reward_amount
            .unwrap_or_else(|| Uint128::new(DEFAULT_TASK_REWARD_AMOUNT)),
        per_task_slash_stake_amount: msg
            .per_task_slash_stake_amount
            .unwrap_or_else(|| Uint128::new(DEFAULT_PER_TASK_SLASH_STAKE_AMOUNT)),
    };
    store_set_staking_config(deps.storage, &staking_config)?;

    let timeouts_config = TimeoutsConfig {
        timeout_height: msg.timeout_height.unwrap_or(DEFAULT_TIMEOUT_HEIGHT),
        next_task_id_to_be_checked: 0,
    };
    store_set_timeouts_config(deps.storage, &timeouts_config)?;

    store_set_state(deps.storage, &state)?;

    let new_proxy = Proxy {
        state: ProxyState::Authorised,
        proxy_pubkey: None,
        stake_amount: Uint128::new(0),
    };

    if let Some(ref proxies_addr) = msg.proxies {
        for proxy_addr in proxies_addr {
            store_set_proxy_entry(deps.storage, proxy_addr, &new_proxy);
        }
    };

    let json_response = InstantiateMsgResponse {
        threshold: state.threshold,
        admin: state.admin,
        proxy_whitelisting: state.proxy_whitelisting,
        proxies: msg.proxies,
        stake_denom: staking_config.stake_denom,
        minimum_proxy_stake_amount: staking_config.minimum_proxy_stake_amount,
        per_proxy_task_reward_amount: staking_config.per_proxy_task_reward_amount,
        per_task_slash_stake_amount: staking_config.per_task_slash_stake_amount,
        timeout_height: timeouts_config.timeout_height,
    };

    let serialized_json_response = match serde_json::to_string(&json_response) {
        Ok(s) => Ok(s),
        Err(_err) => generic_err!("failed to serialize json response"),
    }?;
    let response = Response::new()
        .add_attribute("indexer", "fetchai.pre")
        .add_attribute("json", serialized_json_response);
    Ok(response)
}

// Admin actions

fn try_add_proxy(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proxy_addr: &Addr,
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    if store_get_proxy_entry(deps.storage, proxy_addr).is_some() {
        return generic_err!(format!("{} is already proxy", proxy_addr));
    }

    let new_proxy = Proxy {
        state: ProxyState::Authorised,
        proxy_pubkey: None,
        stake_amount: Uint128::new(0),
    };

    store_set_proxy_entry(deps.storage, proxy_addr, &new_proxy);

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "add_proxy"));
    response
        .attributes
        .push(Attribute::new("admin", info.sender.as_str()));
    response
        .attributes
        .push(Attribute::new("proxy_addr", proxy_addr.as_str()));
    Ok(response)
}

fn try_remove_proxy(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proxy_addr: &Addr,
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage)?;
    let staking_config: StakingConfig = store_get_staking_config(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    // check if proxy_addr is authorised
    let mut proxy = match store_get_proxy_entry(deps.storage, proxy_addr) {
        None => generic_err!("Sender is not a proxy"),
        Some(proxy) => Ok(proxy),
    }?;

    // Registered or Leaving state
    if let Some(proxy_pubkey) = proxy.proxy_pubkey {
        // In leaving state this was already done
        if proxy.state != ProxyState::Leaving {
            store_set_is_proxy_active(deps.storage, &proxy_pubkey, false);
            remove_proxy_from_delegations(deps.storage, &proxy_pubkey)?;
        }

        abandon_all_proxy_tasks(deps.storage, &proxy_pubkey, &mut response)?;
        store_remove_proxy_address(deps.storage, &proxy_pubkey);
    }

    // Update proxy entry to get correct stake amount after possible slashing
    proxy = store_get_proxy_entry(deps.storage, proxy_addr).unwrap();

    // Return remaining stake back to proxy
    add_bank_msg(
        &mut response,
        proxy_addr,
        proxy.stake_amount.u128(),
        &staking_config.stake_denom,
    );

    // Remove proxy entry = remove pubkey
    store_remove_proxy_entry(deps.storage, proxy_addr);

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "store_remove_proxy_entry"));
    response
        .attributes
        .push(Attribute::new("admin", info.sender.as_str()));
    response
        .attributes
        .push(Attribute::new("proxy_addr", proxy_addr.as_str()));
    Ok(response)
}

fn try_terminate_contract(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let mut state: State = store_get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    ensure_not_terminated(&state)?;

    let proxy_pubkeys = store_get_all_active_proxy_pubkeys(deps.storage);

    for proxy_pubkey in proxy_pubkeys {
        let proxy_address = store_get_proxy_address(deps.storage, &proxy_pubkey).unwrap();

        let mut proxy_entry = store_get_proxy_entry(deps.storage, &proxy_address).unwrap();

        store_set_is_proxy_active(deps.storage, &proxy_pubkey, false);
        remove_proxy_from_delegations(deps.storage, &proxy_pubkey)?;

        proxy_entry.state = ProxyState::Leaving;
        store_set_proxy_entry(deps.storage, &proxy_address, &proxy_entry);
    }

    // Update contract state
    state.terminated = true;
    store_set_state(deps.storage, &state)?;

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "terminate_contract"));
    response
        .attributes
        .push(Attribute::new("admin", info.sender.as_str()));
    Ok(response)
}

fn try_withdraw_contract(
    mut response: Response,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient_addr: &Addr,
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    if !state.terminated {
        return generic_err!("Contract not terminated");
    }

    // Withdrawal is possible when contract is terminated and all requests are resolved/timed-out
    let timeouts_config: TimeoutsConfig = store_get_timeouts_config(deps.storage).unwrap();
    if timeouts_config.next_task_id_to_be_checked < state.next_proxy_task_id {
        return generic_err!("There are requests to be resolved, cannot withdraw");
    }

    let mut contract_balance: Vec<Coin> = deps.querier.query_all_balances(env.contract.address)?;

    let staking_config = store_get_staking_config(deps.storage)?;

    // Count all remaining proxies stake amount
    let mut proxies_stake_amount: u128 = 0;

    let proxies = store_get_all_proxies(deps.storage);
    for proxy_addr in proxies {
        let proxy_entry = store_get_proxy_entry(deps.storage, &proxy_addr).unwrap();

        proxies_stake_amount += proxy_entry.stake_amount.u128();
    }

    // Subtract returned stake to proxies
    let mut stake_i: Option<usize> = None;
    for (i, coin) in contract_balance.iter_mut().enumerate() {
        if coin.denom == staking_config.stake_denom {
            coin.amount = coin
                .amount
                .checked_sub(Uint128::new(proxies_stake_amount))?;
            stake_i = Some(i);
            break;
        }
    }

    // Remove Coin if amount after subtracting is 0
    if let Some(i) = stake_i {
        if contract_balance[i].amount.u128() == 0u128 {
            contract_balance.remove(i);
        }
    }

    if contract_balance.is_empty() {
        return generic_err!("Nothing to withdraw");
    }

    // Return remaining stake to recipient
    response.messages.push(SubMsg::new(BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: contract_balance,
    }));

    response
        .attributes
        .push(Attribute::new("action", "withdraw_contract"));
    response
        .attributes
        .push(Attribute::new("recipient_addr", recipient_addr.as_str()));
    response
        .attributes
        .push(Attribute::new("admin", info.sender.as_str()));
    Ok(response)
}

// Proxy actions

fn try_register_proxy(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proxy_pubkey: String,
) -> StdResult<Response> {
    let staking_config = store_get_staking_config(deps.storage)?;
    let state: State = store_get_state(deps.storage)?;

    ensure_not_terminated(&state)?;

    let mut proxy = match store_get_proxy_entry(deps.storage, &info.sender) {
        None => {
            if state.proxy_whitelisting {
                // Whitelisting enabled - proxy is not authorised
                generic_err!("Sender is not a proxy")
            } else {
                // Whitelisting disabled - anyone can register
                let new_proxy = Proxy {
                    state: ProxyState::Authorised,
                    proxy_pubkey: None,
                    stake_amount: Uint128::new(0),
                };

                store_set_proxy_entry(deps.storage, &info.sender, &new_proxy);
                Ok(new_proxy)
            }
        }
        Some(proxy) => Ok(proxy),
    }?;

    // Check if provided pubkey is not used by other proxy
    if let Some(address) = store_get_proxy_address(deps.storage, &proxy_pubkey) {
        if address != info.sender {
            return generic_err!("Pubkey already used by different proxy.");
        }
    }

    let mut funds_amount: u128 = 0;
    match &proxy.proxy_pubkey {
        // reactivation case
        Some(pubkey) => {
            if proxy.state == ProxyState::Registered {
                return generic_err!("Proxy already registered.");
            }

            if pubkey != &proxy_pubkey {
                return generic_err!(
                    "Proxy need to be unregistered to use a different public key."
                );
            }

            if !info.funds.is_empty() {
                funds_amount = ensure_stake(&staking_config, &info.funds, &0u128)?;
            }
        }
        // registration case
        None => {
            proxy.proxy_pubkey = Some(proxy_pubkey.clone());
            funds_amount = ensure_stake(
                &staking_config,
                &info.funds,
                &staking_config.minimum_proxy_stake_amount.u128(),
            )?;
            store_set_proxy_address(deps.storage, &proxy_pubkey, &info.sender);
        }
    }

    proxy.state = ProxyState::Registered;
    proxy.stake_amount = proxy.stake_amount.checked_add(Uint128::new(funds_amount))?;
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);
    store_set_is_proxy_active(deps.storage, &proxy_pubkey, true);

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "register_proxy"));
    response
        .attributes
        .push(Attribute::new("proxy", info.sender.as_str()));
    Ok(response)
}

fn try_unregister_proxy(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let staking_config = store_get_staking_config(deps.storage)?;

    // Check if proxy is authorised
    let mut proxy = match store_get_proxy_entry(deps.storage, &info.sender) {
        None => generic_err!("Sender is not a proxy"),
        Some(proxy) => Ok(proxy),
    }?;

    // Check if proxy is registered
    let proxy_pubkey = match proxy.proxy_pubkey {
        None => generic_err!("Proxy already unregistered"),
        Some(proxy_pubkey) => Ok(proxy_pubkey),
    }?;

    if proxy.state != ProxyState::Leaving {
        store_set_is_proxy_active(deps.storage, &proxy_pubkey, false);
        remove_proxy_from_delegations(deps.storage, &proxy_pubkey)?;
    }

    // This can resolve to proxy being slashed
    abandon_all_proxy_tasks(deps.storage, &proxy_pubkey, &mut response)?;
    store_remove_proxy_address(deps.storage, &proxy_pubkey);

    // Update proxy entry to get correct stake amount after possible slashing
    proxy = store_get_proxy_entry(deps.storage, &info.sender).unwrap();

    // Return remaining stake back to proxy
    add_bank_msg(
        &mut response,
        &info.sender,
        proxy.stake_amount.u128(),
        &staking_config.stake_denom,
    );

    proxy.stake_amount = Uint128::new(0);
    proxy.state = ProxyState::Authorised;
    proxy.proxy_pubkey = None;
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "unregister_proxy"));
    response
        .attributes
        .push(Attribute::new("proxy", info.sender.as_str()));
    response
        .attributes
        .push(Attribute::new("proxy_pubkey", proxy_pubkey.as_str()));
    Ok(response)
}

fn try_deactivate_proxy(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    match store_get_proxy_entry(deps.storage, &info.sender) {
        None => {
            // Unregistered state
            return generic_err!("Sender is not a proxy");
        }
        Some(mut proxy) => {
            if proxy.state == ProxyState::Leaving || proxy.state == ProxyState::Authorised {
                return generic_err!("Proxy already deactivated");
            }

            // Pubkey is missing only when in Authorised/Unregistered state
            let proxy_pubkey = proxy.proxy_pubkey.clone().unwrap();

            store_set_is_proxy_active(deps.storage, &proxy_pubkey, false);
            remove_proxy_from_delegations(deps.storage, &proxy_pubkey)?;

            proxy.state = ProxyState::Leaving;
            store_set_proxy_entry(deps.storage, &info.sender, &proxy);
        }
    }

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "deactivate_proxy"));
    response
        .attributes
        .push(Attribute::new("proxy", info.sender.as_str()));
    Ok(response)
}

fn try_provide_reencrypted_fragment(
    mut response: Response,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data_id: &str,
    delegatee_pubkey: &str,
    fragment: &str,
) -> StdResult<Response> {
    // Get proxy_pubkey or return error
    let mut proxy = match store_get_proxy_entry(deps.storage, &info.sender) {
        None => generic_err!("Proxy not registered"),
        Some(proxy) => Ok(proxy),
    }?;

    let proxy_pubkey = match proxy.proxy_pubkey.clone() {
        None => generic_err!("Proxy not active"),
        Some(proxy_pubkey) => Ok(proxy_pubkey),
    }?;

    // Get task_id or return error
    let task_id: u64 = match store_get_delegatee_proxy_task(
        deps.storage,
        data_id,
        delegatee_pubkey,
        &proxy_pubkey,
    ) {
        None => generic_err!("This fragment was not requested."),
        Some(task_id) => Ok(task_id),
    }?;

    // Task must exist - panic otherwise
    let mut proxy_task = store_get_proxy_task(deps.storage, &task_id).unwrap();

    if env.block.height >= proxy_task.timeout_height {
        return generic_err!("Request timed out.");
    }

    if proxy_task.fragment.is_some() {
        return generic_err!("Fragment already provided.");
    }

    /*
    let data_entry = store_get_data_entry(deps.storage, data_id).unwrap();
    verify_fragment(
        fragment,
        &data_entry.capsule,
        &data_entry.delegator_pubkey,
        &proxy_task.delegatee_pubkey,
    )?;
     */

    if get_all_fragments(deps.storage, data_id, delegatee_pubkey).contains(&fragment.to_string()) {
        return generic_err!("Fragment already provided by other proxy.");
    }

    let staking_config: StakingConfig = store_get_staking_config(deps.storage)?;

    // Add fragment to task

    proxy_task.fragment = Some(fragment.to_string());

    // Add reward to proxy stake + return withdrawn stake
    let return_stake_amount = staking_config.per_proxy_task_reward_amount.u128()
        + staking_config.per_task_slash_stake_amount.u128();
    proxy.stake_amount = Uint128::new(proxy.stake_amount.u128() + return_stake_amount);

    // Update maps
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);
    store_set_proxy_task(deps.storage, &task_id, &proxy_task);

    // Remove task from proxy queue as it's completed
    store_remove_proxy_task_from_queue(deps.storage, &proxy_pubkey, &task_id);

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "provide_reencrypted_fragment"));
    response.attributes.push(Attribute::new("data_id", data_id));
    response
        .attributes
        .push(Attribute::new("delegatee_pubkey", delegatee_pubkey));
    response
        .attributes
        .push(Attribute::new("fragment", fragment));
    response
        .attributes
        .push(Attribute::new("new_stake", proxy.stake_amount));
    Ok(response)
}

fn try_skip_reencryption_task(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &str,
    delegatee_pubkey: &str,
) -> StdResult<Response> {
    let staking_config = store_get_staking_config(deps.storage)?;
    let state = store_get_state(deps.storage)?;

    // Map of funds to be retrieved to delegator
    let mut delegator_retrieve_funds_amount: HashMap<Addr, u128> = HashMap::new();

    // Get proxy pubkey
    let proxy = match store_get_proxy_entry(deps.storage, &info.sender) {
        None => generic_err!("Sender is not a proxy"),
        Some(proxy) => Ok(proxy),
    }?;

    let proxy_pubkey = match proxy.proxy_pubkey {
        None => generic_err!("Proxy not registered"),
        Some(proxy_pubkey) => Ok(proxy_pubkey),
    }?;

    // Get task_id or return error
    let task_id: u64 = match store_get_delegatee_proxy_task(
        deps.storage,
        data_id,
        delegatee_pubkey,
        &proxy_pubkey,
    ) {
        None => generic_err!("Task doesn't exist."),
        Some(task_id) => Ok(task_id),
    }?;

    let proxy_task = store_get_proxy_task(deps.storage, &task_id).unwrap();

    if proxy_task.fragment.is_some() {
        return generic_err!("Task was already completed.");
    }

    // Remove re-encryption task and slash proxy
    abandon_proxy_task(
        deps.storage,
        &task_id,
        &state,
        &staking_config,
        &mut delegator_retrieve_funds_amount,
    )?;

    // Return stake from unfinished task to delegator
    for (delegator_addr, stake_amount) in delegator_retrieve_funds_amount {
        add_bank_msg(
            &mut response,
            &delegator_addr,
            stake_amount,
            &staking_config.stake_denom,
        );
    }

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "skip_reencryption_task"));
    response.attributes.push(Attribute::new("data_id", data_id));
    response
        .attributes
        .push(Attribute::new("delegatee_pubkey", delegatee_pubkey));
    Ok(response)
}

fn try_withdraw_stake(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    stake_amount: &Option<Uint128>,
) -> StdResult<Response> {
    let staking_config = store_get_staking_config(deps.storage)?;

    // Check if proxy is authorised
    let mut proxy = match store_get_proxy_entry(deps.storage, &info.sender) {
        None => generic_err!("Sender is not a proxy"),
        Some(proxy) => Ok(proxy),
    }?;

    // Check maximum amount that can be withdrawn
    let maximum_withdrawable_amount =
        get_maximum_withdrawable_stake_amount(&staking_config, &proxy);

    if maximum_withdrawable_amount == 0 {
        return generic_err!("Not enough stake to withdraw");
    }

    // Withdraw maximum possible in case of stake_amount is None
    let mut withdraw_stake_amount = stake_amount
        .map(|data| data.u128())
        .unwrap_or(maximum_withdrawable_amount);

    // Need to leave at least minimum_proxy_stake_amount after withdrawal
    if withdraw_stake_amount > maximum_withdrawable_amount {
        withdraw_stake_amount = maximum_withdrawable_amount;
    }

    // Update proxy stake amount
    proxy.stake_amount = Uint128::new(proxy.stake_amount.u128() - withdraw_stake_amount);
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);
    add_bank_msg(
        &mut response,
        &info.sender,
        withdraw_stake_amount,
        &staking_config.stake_denom,
    );

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "withdraw_stake"));
    response.attributes.push(Attribute::new(
        "withdrawn_stake",
        Uint128::new(withdraw_stake_amount),
    ));
    response
        .attributes
        .push(Attribute::new("new_stake", proxy.stake_amount));
    Ok(response)
}

fn try_add_stake(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    // Check if proxy is authorised
    let mut proxy = match store_get_proxy_entry(deps.storage, &info.sender) {
        None => generic_err!("Sender is not a proxy"),
        Some(proxy) => Ok(proxy),
    }?;

    // Ensure correct denom
    let staking_config = store_get_staking_config(deps.storage)?;
    ensure_stake(&staking_config, &info.funds, &1)?;

    // Update proxy stake amount
    proxy.stake_amount = Uint128::new(proxy.stake_amount.u128() + info.funds[0].amount.u128());
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "add_stake"));
    response
        .attributes
        .push(Attribute::new("added_stake", info.funds[0].amount));
    response
        .attributes
        .push(Attribute::new("new_stake", proxy.stake_amount));
    Ok(response)
}

// Delegator actions

fn try_add_data(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &str,
    delegator_pubkey: &str,
    capsule: &str,
) -> StdResult<Response> {
    if store_get_data_entry(deps.storage, data_id).is_some() {
        return generic_err!(format!("Entry with ID {} already exist.", data_id));
    }

    ensure_delegator(deps.storage, delegator_pubkey, &info.sender)?;

    let entry = DataEntry {
        delegator_pubkey: delegator_pubkey.to_string(),
        capsule: capsule.to_string(),
    };
    store_set_data_entry(deps.storage, data_id, &entry);

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "add_data"));
    response
        .attributes
        .push(Attribute::new("owner", info.sender.as_str()));
    response.attributes.push(Attribute::new("data_id", data_id));
    response
        .attributes
        .push(Attribute::new("delegator_pubkey", delegator_pubkey));

    Ok(response)
}

fn try_add_delegation(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_delegations: &[ProxyDelegationString],
) -> StdResult<Response> {
    ensure_delegator(deps.storage, delegator_pubkey, &info.sender)?;

    let mut state: State = store_get_state(deps.storage)?;
    let staking_config: StakingConfig = store_get_staking_config(deps.storage)?;

    ensure_not_terminated(&state)?;

    if !store_is_proxy_delegation_empty(deps.storage, delegator_pubkey, delegatee_pubkey) {
        return generic_err!("Delegation already exists.");
    }

    let n_minimum_proxies = get_n_minimum_proxies_for_refund(&state, &staking_config);

    if proxy_delegations.len() < n_minimum_proxies as usize {
        return generic_err!(format!("Required at least {} proxies.", n_minimum_proxies));
    }

    for proxy_delegation in proxy_delegations {
        if store_get_proxy_address(deps.storage, &proxy_delegation.proxy_pubkey).is_none() {
            return generic_err!(format!(
                "Unknown proxy with pubkey {}",
                &proxy_delegation.proxy_pubkey
            ));
        }

        if store_get_proxy_delegation_id(
            deps.storage,
            delegator_pubkey,
            delegatee_pubkey,
            &proxy_delegation.proxy_pubkey,
        )
        .is_some()
        {
            return generic_err!(format!(
                "Delegation string was already provided for proxy {}.",
                &proxy_delegation.proxy_pubkey
            ));
        }

        let delegation = ProxyDelegation {
            delegator_pubkey: delegator_pubkey.to_string(),
            delegatee_pubkey: delegatee_pubkey.to_string(),
            delegation_string: proxy_delegation.delegation_string.clone(),
        };

        store_set_delegation(deps.storage, &state.next_delegation_id, &delegation);
        store_set_delegation_id(
            deps.storage,
            delegator_pubkey,
            delegatee_pubkey,
            &proxy_delegation.proxy_pubkey,
            &state.next_delegation_id,
        );
        store_add_per_proxy_delegation(
            deps.storage,
            &proxy_delegation.proxy_pubkey,
            &state.next_delegation_id,
        );

        state.next_delegation_id += 1;
    }

    store_set_state(deps.storage, &state)?;

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "add_delegation"));
    response
        .attributes
        .push(Attribute::new("delegator_address", info.sender.as_str()));
    response
        .attributes
        .push(Attribute::new("delegator_pubkey", delegator_pubkey));
    response
        .attributes
        .push(Attribute::new("delegatee_pubkey", delegatee_pubkey));

    Ok(response)
}

fn try_request_reencryption(
    mut response: Response,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data_id: &str,
    delegatee_pubkey: &str,
) -> StdResult<Response> {
    // Load config
    let staking_config: StakingConfig = store_get_staking_config(deps.storage)?;
    let mut state: State = store_get_state(deps.storage)?;
    let timeouts_config: TimeoutsConfig = store_get_timeouts_config(deps.storage)?;

    ensure_not_terminated(&state)?;

    // Only data owner can request reencryption
    let data_entry: DataEntry = match store_get_data_entry(deps.storage, data_id) {
        None => generic_err!("Data entry doesn't exist."),
        Some(data_entry) => Ok(data_entry),
    }?;

    ensure_delegator(deps.storage, &data_entry.delegator_pubkey, &info.sender)?;

    // Get selected proxies for current delegation
    let proxy_pubkeys = store_get_all_proxies_from_delegation(
        deps.storage,
        &data_entry.delegator_pubkey,
        delegatee_pubkey,
    );

    if proxy_pubkeys.is_empty() {
        return generic_err!("ProxyDelegation doesn't exist.");
    }

    // Get number of proxies with enough stake
    let n_available_proxies = get_n_available_proxies_from_delegation(
        deps.storage,
        &data_entry.delegator_pubkey,
        delegatee_pubkey,
        &staking_config.per_task_slash_stake_amount.u128(),
    );

    let n_minimum_proxies = get_n_minimum_proxies_for_refund(&state, &staking_config);

    // Not enough request can be created
    if n_available_proxies < n_minimum_proxies {
        return generic_err!(format!(
            "Proxies are too busy, try again later. Available {} proxies out of {}, minimum is {}",
            n_available_proxies,
            proxy_pubkeys.len(),
            n_minimum_proxies
        ));
    }

    if !store_is_list_of_delegatee_proxy_tasks_empty(deps.storage, data_id, delegatee_pubkey) {
        return generic_err!("Reencryption already requested");
    }

    // Ensure more than per_proxy_task_reward_amount * number_of_proxies of stake provided
    let total_required_reward_amount =
        staking_config.per_proxy_task_reward_amount.u128() * n_available_proxies as u128;
    ensure_stake(&staking_config, &info.funds, &total_required_reward_amount)?;

    // Prepare template for each proxy task
    let mut new_proxy_task = ProxyTask {
        delegatee_pubkey: delegatee_pubkey.to_string(),
        data_id: data_id.to_string(),
        fragment: None,
        proxy_pubkey: "".to_string(),
        delegation_string: "".to_string(),
        resolved: false,
        abandoned: false,
        timeout_height: env.block.height + timeouts_config.timeout_height,
        delegator_addr: info.sender.clone(), // Per proxy stake amount
    };

    let mut proxy_stake = Vec::new();

    // Assign re-encrpytion tasks to all available proxies
    for proxy_pubkey in &proxy_pubkeys {
        // Check if proxy has enough stake
        let proxy_addr = store_get_proxy_address(deps.storage, proxy_pubkey).unwrap();
        let mut proxy = store_get_proxy_entry(deps.storage, &proxy_addr).unwrap();

        if proxy.stake_amount.u128() < staking_config.per_task_slash_stake_amount.u128() {
            // Proxy cannot be selected for insufficient amount
            continue;
        }

        // Subtract stake from proxy
        proxy.stake_amount = proxy
            .stake_amount
            .checked_sub(staking_config.per_task_slash_stake_amount)?;
        store_set_proxy_entry(deps.storage, &proxy_addr, &proxy);

        // Get delegation
        let delegation_id = store_get_proxy_delegation_id(
            deps.storage,
            &data_entry.delegator_pubkey,
            delegatee_pubkey,
            proxy_pubkey,
        )
        .unwrap();
        let delegation = store_get_delegation(deps.storage, &delegation_id).unwrap();

        // Add reencryption task for each proxy
        new_proxy_task.proxy_pubkey = proxy_pubkey.clone();
        new_proxy_task.delegation_string = delegation.delegation_string;
        let task_id = state.next_proxy_task_id;
        store_set_proxy_task(deps.storage, &task_id, &new_proxy_task);
        store_add_delegatee_proxy_task(
            deps.storage,
            data_id,
            delegatee_pubkey,
            proxy_pubkey,
            &task_id,
        );
        store_add_proxy_task_to_queue(deps.storage, proxy_pubkey, &task_id);
        state.next_proxy_task_id += 1;

        proxy_stake.push(ProxyStakeResponse {
            proxy_addr: proxy_addr.clone(),
            stake: proxy.stake_amount,
        });
    }

    store_set_state(deps.storage, &state)?;

    // Return back part of funds if more funds than necessary was provided
    if info.funds[0].amount.u128() > total_required_reward_amount {
        add_bank_msg(
            &mut response,
            &info.sender,
            info.funds[0].amount.u128() - total_required_reward_amount,
            &staking_config.stake_denom,
        );
    }

    let json_response = ExecuteMsgJSONResponse::RequestReencryption {
        proxies: proxy_stake,
    };
    let serialized_json_response = match serde_json::to_string(&json_response) {
        Ok(s) => Ok(s),
        Err(_err) => generic_err!("failed to serialize json response"),
    }?;

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "request_reencryption"));
    response.attributes.push(Attribute::new("data_id", data_id));
    response
        .attributes
        .push(Attribute::new("delegatee_pubkey", delegatee_pubkey));
    response
        .attributes
        .push(Attribute::new("json", serialized_json_response));

    Ok(response)
}

pub fn get_proxy_tasks(
    store: &dyn Storage,
    proxy_pubkey: &str,
    block_height: &u64,
) -> StdResult<Vec<ProxyTaskResponse>> {
    // Returns first available proxy task from queue

    let mut tasks_response: Vec<ProxyTaskResponse> = Vec::new();

    let tasks = store_get_all_proxy_tasks_in_queue(store, proxy_pubkey);

    // No tasks
    if tasks.is_empty() {
        return Ok(tasks_response);
    }

    // Skip all TimedOut tasks
    for task_id in tasks {
        let proxy_task: ProxyTask = store_get_proxy_task(store, &task_id).unwrap();
        if block_height < &proxy_task.timeout_height {
            let data_entry = store_get_data_entry(store, &proxy_task.data_id).unwrap();

            let proxy_task = ProxyTaskResponse {
                data_id: proxy_task.data_id.clone(),
                capsule: data_entry.capsule.clone(),
                delegatee_pubkey: proxy_task.delegatee_pubkey,
                delegator_pubkey: data_entry.delegator_pubkey,
                delegation_string: proxy_task.delegation_string,
            };
            tasks_response.push(proxy_task);
        }
    }
    Ok(tasks_response)
}

pub fn get_proxies_availability(store: &dyn Storage) -> Vec<ProxyAvailabilityResponse> {
    let proxy_pubkeys = store_get_all_active_proxy_pubkeys(store);

    let mut res: Vec<ProxyAvailabilityResponse> = Vec::new();

    for proxy_pubkey in proxy_pubkeys {
        let proxy_addr = store_get_proxy_address(store, &proxy_pubkey).unwrap();
        let proxy_entry: Proxy = store_get_proxy_entry(store, &proxy_addr).unwrap();

        res.push(ProxyAvailabilityResponse {
            proxy_pubkey,
            stake_amount: proxy_entry.stake_amount,
        });
    }

    res
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    let mut response: Response = Response::new();

    // Actions to be performed before every ExecuteMSG call

    // Resolve all timed-out re-encryption requests
    check_and_resolve_all_timedout_tasks(deps.storage, &mut response, env.block.height);

    match msg {
        // Admin actions
        ExecuteMsg::AddProxy { proxy_addr } => {
            try_add_proxy(response, deps, env, info, &proxy_addr)
        }
        ExecuteMsg::RemoveProxy { proxy_addr } => {
            try_remove_proxy(response, deps, env, info, &proxy_addr)
        }
        ExecuteMsg::TerminateContract {} => try_terminate_contract(response, deps, env, info),
        ExecuteMsg::WithdrawContract { recipient_addr } => {
            try_withdraw_contract(response, deps, env, info, &recipient_addr)
        }

        // Proxy actions
        ExecuteMsg::RegisterProxy { proxy_pubkey } => {
            try_register_proxy(response, deps, env, info, proxy_pubkey)
        }
        ExecuteMsg::ProvideReencryptedFragment {
            data_id,
            delegatee_pubkey,
            fragment,
        } => try_provide_reencrypted_fragment(
            response,
            deps,
            env,
            info,
            &data_id,
            &delegatee_pubkey,
            &fragment,
        ),
        ExecuteMsg::SkipReencryptionTask {
            data_id,
            delegatee_pubkey,
        } => try_skip_reencryption_task(response, deps, env, info, &data_id, &delegatee_pubkey),
        ExecuteMsg::UnregisterProxy {} => try_unregister_proxy(response, deps, env, info),
        ExecuteMsg::DeactivateProxy {} => try_deactivate_proxy(response, deps, env, info),
        ExecuteMsg::WithdrawStake { stake_amount } => {
            try_withdraw_stake(response, deps, env, info, &stake_amount)
        }
        ExecuteMsg::AddStake {} => try_add_stake(response, deps, env, info),

        // Delegator actions
        ExecuteMsg::AddData {
            data_id,
            delegator_pubkey,
            capsule,
            ..
        } => try_add_data(
            response,
            deps,
            env,
            info,
            &data_id,
            &delegator_pubkey,
            &capsule,
        ),
        ExecuteMsg::AddDelegation {
            delegator_pubkey,
            delegatee_pubkey,
            proxy_delegations,
        } => try_add_delegation(
            response,
            deps,
            env,
            info,
            &delegator_pubkey,
            &delegatee_pubkey,
            &proxy_delegations,
        ),
        ExecuteMsg::RequestReencryption {
            data_id,
            delegatee_pubkey,
        } => try_request_reencryption(response, deps, env, info, &data_id, &delegatee_pubkey),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAvailableProxies {} => Ok(to_binary(&GetAvailableProxiesResponse {
            proxies: get_proxies_availability(deps.storage),
        })?),
        QueryMsg::GetDataID { data_id } => Ok(to_binary(&GetDataIDResponse {
            data_entry: store_get_data_entry(deps.storage, &data_id),
        })?),
        QueryMsg::GetFragments {
            data_id,
            delegatee_pubkey,
        } => {
            let state = store_get_state(deps.storage)?;
            let data_entry = match store_get_data_entry(deps.storage, &data_id) {
                None => generic_err!("Data entry doesn't exist"),
                Some(data) => Ok(data),
            }?;

            Ok(to_binary(&GetFragmentsResponse {
                reencryption_request_state: get_reencryption_request_state(
                    deps.storage,
                    &state,
                    &data_id,
                    &delegatee_pubkey,
                    &env.block.height,
                ),
                capsule: data_entry.capsule,
                fragments: get_all_fragments(deps.storage, &data_id, &delegatee_pubkey),
                threshold: state.threshold,
            })?)
        }
        QueryMsg::GetContractState {} => {
            let state = store_get_state(deps.storage)?;

            Ok(to_binary(&GetContractStateResponse {
                admin: state.admin,
                threshold: state.threshold,
                terminated: state.terminated,
            })?)
        }

        QueryMsg::GetStakingConfig {} => {
            let staking_config = store_get_staking_config(deps.storage)?;

            Ok(to_binary(&GetStakingConfigResponse {
                stake_denom: staking_config.stake_denom,
                minimum_proxy_stake_amount: staking_config.minimum_proxy_stake_amount,
                per_proxy_task_reward_amount: staking_config.per_proxy_task_reward_amount,
            })?)
        }

        QueryMsg::GetProxyTasks { proxy_pubkey } => Ok(to_binary(&GetProxyTasksResponse {
            proxy_tasks: get_proxy_tasks(deps.storage, &proxy_pubkey, &env.block.height)?,
        })?),

        QueryMsg::GetDelegationStatus {
            delegator_pubkey,
            delegatee_pubkey,
        } => {
            let staking_config = store_get_staking_config(deps.storage)?;
            let n_availbale_proxies = get_n_available_proxies_from_delegation(
                deps.storage,
                &delegator_pubkey,
                &delegatee_pubkey,
                &staking_config.per_task_slash_stake_amount.u128(),
            );

            let minimum_stake_amount =
                n_availbale_proxies as u128 * staking_config.per_proxy_task_reward_amount.u128();

            Ok(to_binary(&GetDelegationStatusResponse {
                delegation_state: get_delegation_state(
                    deps.storage,
                    &delegator_pubkey,
                    &delegatee_pubkey,
                ),
                total_request_reward_amount: Coin {
                    denom: staking_config.stake_denom,
                    amount: Uint128::new(minimum_stake_amount),
                },
            })?)
        }

        QueryMsg::GetProxyStatus { proxy_pubkey } => {
            let mut proxy_status: Option<ProxyStatusResponse> = None;

            if let Some(proxy_addr) = store_get_proxy_address(deps.storage, &proxy_pubkey) {
                let proxy = store_get_proxy_entry(deps.storage, &proxy_addr).unwrap();
                let staking_config = store_get_staking_config(deps.storage)?;

                proxy_status = Some(ProxyStatusResponse {
                    proxy_address: proxy_addr,
                    stake_amount: proxy.stake_amount,
                    withdrawable_stake_amount: Uint128::new(get_maximum_withdrawable_stake_amount(
                        &staking_config,
                        &proxy,
                    )),
                    proxy_state: proxy.state,
                })
            }

            Ok(to_binary(&GetProxyStatusResponse { proxy_status })?)
        }
    }
}

// Private functions

fn ensure_admin(state: &State, addr: &Addr) -> StdResult<()> {
    if addr != &state.admin {
        return generic_err!("Only admin can execute this method.");
    }
    Ok(())
}

fn ensure_delegator(
    storage: &mut dyn Storage,
    delegator_pubkey: &str,
    delegator_address: &Addr,
) -> StdResult<()> {
    if let Some(correct_delegator_addr) = store_get_delegator_address(storage, delegator_pubkey) {
        // Check if delegator_pubkey is registered with delegator_address

        if &correct_delegator_addr != delegator_address {
            return generic_err!(format!(
                "Delegator {} already registered with this pubkey.",
                correct_delegator_addr
            ));
        }
    } else {
        // Reserve delegator_pubkey for current delegator_address
        store_set_delegator_address(storage, delegator_pubkey, delegator_address);
    }
    Ok(())
}

fn ensure_not_terminated(state: &State) -> StdResult<()> {
    if state.terminated {
        return generic_err!("Contract was terminated.");
    }

    Ok(())
}

fn ensure_stake(
    staking_config: &StakingConfig,
    funds: &[Coin],
    required_stake: &u128,
) -> StdResult<u128> {
    if funds.len() != 1 || funds[0].denom != staking_config.stake_denom {
        return generic_err!(format!(
            "Expected 1 Coin with denom {}",
            staking_config.stake_denom
        ));
    }

    if &funds[0].amount.u128() < required_stake {
        return generic_err!(format!(
            "Requires at least {} {}.",
            required_stake, staking_config.stake_denom
        ));
    }
    Ok(funds[0].amount.u128())
}

/*
fn unwrap_or_pass_error<ResultType, ErrType: std::fmt::Display>(
    obj: Result<ResultType, ErrType>,
    error_str: &str,
) -> StdResult<ResultType> {
    // Convert any error to StdError to pass it outside of contract
    match obj {
        Ok(data) => Ok(data),
        Err(error) => generic_err!(format!("{}{}", error_str, error)),
    }
}

pub fn verify_fragment(
    fragment: &str,
    capsule: &str,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
) -> StdResult<()> {
    let fragment_vec =
        unwrap_or_pass_error(base64::decode(&fragment), FRAGMENT_VERIFICATION_ERROR)?;
    let fragment = unwrap_or_pass_error(
        CapsuleFrag::from_bytes(&fragment_vec),
        FRAGMENT_VERIFICATION_ERROR,
    )?;

    let capsule_vec = unwrap_or_pass_error(base64::decode(&capsule), FRAGMENT_VERIFICATION_ERROR)?;
    let capsule = unwrap_or_pass_error(
        Capsule::from_bytes(&capsule_vec),
        FRAGMENT_VERIFICATION_ERROR,
    )?;

    let delegator_pubkey_vec = unwrap_or_pass_error(
        base64::decode(&delegator_pubkey),
        FRAGMENT_VERIFICATION_ERROR,
    )?;
    let delegator_pubkey = unwrap_or_pass_error(
        PublicKey::from_bytes(&delegator_pubkey_vec),
        FRAGMENT_VERIFICATION_ERROR,
    )?;

    let delegatee_pubkey_vec = unwrap_or_pass_error(
        base64::decode(&delegatee_pubkey),
        FRAGMENT_VERIFICATION_ERROR,
    )?;
    let delegatee_pubkey = unwrap_or_pass_error(
        PublicKey::from_bytes(&delegatee_pubkey_vec),
        FRAGMENT_VERIFICATION_ERROR,
    )?;

    match fragment.verify(
        &capsule,
        &delegator_pubkey,
        &delegator_pubkey,
        &delegatee_pubkey,
    ) {
        Ok(_) => Ok(()),
        Err(error) => generic_err!(format!("{}{}", FRAGMENT_VERIFICATION_ERROR, error)),
    }
}
*/
