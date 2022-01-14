use crate::msg::{
    ExecuteMsg, GetAvailableProxiesResponse, GetContractStateResponse, GetDataIDResponse,
    GetDelegationStatusResponse, GetFragmentsResponse, GetNextProxyTaskResponse,
    GetProxyStatusResponse, GetSelectedProxiesForDelegationResponse, GetStakingConfigResponse,
    InstantiateMsg, ProxyDelegationString, ProxyStatus, ProxyTask, QueryMsg,
};
use crate::proxies::{
    maximum_withdrawable_stake_amount, store_get_all_active_proxy_pubkeys, store_get_proxy_address,
    store_get_proxy_entry, store_remove_proxy_address, store_remove_proxy_entry,
    store_set_is_proxy_active, store_set_proxy_address, store_set_proxy_entry, Proxy, ProxyState,
};
use crate::state::{
    store_get_data_entry, store_get_delegator_address, store_get_staking_config, store_get_state,
    store_set_data_entry, store_set_delegator_address, store_set_staking_config, store_set_state,
    DataEntry, StakingConfig, State,
};

use crate::delegations::{
    get_delegation_state, get_n_available_proxies_from_delegation,
    get_n_minimum_proxies_for_refund, remove_proxy_delegations, store_add_per_proxy_delegation,
    store_get_all_proxies_from_delegation, store_get_delegation, store_get_proxy_delegation_id,
    store_is_proxy_delegation_empty, store_set_delegation, store_set_delegation_id,
    ProxyDelegation,
};
use crate::reencryption_requests::{
    get_all_fragments, get_reencryption_request_state, remove_proxy_reencryption_requests,
    store_add_delegatee_proxy_reencryption_request, store_add_proxy_reencryption_request_to_queue,
    store_get_all_proxy_reencryption_requests_in_queue,
    store_get_delegatee_proxy_reencryption_request, store_get_parent_reencryption_request,
    store_get_proxy_reencryption_request, store_remove_proxy_reencryption_request_from_queue,
    store_set_parent_reencryption_request, store_set_proxy_reencryption_request,
    ParentReencryptionRequest, ProxyReencryptionRequest, ReencryptionRequestState,
};
use cosmwasm_std::{
    attr, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Storage, Uint128,
};

use crate::common::add_bank_msg;

use umbral_pre::{Capsule, CapsuleFrag, DeserializableFromArray, PublicKey};

macro_rules! generic_err {
    ($val:expr) => {
        Err(StdError::generic_err($val))
    };
}

pub const DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT: u128 = 1000;
pub const DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT: u128 = 100;
pub const DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT: u128 = 100;
pub const FRAGMENT_VERIFICATION_ERROR: &str = "Fragment verification failed: ";

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        admin: msg.admin.unwrap_or(info.sender),
        n_max_proxies: msg.n_max_proxies.unwrap_or(u32::MAX),
        threshold: msg.threshold.unwrap_or(1),
        next_proxy_request_id: 0,
        next_delegation_id: 0,
    };

    if state.threshold == 0 {
        return generic_err!("Threshold cannot be 0");
    }

    let staking_config = StakingConfig {
        stake_denom: msg.stake_denom,
        minimum_proxy_stake_amount: msg
            .minimum_proxy_stake_amount
            .unwrap_or_else(|| Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT)),
        minimum_request_reward_amount: msg
            .minimum_request_reward_amount
            .unwrap_or_else(|| Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT)),
        per_request_slash_stake_amount: msg
            .per_request_slash_stake_amount
            .unwrap_or_else(|| Uint128::new(DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT)),
    };
    store_set_staking_config(deps.storage, &staking_config)?;

    if state.n_max_proxies < get_n_minimum_proxies_for_refund(&state, &staking_config) {
        return generic_err!(
            "Value of n_max_proxies cannot be lower than minimum proxies to refund delegator."
        );
    }

    store_set_state(deps.storage, &state)?;

    let new_proxy = Proxy {
        state: ProxyState::Authorised,
        proxy_pubkey: None,
        stake_amount: Uint128::new(0),
    };

    if let Some(proxies_addr) = msg.proxies {
        for proxy_addr in proxies_addr {
            store_set_proxy_entry(deps.storage, &proxy_addr, &new_proxy);
        }
    };

    Ok(Response::default())
}

// Admin actions

fn try_add_proxy(
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
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "add_proxy"),
            attr("admin", info.sender.as_str()),
            attr("proxy_addr", proxy_addr.as_str()),
        ],
        data: None,
    };
    Ok(res)
}

fn try_remove_proxy(
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
        None => generic_err!(format!("{} is not a proxy", proxy_addr)),
        Some(proxy) => Ok(proxy),
    }?;

    // Return response
    let mut res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "store_remove_proxy_entry"),
            attr("admin", info.sender.as_str()),
            attr("proxy_addr", proxy_addr.as_str()),
        ],
        data: None,
    };

    // Registered or Leaving state
    if let Some(proxy_pubkey) = proxy.proxy_pubkey {
        // In leaving state this was already done
        if proxy.state != ProxyState::Leaving {
            store_set_is_proxy_active(deps.storage, &proxy_pubkey, false);
            remove_proxy_delegations(deps.storage, &proxy_pubkey)?;
        }

        remove_proxy_reencryption_requests(deps.storage, &proxy_pubkey, &mut res)?;
        store_remove_proxy_address(deps.storage, &proxy_pubkey);
    }

    // Update proxy entry to get correct stake amount after possible slashing
    proxy = store_get_proxy_entry(deps.storage, proxy_addr).unwrap();

    // Return remaining stake back to proxy
    add_bank_msg(
        &mut res,
        proxy_addr,
        proxy.stake_amount.u128(),
        &staking_config.stake_denom,
    );

    // Remove proxy entry = remove pubkey
    store_remove_proxy_entry(deps.storage, proxy_addr);

    Ok(res)
}

// Proxy actions

fn try_register_proxy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proxy_pubkey: String,
) -> StdResult<Response> {
    let staking_config = store_get_staking_config(deps.storage)?;

    let mut proxy = match store_get_proxy_entry(deps.storage, &info.sender) {
        None => generic_err!("Sender is not a proxy."),
        Some(proxy) => Ok(proxy),
    }?;

    if proxy.proxy_pubkey.is_some() {
        return generic_err!("Proxy already registered.");
    }

    if store_get_proxy_address(deps.storage, &proxy_pubkey).is_some() {
        return generic_err!("Pubkey already used.");
    }

    ensure_stake(
        &staking_config,
        &info.funds,
        &staking_config.minimum_proxy_stake_amount.u128(),
    )?;

    proxy.proxy_pubkey = Some(proxy_pubkey.clone());
    proxy.state = ProxyState::Registered;
    proxy.stake_amount = Uint128::new(proxy.stake_amount.u128() + info.funds[0].amount.u128());
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);
    store_set_is_proxy_active(deps.storage, &proxy_pubkey, true);
    store_set_proxy_address(deps.storage, &proxy_pubkey, &info.sender);

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "register_proxy"),
            attr("proxy", info.sender.as_str()),
        ],
        data: None,
    };
    Ok(res)
}

fn try_unregister_proxy(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
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

    // Return response
    let mut res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "unregister_proxy"),
            attr("proxy", info.sender.as_str()),
        ],
        data: None,
    };

    if proxy.state != ProxyState::Leaving {
        store_set_is_proxy_active(deps.storage, &proxy_pubkey, false);
        remove_proxy_delegations(deps.storage, &proxy_pubkey)?;
    }

    // This can resolve to proxy being slashed
    remove_proxy_reencryption_requests(deps.storage, &proxy_pubkey, &mut res)?;
    store_remove_proxy_address(deps.storage, &proxy_pubkey);

    // Update proxy entry to get correct stake amount after possible slashing
    proxy = store_get_proxy_entry(deps.storage, &info.sender).unwrap();

    // Return remaining stake back to proxy
    add_bank_msg(
        &mut res,
        &info.sender,
        proxy.stake_amount.u128(),
        &staking_config.stake_denom,
    );

    proxy.stake_amount = Uint128::new(0);
    proxy.state = ProxyState::Authorised;
    proxy.proxy_pubkey = None;
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);

    Ok(res)
}

fn try_deactivate_proxy(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
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
            remove_proxy_delegations(deps.storage, &proxy_pubkey)?;

            proxy.state = ProxyState::Leaving;
            store_set_proxy_entry(deps.storage, &info.sender, &proxy);
        }
    }

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "deactivate_proxy"),
            attr("proxy", info.sender.as_str()),
        ],
        data: None,
    };
    Ok(res)
}

fn try_provide_reencrypted_fragment(
    deps: DepsMut,
    _env: Env,
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

    // Get request_id or return error
    let request_id: u64 = match store_get_delegatee_proxy_reencryption_request(
        deps.storage,
        data_id,
        delegatee_pubkey,
        &proxy_pubkey,
    ) {
        None => generic_err!("This fragment was not requested."),
        Some(request_id) => Ok(request_id),
    }?;

    // Request must exist - panic otherwise
    let mut proxy_request =
        store_get_proxy_reencryption_request(deps.storage, &request_id).unwrap();

    if proxy_request.fragment.is_some() {
        return generic_err!("Fragment already provided.");
    }

    let data_entry = store_get_data_entry(deps.storage, data_id).unwrap();

    verify_fragment(
        &fragment.to_string(),
        &data_entry.capsule,
        &data_entry.delegator_pubkey,
        &proxy_request.delegatee_pubkey,
    )?;

    if get_all_fragments(deps.storage, data_id, delegatee_pubkey).contains(&fragment.to_string()) {
        return generic_err!("Fragment already provided by other proxy.");
    }

    // Prepare return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "try_provide_reencrypted_fragment"),
            attr("data_id", data_id),
            attr("delegatee_pubkey", delegatee_pubkey),
            attr("fragment", fragment),
        ],
        data: None,
    };

    let state = store_get_state(deps.storage)?;
    let mut parent_request = store_get_parent_reencryption_request(
        deps.storage,
        &proxy_request.data_id,
        &proxy_request.delegatee_pubkey,
    )
    .unwrap();
    parent_request.n_provided_fragments += 1;
    if parent_request.n_provided_fragments >= state.threshold {
        parent_request.state = ReencryptionRequestState::Granted;
    }
    store_set_parent_reencryption_request(
        deps.storage,
        &proxy_request.data_id,
        &proxy_request.delegatee_pubkey,
        &parent_request,
    );

    // Add reward to proxy stake + return withdrawn stake
    proxy.stake_amount += proxy_request.reward_amount + proxy_request.proxy_slashed_amount;
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);

    // Add fragment to request and update funds amount
    proxy_request.reward_amount = Uint128::new(0);
    proxy_request.proxy_slashed_amount = Uint128::new(0);
    proxy_request.fragment = Some(fragment.to_string());
    store_set_proxy_reencryption_request(deps.storage, &request_id, &proxy_request);

    // Remove request from proxy queue as it's completed
    store_remove_proxy_reencryption_request_from_queue(deps.storage, &proxy_pubkey, &request_id);

    Ok(res)
}

fn try_withdraw_stake(
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
    let maximum_withdrawable_amount = maximum_withdrawable_stake_amount(&staking_config, &proxy);

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

    // Return response
    let mut res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "withdraw_stake"),
            attr("stake_amount", withdraw_stake_amount),
        ],
        data: None,
    };

    // Update proxy stake amount
    proxy.stake_amount = Uint128::new(proxy.stake_amount.u128() - withdraw_stake_amount);
    store_set_proxy_entry(deps.storage, &info.sender, &proxy);
    add_bank_msg(
        &mut res,
        &info.sender,
        withdraw_stake_amount,
        &staking_config.stake_denom,
    );

    Ok(res)
}

fn try_add_stake(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
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
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "add_stake"),
            attr("stake_amount", info.funds[0].amount),
        ],
        data: None,
    };
    Ok(res)
}

// Delegator actions

fn try_add_data(
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
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "add_data"),
            attr("owner", info.sender.as_str()),
            attr("data_id", data_id),
            attr("delegator_pubkey", delegator_pubkey),
        ],
        data: None,
    };
    Ok(res)
}

fn try_request_proxies_for_delegation(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
) -> StdResult<Response> {
    let mut state = store_get_state(deps.storage)?;

    ensure_delegator(deps.storage, delegator_pubkey, &info.sender)?;

    if !store_is_proxy_delegation_empty(deps.storage, delegator_pubkey, delegatee_pubkey) {
        return generic_err!("Delegation already exist");
    }

    let selected_proxy_pubkeys = select_proxy_pubkeys(deps.storage)?;
    let mut selected_proxy_pubkeys_str: String = String::from("[");

    // Create individual proxy delegations
    let delegation = ProxyDelegation {
        delegator_pubkey: delegator_pubkey.to_string(),
        delegatee_pubkey: delegatee_pubkey.to_string(),
        delegation_string: None,
    };

    for proxy_pubkey in selected_proxy_pubkeys {
        store_set_delegation(deps.storage, &state.next_delegation_id, &delegation);
        store_set_delegation_id(
            deps.storage,
            delegator_pubkey,
            delegatee_pubkey,
            &proxy_pubkey,
            &state.next_delegation_id,
        );
        store_add_per_proxy_delegation(deps.storage, &proxy_pubkey, &state.next_delegation_id);

        selected_proxy_pubkeys_str += format!("\"{}\", ", proxy_pubkey).as_str();

        state.next_delegation_id += 1;
    }
    selected_proxy_pubkeys_str += "]";

    store_set_state(deps.storage, &state)?;
    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "request_proxies_for_delegation"),
            attr("delegator_address", info.sender.as_str()),
            attr("delegator_pubkey", delegator_pubkey),
            attr("delegatee_pubkey", delegatee_pubkey),
            attr("selected_proxies", selected_proxy_pubkeys_str),
        ],
        data: None,
    };
    Ok(res)
}

fn try_add_delegation(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_delegations: &[ProxyDelegationString],
) -> StdResult<Response> {
    ensure_delegator(deps.storage, delegator_pubkey, &info.sender)?;

    let n_expected_strings =
        store_get_all_proxies_from_delegation(deps.storage, delegator_pubkey, delegatee_pubkey)
            .len();

    if n_expected_strings == 0 {
        return generic_err!("No proxies selected for this delegation");
    }

    if n_expected_strings != proxy_delegations.len() {
        return generic_err!(format!(
            "Provided wrong number of delegation strings, expected {} got {}.",
            n_expected_strings,
            proxy_delegations.len()
        ));
    }

    for proxy_delegation in proxy_delegations {
        let delegation_id = match store_get_proxy_delegation_id(
            deps.storage,
            delegator_pubkey,
            delegatee_pubkey,
            &proxy_delegation.proxy_pubkey,
        ) {
            None => generic_err!(format!(
                "Proxy {} not selected for delegation.",
                proxy_delegation.proxy_pubkey
            )),
            Some(delegation_id) => Ok(delegation_id),
        }?;

        let mut delegation = store_get_delegation(deps.storage, &delegation_id).unwrap();

        // ProxyDelegation requested and strings already provided
        if delegation.delegation_string.is_some() {
            return generic_err!("ProxyDelegation strings already provided");
        }

        delegation.delegation_string = Some(proxy_delegation.delegation_string.clone());
        store_set_delegation(deps.storage, &delegation_id, &delegation);
    }

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "add_delegation"),
            attr("delegator_address", info.sender.as_str()),
            attr("delegator_pubkey", delegator_pubkey),
            attr("delegatee_pubkey", delegatee_pubkey),
        ],
        data: None,
    };
    Ok(res)
}

fn try_request_reencryption(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &str,
    delegatee_pubkey: &str,
) -> StdResult<Response> {
    // Only data owner can request reencryption
    let data_entry: DataEntry = match store_get_data_entry(deps.storage, data_id) {
        None => generic_err!("Data entry doesn't exist."),
        Some(data_entry) => Ok(data_entry),
    }?;

    ensure_delegator(deps.storage, &data_entry.delegator_pubkey, &info.sender)?;

    // Load config
    let staking_config = store_get_staking_config(deps.storage)?;
    let mut state = store_get_state(deps.storage)?;

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
        &staking_config.minimum_request_reward_amount.u128(),
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

    // Ensure more than minimum_request_reward_amount * number_of_proxies of stake provided
    ensure_stake(
        &staking_config,
        &info.funds,
        &(staking_config.minimum_request_reward_amount.u128() * n_available_proxies as u128),
    )?;

    // Prepare template for each proxy request
    let mut new_proxy_request = ProxyReencryptionRequest {
        delegatee_pubkey: delegatee_pubkey.to_string(),
        data_id: data_id.to_string(),
        fragment: None,
        proxy_pubkey: "".to_string(),
        delegation_string: "".to_string(),
        // Per proxy stake amount
        reward_amount: Uint128::new(info.funds[0].amount.u128() / n_available_proxies as u128),
        proxy_slashed_amount: staking_config.per_request_slash_stake_amount,
    };

    if store_get_parent_reencryption_request(deps.storage, data_id, delegatee_pubkey).is_some() {
        return generic_err!("Reencryption already requested");
    }

    // Assign re-encrpytion requests to all available proxies
    for proxy_pubkey in &proxy_pubkeys {
        // Check if proxy has enough stake
        let proxy_addr = store_get_proxy_address(deps.storage, proxy_pubkey).unwrap();
        let mut proxy = store_get_proxy_entry(deps.storage, &proxy_addr).unwrap();

        if proxy.stake_amount.u128() < new_proxy_request.proxy_slashed_amount.u128() {
            // Proxy cannot be selected for insufficient amount
            continue;
        }

        // Subtract stake from proxy - stake is reserved in re-encryption request
        proxy.stake_amount =
            Uint128::new(proxy.stake_amount.u128() - new_proxy_request.proxy_slashed_amount.u128());
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

        // Can happen only when you request re-encryption before providing delegation strings for proxies
        let delegation_string: String = match delegation.delegation_string {
            None => generic_err!("Not all delegation strings provided"),
            Some(delegation_string) => Ok(delegation_string),
        }?;

        // Add reencryption task for each proxy
        new_proxy_request.proxy_pubkey = proxy_pubkey.clone();
        new_proxy_request.delegation_string = delegation_string;
        let request_id = state.next_proxy_request_id;
        store_set_proxy_reencryption_request(deps.storage, &request_id, &new_proxy_request);
        store_add_delegatee_proxy_reencryption_request(
            deps.storage,
            data_id,
            delegatee_pubkey,
            proxy_pubkey,
            &request_id,
        );
        store_add_proxy_reencryption_request_to_queue(deps.storage, proxy_pubkey, &request_id);
        state.next_proxy_request_id += 1;
    }

    // Create parent request
    let parent_request = ParentReencryptionRequest {
        n_provided_fragments: 0,
        n_proxy_requests: n_available_proxies,
        slashed_stake_amount: Uint128::new(0),
        state: ReencryptionRequestState::Ready,
        delegator_addr: info.sender,
    };
    store_set_parent_reencryption_request(deps.storage, data_id, delegatee_pubkey, &parent_request);

    store_set_state(deps.storage, &state)?;

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "request_reencryption"),
            attr("data_id", &data_id),
            attr("delegatee_pubkey", &delegatee_pubkey),
        ],
        data: None,
    };
    Ok(res)
}

pub fn get_next_proxy_task(
    store: &dyn Storage,
    proxy_pubkey: &str,
) -> StdResult<Option<ProxyTask>> {
    let requests = store_get_all_proxy_reencryption_requests_in_queue(store, proxy_pubkey);

    if requests.is_empty() {
        return Ok(None);
    }
    // Request must exist
    let request = store_get_proxy_reencryption_request(store, &requests[0]).unwrap();

    let data_entry = store_get_data_entry(store, &request.data_id).unwrap();

    let proxy_task = ProxyTask {
        data_id: request.data_id.clone(),
        capsule: data_entry.capsule.clone(),
        delegatee_pubkey: request.delegatee_pubkey,
        delegator_pubkey: data_entry.delegator_pubkey,
        delegation_string: request.delegation_string,
    };

    Ok(Some(proxy_task))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        // Admin actions
        ExecuteMsg::AddProxy { proxy_addr } => try_add_proxy(deps, env, info, &proxy_addr),
        ExecuteMsg::RemoveProxy { proxy_addr } => try_remove_proxy(deps, env, info, &proxy_addr),

        // Proxy actions
        ExecuteMsg::RegisterProxy { proxy_pubkey } => {
            try_register_proxy(deps, env, info, proxy_pubkey)
        }
        ExecuteMsg::ProvideReencryptedFragment {
            data_id,
            delegatee_pubkey,
            fragment,
        } => try_provide_reencrypted_fragment(
            deps,
            env,
            info,
            &data_id,
            &delegatee_pubkey,
            &fragment,
        ),
        ExecuteMsg::UnregisterProxy {} => try_unregister_proxy(deps, env, info),
        ExecuteMsg::DeactivateProxy {} => try_deactivate_proxy(deps, env, info),
        ExecuteMsg::WithdrawStake { stake_amount } => {
            try_withdraw_stake(deps, env, info, &stake_amount)
        }
        ExecuteMsg::AddStake {} => try_add_stake(deps, env, info),

        // Delegator actions
        ExecuteMsg::AddData {
            data_id,
            delegator_pubkey,
            capsule,
        } => try_add_data(deps, env, info, &data_id, &delegator_pubkey, &capsule),
        ExecuteMsg::AddDelegation {
            delegator_pubkey,
            delegatee_pubkey,
            proxy_delegations,
        } => try_add_delegation(
            deps,
            env,
            info,
            &delegator_pubkey,
            &delegatee_pubkey,
            &proxy_delegations,
        ),
        ExecuteMsg::RequestProxiesForDelegation {
            delegator_pubkey,
            delegatee_pubkey,
        } => try_request_proxies_for_delegation(
            deps,
            env,
            info,
            &delegator_pubkey,
            &delegatee_pubkey,
        ),
        ExecuteMsg::RequestReencryption {
            data_id,
            delegatee_pubkey,
        } => try_request_reencryption(deps, env, info, &data_id, &delegatee_pubkey),
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAvailableProxies {} => Ok(to_binary(&GetAvailableProxiesResponse {
            proxy_pubkeys: store_get_all_active_proxy_pubkeys(deps.storage),
        })?),
        QueryMsg::GetDataID { data_id } => Ok(to_binary(&GetDataIDResponse {
            data_entry: store_get_data_entry(deps.storage, &data_id),
        })?),
        QueryMsg::GetFragments {
            data_id,
            delegatee_pubkey,
        } => {
            let state = store_get_state(deps.storage)?;
            Ok(to_binary(&GetFragmentsResponse {
                reencryption_request_state: get_reencryption_request_state(
                    deps.storage,
                    &data_id,
                    &delegatee_pubkey,
                ),
                fragments: get_all_fragments(deps.storage, &data_id, &delegatee_pubkey),
                threshold: state.threshold,
            })?)
        }
        QueryMsg::GetContractState {} => {
            let state = store_get_state(deps.storage)?;

            Ok(to_binary(&GetContractStateResponse {
                admin: state.admin,
                threshold: state.threshold,
                n_max_proxies: state.n_max_proxies,
            })?)
        }

        QueryMsg::GetStakingConfig {} => {
            let staking_config = store_get_staking_config(deps.storage)?;

            Ok(to_binary(&GetStakingConfigResponse {
                stake_denom: staking_config.stake_denom,
                minimum_proxy_stake_amount: staking_config.minimum_proxy_stake_amount,
                minimum_request_reward_amount: staking_config.minimum_request_reward_amount,
            })?)
        }

        QueryMsg::GetNextProxyTask { proxy_pubkey } => Ok(to_binary(&GetNextProxyTaskResponse {
            proxy_task: get_next_proxy_task(deps.storage, &proxy_pubkey)?,
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
                &staking_config.per_request_slash_stake_amount.u128(),
            );

            let minimum_stake_amount =
                n_availbale_proxies as u128 * staking_config.minimum_request_reward_amount.u128();

            Ok(to_binary(&GetDelegationStatusResponse {
                delegation_state: get_delegation_state(
                    deps.storage,
                    &delegator_pubkey,
                    &delegatee_pubkey,
                ),
                minimum_request_reward: Coin {
                    denom: staking_config.stake_denom,
                    amount: Uint128::new(minimum_stake_amount),
                },
            })?)
        }

        QueryMsg::GetSelectedProxiesForDelegation {
            delegator_pubkey,
            delegatee_pubkey,
        } => Ok(to_binary(&GetSelectedProxiesForDelegationResponse {
            proxy_pubkeys: store_get_all_proxies_from_delegation(
                deps.storage,
                &delegator_pubkey,
                &delegatee_pubkey,
            ),
        })?),

        QueryMsg::GetProxyStatus { proxy_pubkey } => {
            let mut proxy_status: Option<ProxyStatus> = None;

            if let Some(proxy_addr) = store_get_proxy_address(deps.storage, &proxy_pubkey) {
                let proxy = store_get_proxy_entry(deps.storage, &proxy_addr).unwrap();
                let staking_config = store_get_staking_config(deps.storage)?;

                proxy_status = Some(ProxyStatus {
                    proxy_address: proxy_addr,
                    stake_amount: proxy.stake_amount,
                    withdrawable_stake_amount: Uint128::new(maximum_withdrawable_stake_amount(
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

fn ensure_stake(
    staking_config: &StakingConfig,
    funds: &[Coin],
    required_stake: &u128,
) -> StdResult<()> {
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
    Ok(())
}

fn select_proxy_pubkeys(store: &dyn Storage) -> StdResult<Vec<String>> {
    let state: State = store_get_state(store)?;
    let staking_config: StakingConfig = store_get_staking_config(store)?;

    let proxy_pubkeys = store_get_all_active_proxy_pubkeys(store);

    // Select n_max_proxies or maximum possible
    let n_proxies = std::cmp::min(state.n_max_proxies as usize, proxy_pubkeys.len());

    let n_min_proxies = get_n_minimum_proxies_for_refund(&state, &staking_config);

    if n_proxies < n_min_proxies as usize {
        return generic_err!("Less than minimum proxies registered");
    }

    Ok(proxy_pubkeys[0..n_proxies].to_vec())
}

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
