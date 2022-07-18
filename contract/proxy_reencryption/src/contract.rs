use crate::msg::{
    ExecuteMsg, ExecuteMsgJSONResponse, GetAvailableProxiesResponse, GetContractStateResponse,
    GetDataIDResponse, GetDataLabelsResponse, GetDelegateeLabelsResponse,
    GetDelegationStatusResponse, GetFragmentsResponse, GetProxyStatusResponse,
    GetProxyTasksResponse, GetStakingConfigResponse, InstantiateMsg, InstantiateMsgResponse,
    ProxyAvailabilityResponse, ProxyDelegationString, ProxyStakeResponse, ProxyStatusResponse,
    ProxyTaskResponse, QueryMsg,
};
use crate::proxies::{
    get_maximum_withdrawable_stake_amount, store_get_all_active_proxy_addresses,
    store_get_proxy_entry, store_remove_proxy_entry, store_set_is_proxy_active,
    store_set_proxy_entry, Proxy, ProxyState,
};
use crate::state::{
    store_get_data_entry, store_get_delegator_address, store_get_staking_config, store_get_state,
    store_get_timeouts_config, store_remove_data_entry, store_set_data_entry,
    store_set_delegator_address, store_set_staking_config, store_set_state,
    store_set_timeouts_config, DataEntry, StakingConfig, State, TimeoutsConfig,
};

use crate::delegations::{
    get_delegation_state, get_n_available_proxies_from_delegation,
    get_n_minimum_proxies_for_refund, remove_proxy_from_delegations,
    store_add_per_proxy_delegation, store_get_all_proxies_from_delegation, store_get_delegation,
    store_get_proxy_delegation_id, store_is_proxy_delegation_empty, store_set_delegation,
    store_set_delegation_id, ProxyDelegation,
};
use crate::reencryption_requests::{
    abandon_all_proxy_tasks, abandon_proxy_task, get_all_fragments, get_reencryption_request_state,
    store_add_data_id_task, store_add_delegatee_proxy_task, store_add_proxy_task_to_queue,
    store_get_all_delegatee_proxy_tasks, store_get_all_proxy_tasks_in_queue,
    store_get_data_id_tasks, store_get_delegatee_proxy_task, store_get_proxy_task,
    store_is_list_of_delegatee_proxy_tasks_empty, store_remove_data_id_task,
    store_remove_delegatee_proxy_task, store_remove_proxy_task, store_remove_proxy_task_from_queue,
    store_set_proxy_task, timeout_proxy_task, ProxyTask, ReencryptionRequestState,
};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Attribute, BankMsg, Binary, Coin, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Storage, SubMsg, Uint128,
};
use std::collections::HashMap;

use crate::common::add_bank_msg;
use crate::reencryption_permissions::{
    get_permission, store_add_data_labels, store_add_delegatee_labels, store_get_all_data_labels,
    store_get_all_delegatee_labels, store_remove_data_labels, store_remove_delegatee_labels,
};

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
pub const DEFAULT_WITHDRAWAL_PERIOD: u64 = 500;

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
        withdrawn: false,
        terminate_height: 0,
        withdrawal_period: msg.withdrawal_period.unwrap_or(DEFAULT_WITHDRAWAL_PERIOD),
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
    ensure_not_terminated(&state)?;

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
    ensure_not_terminated(&state)?;

    // check if proxy_addr is authorised
    let mut proxy = match store_get_proxy_entry(deps.storage, proxy_addr) {
        None => generic_err!("Sender is not a proxy"),
        Some(proxy) => Ok(proxy),
    }?;

    // Registered or Leaving state
    if proxy.proxy_pubkey.is_some() {
        // In leaving state this was already done
        if proxy.state != ProxyState::Leaving {
            store_set_is_proxy_active(deps.storage, proxy_addr, false);
            remove_proxy_from_delegations(deps.storage, proxy_addr)?;
        }

        abandon_all_proxy_tasks(deps.storage, proxy_addr, &mut response)?;
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
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let mut state: State = store_get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;
    ensure_not_terminated(&state)?;

    // Update contract state
    state.terminated = true;
    state.terminate_height = env.block.height;
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
    let mut state: State = store_get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    if !state.terminated {
        return generic_err!("Contract not terminated");
    }

    if env.block.height < state.terminate_height + state.withdrawal_period {
        return generic_err!(format!(
            "Withdrawal will be possible at height {}",
            state.terminate_height + state.withdrawal_period
        ));
    }

    let contract_balance: Vec<Coin> = deps.querier.query_all_balances(env.contract.address)?;

    if contract_balance.is_empty() {
        return generic_err!("Nothing to withdraw");
    }

    // Return remaining stake to recipient
    response.messages.push(SubMsg::new(BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: contract_balance,
    }));

    state.withdrawn = true;
    store_set_state(deps.storage, &state)?;

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
            proxy.proxy_pubkey = Some(proxy_pubkey);
            funds_amount = ensure_stake(
                &staking_config,
                &info.funds,
                &staking_config.minimum_proxy_stake_amount.u128(),
            )?;
        }
    }

    proxy.state = ProxyState::Registered;
    proxy.stake_amount = proxy.stake_amount.checked_add(Uint128::new(funds_amount))?;
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);
    store_set_is_proxy_active(deps.storage, &info.sender, true);

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
    let state = store_get_state(deps.storage)?;

    ensure_not_withdrawn(&state)?;

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
        store_set_is_proxy_active(deps.storage, &info.sender, false);
        remove_proxy_from_delegations(deps.storage, &info.sender)?;
    }

    // This can resolve to proxy being slashed
    abandon_all_proxy_tasks(deps.storage, &info.sender, &mut response)?;

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
    let state: State = store_get_state(deps.storage)?;

    ensure_not_withdrawn(&state)?;

    match store_get_proxy_entry(deps.storage, &info.sender) {
        None => {
            // Unregistered state
            return generic_err!("Sender is not a proxy");
        }
        Some(mut proxy) => {
            if proxy.state == ProxyState::Leaving || proxy.state == ProxyState::Authorised {
                return generic_err!("Proxy already deactivated");
            }

            store_set_is_proxy_active(deps.storage, &info.sender, false);
            remove_proxy_from_delegations(deps.storage, &info.sender)?;

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
    let state: State = store_get_state(deps.storage)?;

    ensure_not_withdrawn(&state)?;

    // Get proxy_pubkey or return error
    let mut proxy = match store_get_proxy_entry(deps.storage, &info.sender) {
        None => generic_err!("Proxy not registered"),
        Some(proxy) => Ok(proxy),
    }?;

    if proxy.proxy_pubkey.is_none() {
        return generic_err!("Proxy not active");
    }

    // Get task_id or return error
    let task_id: u64 =
        match store_get_delegatee_proxy_task(deps.storage, data_id, delegatee_pubkey, &info.sender)
        {
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
    store_remove_proxy_task_from_queue(deps.storage, &info.sender, &task_id);

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
    env: Env,
    info: MessageInfo,
    data_id: &str,
    delegatee_pubkey: &str,
) -> StdResult<Response> {
    let staking_config = store_get_staking_config(deps.storage)?;
    let state = store_get_state(deps.storage)?;

    ensure_not_withdrawn(&state)?;

    // Map of funds to be retrieved to delegator
    let mut delegator_retrieve_funds_amount: HashMap<Addr, u128> = HashMap::new();

    // Get proxy pubkey
    let proxy = match store_get_proxy_entry(deps.storage, &info.sender) {
        None => generic_err!("Sender is not a proxy"),
        Some(proxy) => Ok(proxy),
    }?;

    if proxy.proxy_pubkey.is_none() {
        return generic_err!("Proxy not registered");
    }

    // Get task_id or return error
    let task_id: u64 =
        match store_get_delegatee_proxy_task(deps.storage, data_id, delegatee_pubkey, &info.sender)
        {
            None => generic_err!("Task doesn't exist."),
            Some(task_id) => Ok(task_id),
        }?;

    let proxy_task = store_get_proxy_task(deps.storage, &task_id).unwrap();

    if proxy_task.fragment.is_some() {
        return generic_err!("Task was already completed.");
    }

    if env.block.height >= proxy_task.timeout_height {
        return generic_err!("Task timed out.");
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

pub fn try_resolve_timed_out_request(
    mut response: Response,
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    data_id: &str,
    delegatee_pubkey: &str,
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage).unwrap();

    ensure_not_withdrawn(&state)?;

    if get_reencryption_request_state(
        deps.storage,
        &state,
        data_id,
        delegatee_pubkey,
        &env.block.height,
    ) != ReencryptionRequestState::TimedOut
    {
        return generic_err!("Task is not timed-out.");
    }

    let staking_config: StakingConfig = store_get_staking_config(deps.storage).unwrap();
    let task_ids = store_get_all_delegatee_proxy_tasks(deps.storage, data_id, delegatee_pubkey);

    let mut delegator_retrieve_funds_amount: HashMap<Addr, u128> = HashMap::new();
    for i in task_ids {
        match store_get_proxy_task(deps.storage, &i) {
            // Skip if task was deleted
            None => {}
            Some(proxy_task) => {
                // We can move pointer when task is already resolved in case of abandoned request
                if proxy_task.resolved {
                    continue;
                }

                // Resolve timed-out task
                timeout_proxy_task(
                    deps.storage,
                    &i,
                    &staking_config,
                    &mut delegator_retrieve_funds_amount,
                )
                .unwrap();
            }
        }
    }

    if delegator_retrieve_funds_amount.len() > 1 {
        return generic_err!("One request can't have multiple delegators.");
    }

    // Return stake from unfinished tasks to delegators
    let staking_config = store_get_staking_config(deps.storage).unwrap();
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
        .push(Attribute::new("action", "resolve_timed_out_request"));
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
    let state: State = store_get_state(deps.storage)?;

    ensure_not_withdrawn(&state)?;

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
    let state: State = store_get_state(deps.storage)?;

    ensure_not_terminated(&state)?;

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
    data_labels: &Option<Vec<String>>,
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage)?;

    ensure_not_terminated(&state)?;

    if store_get_data_entry(deps.storage, data_id).is_some() {
        return generic_err!(format!("Entry with ID {} already exist.", data_id));
    }

    ensure_delegator(deps.storage, delegator_pubkey, &info.sender)?;

    let entry = DataEntry {
        delegator_pubkey: delegator_pubkey.to_string(),
        capsule: capsule.to_string(),
    };
    store_set_data_entry(deps.storage, data_id, &entry);

    // Add data labels
    if let Some(data_labels) = data_labels {
        store_add_data_labels(deps.storage, data_id, data_labels);
        response
            .attributes
            .push(Attribute::new("data_labels", data_labels.join(", ")));
    }

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

fn try_remove_data(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &str,
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage)?;

    ensure_not_terminated(&state)?;

    // Only data owner can remove data
    let data_entry: DataEntry = match store_get_data_entry(deps.storage, data_id) {
        None => generic_err!(format!("Entry with ID {} does not exist.", data_id)),
        Some(data_entry) => Ok(data_entry),
    }?;

    ensure_delegator(deps.storage, &data_entry.delegator_pubkey, &info.sender)?;

    let staking_config = store_get_staking_config(deps.storage)?;

    let tasks = store_get_data_id_tasks(deps.storage, data_id);

    let mut refund: u128 = 0;
    let mut proxy_stake = Vec::new();

    for task_id in tasks.iter() {
        let proxy_task = store_get_proxy_task(deps.storage, task_id).unwrap();

        // Remove task from proxy queue
        store_remove_proxy_task_from_queue(deps.storage, &proxy_task.proxy_addr, task_id);

        // Remove delegatee proxy task
        store_remove_delegatee_proxy_task(
            deps.storage,
            data_id,
            &proxy_task.delegatee_pubkey,
            &proxy_task.proxy_addr,
        );

        // Remove proxy task
        store_remove_proxy_task(deps.storage, task_id);
        store_remove_data_id_task(deps.storage, data_id, task_id);

        if proxy_task.fragment.is_none() {
            refund += staking_config.per_proxy_task_reward_amount.u128();

            let mut proxy = store_get_proxy_entry(deps.storage, &proxy_task.proxy_addr).unwrap();

            // Give back stake to proxy
            proxy.stake_amount = proxy
                .stake_amount
                .checked_add(staking_config.per_task_slash_stake_amount)?;
            store_set_proxy_entry(deps.storage, &proxy_task.proxy_addr, &proxy);

            proxy_stake.push(ProxyStakeResponse {
                proxy_addr: proxy_task.proxy_addr.clone(),
                stake: proxy.stake_amount,
            });
        }
    }

    // Return stake from unfinished tasks to delegator
    if refund > 0 {
        add_bank_msg(
            &mut response,
            &info.sender,
            refund,
            &staking_config.stake_denom,
        );
    }

    store_remove_data_entry(deps.storage, data_id);

    let json_response = ExecuteMsgJSONResponse::RemoveData {
        proxies: proxy_stake,
    };
    let serialized_json_response = match serde_json::to_string(&json_response) {
        Ok(s) => Ok(s),
        Err(_err) => generic_err!("failed to serialize json response"),
    }?;

    // Return response
    response
        .attributes
        .push(Attribute::new("json", serialized_json_response));

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
    delegatee_labels: &Option<Vec<String>>,
) -> StdResult<Response> {
    ensure_delegator(deps.storage, delegator_pubkey, &info.sender)?;

    let mut state: State = store_get_state(deps.storage)?;
    ensure_not_terminated(&state)?;

    let staking_config: StakingConfig = store_get_staking_config(deps.storage)?;

    if !store_is_proxy_delegation_empty(deps.storage, delegator_pubkey, delegatee_pubkey) {
        return generic_err!("Delegation already exists.");
    }

    let n_minimum_proxies = get_n_minimum_proxies_for_refund(&state, &staking_config);

    if proxy_delegations.len() < n_minimum_proxies as usize {
        return generic_err!(format!("Required at least {} proxies.", n_minimum_proxies));
    }

    for proxy_delegation in proxy_delegations {
        // Proxy must be registered
        match store_get_proxy_entry(deps.storage, &proxy_delegation.proxy_addr) {
            None => {
                return generic_err!(format!(
                    "Unknown proxy with address {}",
                    &proxy_delegation.proxy_addr
                ));
            }
            Some(proxy_entry) => {
                if proxy_entry.proxy_pubkey.is_none() {
                    return generic_err!(format!(
                        "Unregistered proxy with address {}",
                        &proxy_delegation.proxy_addr
                    ));
                }
            }
        }

        if store_get_proxy_delegation_id(
            deps.storage,
            delegator_pubkey,
            delegatee_pubkey,
            &proxy_delegation.proxy_addr,
        )
        .is_some()
        {
            return generic_err!(format!(
                "Delegation string was already provided for proxy {}.",
                &proxy_delegation.proxy_addr
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
            &proxy_delegation.proxy_addr,
            &state.next_delegation_id,
        );
        store_add_per_proxy_delegation(
            deps.storage,
            &proxy_delegation.proxy_addr,
            &state.next_delegation_id,
        );

        state.next_delegation_id += 1;
    }

    store_set_state(deps.storage, &state)?;

    // Add delegatee labels
    if let Some(delegatee_labels) = delegatee_labels {
        store_add_delegatee_labels(
            deps.storage,
            &info.sender,
            delegatee_pubkey,
            delegatee_labels,
        );
        response.attributes.push(Attribute::new(
            "delegatee_labels",
            delegatee_labels.join(", "),
        ));
    }

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
    let mut state: State = store_get_state(deps.storage)?;
    ensure_not_terminated(&state)?;

    let staking_config: StakingConfig = store_get_staking_config(deps.storage)?;
    let timeouts_config: TimeoutsConfig = store_get_timeouts_config(deps.storage)?;

    let data_entry: DataEntry = match store_get_data_entry(deps.storage, data_id) {
        None => generic_err!("Data entry doesn't exist."),
        Some(data_entry) => Ok(data_entry),
    }?;

    let delegator_addr =
        match store_get_delegator_address(deps.storage, &data_entry.delegator_pubkey) {
            Some(delegator_addr) => Ok(delegator_addr),
            None => generic_err!("Invalid delegator pubkey."),
        }?;

    // Get selected proxies for current delegation
    let proxy_addresses = store_get_all_proxies_from_delegation(
        deps.storage,
        &data_entry.delegator_pubkey,
        delegatee_pubkey,
    );

    if proxy_addresses.is_empty() {
        return generic_err!("ProxyDelegation doesn't exist.");
    }

    // Check if encryption was permitted
    if info.sender != delegator_addr
        && !get_permission(deps.storage, &delegator_addr, delegatee_pubkey, data_id)
    {
        return generic_err!("Reencryption is not permitted.");
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
            proxy_addresses.len(),
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
        proxy_addr: Addr::unchecked(""),
        delegation_string: "".to_string(),
        resolved: false,
        abandoned: false,
        timeout_height: env.block.height + timeouts_config.timeout_height,
        refund_addr: info.sender.clone(),
    };

    let mut proxy_stake = Vec::new();

    // Assign re-encrpytion tasks to all available proxies
    for proxy_addr in &proxy_addresses {
        // Check if proxy has enough stake
        let mut proxy = store_get_proxy_entry(deps.storage, proxy_addr).unwrap();

        if proxy.stake_amount.u128() < staking_config.per_task_slash_stake_amount.u128() {
            // Proxy cannot be selected for insufficient amount
            continue;
        }

        // Subtract stake from proxy
        proxy.stake_amount = proxy
            .stake_amount
            .checked_sub(staking_config.per_task_slash_stake_amount)?;
        store_set_proxy_entry(deps.storage, proxy_addr, &proxy);

        // Get delegation
        let delegation_id = store_get_proxy_delegation_id(
            deps.storage,
            &data_entry.delegator_pubkey,
            delegatee_pubkey,
            proxy_addr,
        )
        .unwrap();
        let delegation = store_get_delegation(deps.storage, &delegation_id).unwrap();

        // Add reencryption task for each proxy
        match store_get_proxy_entry(deps.storage, proxy_addr) {
            None => generic_err!("Proxy not registered"),
            Some(proxy_entry) => match proxy_entry.proxy_pubkey {
                None => generic_err!("Proxy not registered"),
                Some(proxy_pubkey) => Ok(proxy_pubkey),
            },
        }?;

        new_proxy_task.proxy_addr = proxy_addr.clone();
        new_proxy_task.delegation_string = delegation.delegation_string;
        let task_id = state.next_proxy_task_id;
        store_set_proxy_task(deps.storage, &task_id, &new_proxy_task);
        store_add_delegatee_proxy_task(
            deps.storage,
            data_id,
            delegatee_pubkey,
            proxy_addr,
            &task_id,
        );
        store_add_proxy_task_to_queue(deps.storage, proxy_addr, &task_id);
        store_add_data_id_task(deps.storage, data_id, &task_id);
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

pub fn try_add_data_labels(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &str,
    data_labels: &[String],
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage).unwrap();
    ensure_not_terminated(&state)?;
    ensure_data_owner(deps.storage, data_id, &info.sender)?;
    store_add_data_labels(deps.storage, data_id, data_labels);

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "add_data_labels"));
    response.attributes.push(Attribute::new("data_id", data_id));
    response
        .attributes
        .push(Attribute::new("data_labels", data_labels.join(", ")));
    Ok(response)
}

pub fn try_remove_data_labels(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &str,
    data_labels: &[String],
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage).unwrap();
    ensure_not_terminated(&state)?;
    ensure_data_owner(deps.storage, data_id, &info.sender)?;
    store_remove_data_labels(deps.storage, data_id, data_labels)?;

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "remove_data_labels"));
    response.attributes.push(Attribute::new("data_id", data_id));
    response
        .attributes
        .push(Attribute::new("data_labels", data_labels.join(", ")));
    Ok(response)
}

pub fn try_add_delegatee_labels(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    delegatee_pubkey: &str,
    delegatee_labels: &[String],
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage).unwrap();
    ensure_not_terminated(&state)?;

    store_add_delegatee_labels(
        deps.storage,
        &info.sender,
        delegatee_pubkey,
        delegatee_labels,
    );

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "add_delegatee_labels"));
    response
        .attributes
        .push(Attribute::new("delegatee_pubkey", delegatee_pubkey));
    response.attributes.push(Attribute::new(
        "delegatee_labels",
        delegatee_labels.join(", "),
    ));
    Ok(response)
}

pub fn try_remove_delegatee_labels(
    mut response: Response,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    delegatee_pubkey: &str,
    delegatee_labels: &[String],
) -> StdResult<Response> {
    let state: State = store_get_state(deps.storage).unwrap();
    ensure_not_terminated(&state)?;

    store_remove_delegatee_labels(
        deps.storage,
        &info.sender,
        delegatee_pubkey,
        delegatee_labels,
    )?;

    // Return response
    response
        .attributes
        .push(Attribute::new("action", "remove_delegatee_labels"));
    response
        .attributes
        .push(Attribute::new("delegatee_pubkey", delegatee_pubkey));
    response.attributes.push(Attribute::new(
        "delegatee_labels",
        delegatee_labels.join(", "),
    ));
    Ok(response)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    let response: Response = Response::new();

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
            data_labels,
            ..
        } => try_add_data(
            response,
            deps,
            env,
            info,
            &data_id,
            &delegator_pubkey,
            &capsule,
            &data_labels,
        ),
        ExecuteMsg::RemoveData { data_id } => try_remove_data(response, deps, env, info, &data_id),
        ExecuteMsg::AddDelegation {
            delegator_pubkey,
            delegatee_pubkey,
            proxy_delegations,
            delegatee_labels,
        } => try_add_delegation(
            response,
            deps,
            env,
            info,
            &delegator_pubkey,
            &delegatee_pubkey,
            &proxy_delegations,
            &delegatee_labels,
        ),
        ExecuteMsg::RequestReencryption {
            data_id,
            delegatee_pubkey,
        } => try_request_reencryption(response, deps, env, info, &data_id, &delegatee_pubkey),
        ExecuteMsg::ResolveTimedOutRequest {
            data_id,
            delegatee_pubkey,
        } => try_resolve_timed_out_request(response, deps, env, info, &data_id, &delegatee_pubkey),

        ExecuteMsg::AddDataLabels {
            data_id,
            data_labels,
        } => try_add_data_labels(response, deps, env, info, &data_id, &data_labels),

        ExecuteMsg::RemoveDataLabels {
            data_id,
            data_labels,
        } => try_remove_data_labels(response, deps, env, info, &data_id, &data_labels),

        ExecuteMsg::AddDelegateeLabels {
            delegatee_pubkey,
            delegatee_labels,
        } => try_add_delegatee_labels(
            response,
            deps,
            env,
            info,
            &delegatee_pubkey,
            &delegatee_labels,
        ),

        ExecuteMsg::RemoveDelegateeLabels {
            delegatee_pubkey,
            delegatee_labels,
        } => try_remove_delegatee_labels(
            response,
            deps,
            env,
            info,
            &delegatee_pubkey,
            &delegatee_labels,
        ),
    }
}

//// Query functions

pub fn get_proxy_tasks(
    store: &dyn Storage,
    proxy_addr: &Addr,
    block_height: &u64,
) -> StdResult<Vec<ProxyTaskResponse>> {
    // Returns first available proxy task from queue

    let mut tasks_response: Vec<ProxyTaskResponse> = Vec::new();

    let tasks = store_get_all_proxy_tasks_in_queue(store, proxy_addr);

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
    let proxy_addresses = store_get_all_active_proxy_addresses(store);

    let mut res: Vec<ProxyAvailabilityResponse> = Vec::new();

    for proxy_addr in proxy_addresses {
        let proxy_entry: Proxy = store_get_proxy_entry(store, &proxy_addr).unwrap();

        res.push(ProxyAvailabilityResponse {
            proxy_addr,
            // If a proxy is in an active proxies map it has a pubkey
            proxy_pubkey: proxy_entry.proxy_pubkey.unwrap(),
            stake_amount: proxy_entry.stake_amount,
        });
    }

    res
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAvailableProxies {} => {
            let state = store_get_state(deps.storage)?;

            if state.terminated {
                return to_binary(&GetAvailableProxiesResponse {
                    proxies: Vec::new(),
                });
            }

            Ok(to_binary(&GetAvailableProxiesResponse {
                proxies: get_proxies_availability(deps.storage),
            })?)
        }
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
                withdrawn: state.withdrawn,
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

        QueryMsg::GetProxyTasks { proxy_addr } => {
            let state = store_get_state(deps.storage)?;

            if state.withdrawn {
                return to_binary(&GetProxyTasksResponse {
                    proxy_tasks: Vec::new(),
                });
            }

            Ok(to_binary(&GetProxyTasksResponse {
                proxy_tasks: get_proxy_tasks(deps.storage, &proxy_addr, &env.block.height)?,
            })?)
        }

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

        QueryMsg::GetProxyStatus { proxy_addr } => {
            let mut proxy_status: Option<ProxyStatusResponse> = None;

            if let Some(proxy_entry) = store_get_proxy_entry(deps.storage, &proxy_addr) {
                let staking_config = store_get_staking_config(deps.storage)?;

                proxy_status = Some(ProxyStatusResponse {
                    proxy_pubkey: proxy_entry.proxy_pubkey.clone(),
                    stake_amount: proxy_entry.stake_amount,
                    withdrawable_stake_amount: Uint128::new(get_maximum_withdrawable_stake_amount(
                        &staking_config,
                        &proxy_entry,
                    )),
                    proxy_state: proxy_entry.state,
                })
            }

            Ok(to_binary(&GetProxyStatusResponse { proxy_status })?)
        }
        QueryMsg::GetDataLabels { data_id } => Ok(to_binary(&GetDataLabelsResponse {
            data_labels: store_get_all_data_labels(deps.storage, &data_id),
        })?),
        QueryMsg::GetDelegateeLabels {
            delegator_addr,
            delegatee_pubkey,
        } => Ok(to_binary(&GetDelegateeLabelsResponse {
            delegatee_labels: store_get_all_delegatee_labels(
                deps.storage,
                &delegator_addr,
                &delegatee_pubkey,
            ),
        })?),
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

fn ensure_data_owner(
    storage: &mut dyn Storage,
    data_id: &str,
    delegator_address: &Addr,
) -> StdResult<()> {
    let data_entry: DataEntry = match store_get_data_entry(storage, data_id) {
        None => generic_err!("Data entry doesn't exist."),
        Some(data_entry) => Ok(data_entry),
    }?;

    let correct_delegator_addr: Addr =
        store_get_delegator_address(storage, &data_entry.delegator_pubkey).unwrap();

    if &correct_delegator_addr != delegator_address {
        return generic_err!("Sender is not a data owner.");
    }

    Ok(())
}

fn ensure_not_terminated(state: &State) -> StdResult<()> {
    if state.terminated {
        return generic_err!("Contract was terminated.");
    }

    Ok(())
}

fn ensure_not_withdrawn(state: &State) -> StdResult<()> {
    if state.withdrawn {
        return generic_err!("Remaining balances from contract were already withdrawn.");
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
