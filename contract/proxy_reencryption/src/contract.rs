use crate::msg::{
    ExecuteMsg, GetAvailableProxiesResponse, GetContractStateResponse, GetDataIDResponse,
    GetDelegationStateResponse, GetFragmentsResponse, GetNextProxyTaskResponse,
    GetProxyInfoResponse, GetSelectedProxiesForDelegationResponse, InstantiateMsg, ProxyDelegation,
    ProxyInfo, ProxyTask, QueryMsg,
};
use crate::proxies::{
    get_all_active_proxy_pubkeys, get_proxy, get_proxy_address, maximum_withdrawable_stake_amount,
    remove_proxy, remove_proxy_address, set_is_proxy_active, set_proxy, set_proxy_address, Proxy,
    ProxyState,
};
use crate::state::{
    get_data_entry, get_delegator_address, get_state, set_data_entry, set_delegator_address,
    set_state, DataEntry, State,
};

use crate::delegations::{
    add_proxy_delegation, get_all_proxies_from_delegation, get_delegation, get_delegation_id,
    get_delegation_state, remove_proxy_delegations, set_delegation, set_delegation_id, Delegation,
};
use crate::reencryption_requests::{
    add_delegatee_reencryption_request, add_proxy_reencryption_request, get_all_fragments,
    get_all_proxy_reencryption_requests, get_delegatee_reencryption_request,
    get_reencryption_request, get_reencryption_request_state, remove_proxy_reencryption_request,
    remove_proxy_reencryption_requests, set_reencryption_request, ReencryptionRequest,
};
use cosmwasm_std::{
    attr, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Storage, Uint128,
};

use crate::common::add_bank_msg;

macro_rules! generic_err {
    ($val:expr) => {
        Err(StdError::generic_err($val))
    };
}

pub const DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT: u128 = 100;
pub const DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT: u128 = 100;

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
        next_request_id: 0,
        next_delegation_id: 0,
        stake_denom: msg.stake_denom,
        minimum_proxy_stake_amount: msg
            .minimum_proxy_stake_amount
            .unwrap_or_else(|| Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT)),
        minimum_request_reward_amount: msg
            .minimum_request_reward_amount
            .unwrap_or_else(|| Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT)),
    };

    if state.threshold == 0 {
        return generic_err!("Threshold cannot be 0");
    }

    if state.n_max_proxies < state.threshold {
        return generic_err!("Value of n_max_proxies cannot be lower than threshold.");
    }

    set_state(deps.storage, &state)?;

    let new_proxy = Proxy {
        state: ProxyState::Authorised,
        proxy_pubkey: None,
        stake_amount: Uint128::new(0),
    };

    if let Some(proxies_addr) = msg.proxies {
        for proxy_addr in proxies_addr {
            set_proxy(deps.storage, &proxy_addr, &new_proxy);
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
    let state: State = get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    if get_proxy(deps.storage, proxy_addr).is_some() {
        return generic_err!(format!("{} is already proxy", proxy_addr));
    }

    let new_proxy = Proxy {
        state: ProxyState::Authorised,
        proxy_pubkey: None,
        stake_amount: Uint128::new(0),
    };

    set_proxy(deps.storage, proxy_addr, &new_proxy);

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
    let state: State = get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    // check if proxy_addr is authorised
    let proxy = match get_proxy(deps.storage, proxy_addr) {
        None => generic_err!(format!("{} is not a proxy", proxy_addr)),
        Some(proxy) => Ok(proxy),
    }?;

    // Return response
    let mut res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "remove_proxy"),
            attr("admin", info.sender.as_str()),
            attr("proxy_addr", proxy_addr.as_str()),
        ],
        data: None,
    };

    // Return stake back to proxy
    add_bank_msg(
        &mut res,
        proxy_addr,
        proxy.stake_amount.u128(),
        &state.stake_denom,
    );

    // Registered or Leaving state
    if let Some(proxy_pubkey) = proxy.proxy_pubkey {
        // In leaving state this was already done
        if proxy.state != ProxyState::Leaving {
            set_is_proxy_active(deps.storage, &proxy_pubkey, false);
            remove_proxy_delegations(deps.storage, &proxy_pubkey)?;
        }

        remove_proxy_reencryption_requests(deps.storage, &proxy_pubkey, &mut res)?;
        remove_proxy_address(deps.storage, &proxy_pubkey);
    }
    // Authorised state - proxy wasn't registered
    remove_proxy(deps.storage, proxy_addr);

    Ok(res)
}

// Proxy actions

fn try_register_proxy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proxy_pubkey: String,
) -> StdResult<Response> {
    let state = get_state(deps.storage)?;

    let mut proxy = match get_proxy(deps.storage, &info.sender) {
        None => generic_err!("Sender is not a proxy."),
        Some(proxy) => Ok(proxy),
    }?;

    if proxy.proxy_pubkey.is_some() {
        return generic_err!("Proxy already registered.");
    }

    if get_proxy_address(deps.storage, &proxy_pubkey).is_some() {
        return generic_err!("Pubkey already used.");
    }

    ensure_stake(
        &state,
        &info.funds,
        &state.minimum_proxy_stake_amount.u128(),
    )?;

    proxy.proxy_pubkey = Some(proxy_pubkey.clone());
    proxy.state = ProxyState::Registered;
    proxy.stake_amount = Uint128::new(proxy.stake_amount.u128() + info.funds[0].amount.u128());
    set_proxy(deps.storage, &info.sender, &proxy);
    set_is_proxy_active(deps.storage, &proxy_pubkey, true);
    set_proxy_address(deps.storage, &proxy_pubkey, &info.sender);

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
    let state = get_state(deps.storage)?;

    // Check if proxy is authorised
    let mut proxy = match get_proxy(deps.storage, &info.sender) {
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

    // Return stake back to proxy
    add_bank_msg(
        &mut res,
        &info.sender,
        proxy.stake_amount.u128(),
        &state.stake_denom,
    );

    if proxy.state != ProxyState::Leaving {
        set_is_proxy_active(deps.storage, &proxy_pubkey, false);
        remove_proxy_delegations(deps.storage, &proxy_pubkey)?;
    }

    remove_proxy_reencryption_requests(deps.storage, &proxy_pubkey, &mut res)?;
    remove_proxy_address(deps.storage, &proxy_pubkey);
    proxy.stake_amount = Uint128::new(0);
    proxy.state = ProxyState::Authorised;
    proxy.proxy_pubkey = None;
    set_proxy(deps.storage, &info.sender, &proxy);

    Ok(res)
}

fn try_deactivate_proxy(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    match get_proxy(deps.storage, &info.sender) {
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

            set_is_proxy_active(deps.storage, &proxy_pubkey, false);
            remove_proxy_delegations(deps.storage, &proxy_pubkey)?;

            proxy.state = ProxyState::Leaving;
            set_proxy(deps.storage, &info.sender, &proxy);
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
    let mut proxy = match get_proxy(deps.storage, &info.sender) {
        None => generic_err!("Proxy not registered"),
        Some(proxy) => Ok(proxy),
    }?;

    let proxy_pubkey = match proxy.proxy_pubkey.clone() {
        None => generic_err!("Proxy not active"),
        Some(proxy_pubkey) => Ok(proxy_pubkey),
    }?;

    // Get request_id or return error
    let request_id: u64 = match get_delegatee_reencryption_request(
        deps.storage,
        data_id,
        delegatee_pubkey,
        &proxy_pubkey,
    ) {
        None => generic_err!("This fragment was not requested."),
        Some(request_id) => Ok(request_id),
    }?;

    // Request must exist - panic otherwise
    let mut request = get_reencryption_request(deps.storage, &request_id).unwrap();

    if request.fragment.is_some() {
        return generic_err!("Fragment already provided.");
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

    // Add reward to proxy stake
    proxy.stake_amount += request.reward_amount;
    set_proxy(deps.storage, &info.sender, &proxy);

    // Add fragment to fragments store
    request.fragment = Some(fragment.to_string());
    request.reward_amount = Uint128::new(0);
    set_reencryption_request(deps.storage, &request_id, &request);

    // Remove request as it's completed
    remove_proxy_reencryption_request(deps.storage, &proxy_pubkey, &request_id);

    Ok(res)
}

fn try_withdraw_stake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    stake_amount: &Option<Uint128>,
) -> StdResult<Response> {
    let state = get_state(deps.storage)?;

    // Check if proxy is authorised
    let mut proxy = match get_proxy(deps.storage, &info.sender) {
        None => generic_err!("Sender is not a proxy"),
        Some(proxy) => Ok(proxy),
    }?;

    // Check maximum amount that can be withdrawn
    let maximum_withdrawable_amount = maximum_withdrawable_stake_amount(&state, &proxy);

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
    set_proxy(deps.storage, &info.sender, &proxy);
    add_bank_msg(
        &mut res,
        &info.sender,
        withdraw_stake_amount,
        &state.stake_denom,
    );

    Ok(res)
}

// Delegator actions

fn try_add_data(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &str,
    delegator_pubkey: &str,
) -> StdResult<Response> {
    if get_data_entry(deps.storage, data_id).is_some() {
        return generic_err!(format!("Entry with ID {} already exist.", data_id));
    }

    ensure_delegator(deps.storage, delegator_pubkey, &info.sender)?;

    let entry = DataEntry {
        delegator_pubkey: delegator_pubkey.to_string(),
    };
    set_data_entry(deps.storage, data_id, &entry);

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
    let mut state = get_state(deps.storage)?;

    ensure_delegator(deps.storage, delegator_pubkey, &info.sender)?;

    let selected_proxy_pubkeys = select_proxy_pubkeys(deps.storage)?;
    let mut selected_proxy_pubkeys_str: String = String::from("[");

    let delegation = Delegation {
        delegator_pubkey: delegator_pubkey.to_string(),
        delegatee_pubkey: delegatee_pubkey.to_string(),
        delegation_string: None,
    };

    for proxy_pubkey in selected_proxy_pubkeys {
        set_delegation(deps.storage, &state.next_delegation_id, &delegation);
        set_delegation_id(
            deps.storage,
            delegator_pubkey,
            delegatee_pubkey,
            &proxy_pubkey,
            &state.next_delegation_id,
        );
        add_proxy_delegation(deps.storage, &proxy_pubkey, &state.next_delegation_id);

        selected_proxy_pubkeys_str += format!("\"{}\", ", proxy_pubkey).as_str();

        state.next_delegation_id += 1;
    }
    selected_proxy_pubkeys_str += "]";

    set_state(deps.storage, &state)?;
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
    proxy_delegations: &[ProxyDelegation],
) -> StdResult<Response> {
    ensure_delegator(deps.storage, delegator_pubkey, &info.sender)?;

    let n_expected_strings =
        get_all_proxies_from_delegation(deps.storage, delegator_pubkey, delegatee_pubkey).len();

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
        let delegation_id = match get_delegation_id(
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

        let mut delegation = get_delegation(deps.storage, &delegation_id).unwrap();

        // Delegation requested and strings already provided
        if delegation.delegation_string.is_some() {
            return generic_err!("Delegation strings already provided");
        }

        delegation.delegation_string = Some(proxy_delegation.delegation_string.clone());
        set_delegation(deps.storage, &delegation_id, &delegation);
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
    let data_entry: DataEntry = match get_data_entry(deps.storage, data_id) {
        None => generic_err!("Data entry doesn't exist."),
        Some(data_entry) => Ok(data_entry),
    }?;

    ensure_delegator(deps.storage, &data_entry.delegator_pubkey, &info.sender)?;

    let mut state = get_state(deps.storage)?;

    // Get selected proxies for current delegation
    let proxy_pubkeys = get_all_proxies_from_delegation(
        deps.storage,
        &data_entry.delegator_pubkey,
        delegatee_pubkey,
    );

    if proxy_pubkeys.is_empty() {
        return generic_err!("Delegation doesn't exist.");
    }

    // Ensure more than minimum_request_reward_amount * number_of_proxies of stake provided
    ensure_stake(
        &state,
        &info.funds,
        &(state.minimum_request_reward_amount.u128() * proxy_pubkeys.len() as u128),
    )?;

    let mut new_request = ReencryptionRequest {
        delegatee_pubkey: delegatee_pubkey.to_string(),
        data_id: data_id.to_string(),
        fragment: None,
        proxy_pubkey: "".to_string(),
        delegation_string: "".to_string(),
        // Per proxy stake amount
        reward_amount: Uint128::new(info.funds[0].amount.u128() / proxy_pubkeys.len() as u128),
        delegator_addr: info.sender,
    };

    for proxy_pubkey in proxy_pubkeys {
        let delegation_id = get_delegation_id(
            deps.storage,
            &data_entry.delegator_pubkey,
            delegatee_pubkey,
            &proxy_pubkey,
        )
        .unwrap();
        let delegation = get_delegation(deps.storage, &delegation_id).unwrap();

        // Can happen only when you request re-encryption before providing delegation strings for proxies
        let delegation_string: String = match delegation.delegation_string {
            None => generic_err!("Not all delegation strings provided"),
            Some(delegation_string) => Ok(delegation_string),
        }?;

        if get_delegatee_reencryption_request(
            deps.storage,
            data_id,
            delegatee_pubkey,
            &proxy_pubkey,
        )
        .is_some()
        {
            return generic_err!("Reencryption already requested");
        }

        // Add reencryption task for each proxy
        new_request.proxy_pubkey = proxy_pubkey.clone();
        new_request.delegation_string = delegation_string;
        let request_id = state.next_request_id;
        set_reencryption_request(deps.storage, &request_id, &new_request);
        add_delegatee_reencryption_request(
            deps.storage,
            data_id,
            delegatee_pubkey,
            &proxy_pubkey,
            &request_id,
        );
        add_proxy_reencryption_request(deps.storage, &proxy_pubkey, &request_id);
        state.next_request_id += 1;
    }

    set_state(deps.storage, &state)?;

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
    let requests = get_all_proxy_reencryption_requests(store, proxy_pubkey);

    if requests.is_empty() {
        return Ok(None);
    }
    // Request must exist
    let request = get_reencryption_request(store, &requests[0]).unwrap();

    let data_entry = get_data_entry(store, &request.data_id).unwrap();

    let proxy_task = ProxyTask {
        data_id: request.data_id.clone(),
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

        // Delegator actions
        ExecuteMsg::AddData {
            data_id,
            delegator_pubkey,
        } => try_add_data(deps, env, info, &data_id, &delegator_pubkey),
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
            proxy_pubkeys: get_all_active_proxy_pubkeys(deps.storage),
        })?),
        QueryMsg::GetDataID { data_id } => Ok(to_binary(&GetDataIDResponse {
            data_entry: get_data_entry(deps.storage, &data_id),
        })?),
        QueryMsg::GetFragments {
            data_id,
            delegatee_pubkey,
        } => {
            let state = get_state(deps.storage)?;
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
            let state = get_state(deps.storage)?;
            Ok(to_binary(&GetContractStateResponse {
                admin: state.admin,
                threshold: state.threshold,
                n_max_proxies: state.n_max_proxies,
                stake_denom: state.stake_denom,
                minimum_proxy_stake_amount: state.minimum_proxy_stake_amount,
                minimum_request_reward_amount: state.minimum_request_reward_amount,
            })?)
        }

        QueryMsg::GetNextProxyTask { proxy_pubkey } => Ok(to_binary(&GetNextProxyTaskResponse {
            proxy_task: get_next_proxy_task(deps.storage, &proxy_pubkey)?,
        })?),

        QueryMsg::GetDelegationState {
            delegator_pubkey,
            delegatee_pubkey,
        } => {
            let state = get_state(deps.storage)?;
            let n_proxies =
                get_all_proxies_from_delegation(deps.storage, &delegator_pubkey, &delegatee_pubkey)
                    .len();
            let minimum_stake_amount =
                n_proxies as u128 * state.minimum_request_reward_amount.u128();
            Ok(to_binary(&GetDelegationStateResponse {
                delegation_state: get_delegation_state(
                    deps.storage,
                    &delegator_pubkey,
                    &delegatee_pubkey,
                ),
                minimum_request_reward: Coin {
                    denom: state.stake_denom,
                    amount: Uint128::new(minimum_stake_amount),
                },
            })?)
        }

        QueryMsg::GetSelectedProxiesForDelegation {
            delegator_pubkey,
            delegatee_pubkey,
        } => Ok(to_binary(&GetSelectedProxiesForDelegationResponse {
            proxy_pubkeys: get_all_proxies_from_delegation(
                deps.storage,
                &delegator_pubkey,
                &delegatee_pubkey,
            ),
        })?),

        QueryMsg::GetProxyInfo { proxy_pubkey } => {
            let mut proxy_info: Option<ProxyInfo> = None;

            if let Some(proxy_addr) = get_proxy_address(deps.storage, &proxy_pubkey) {
                let proxy = get_proxy(deps.storage, &proxy_addr).unwrap();
                let state = get_state(deps.storage)?;

                proxy_info = Some(ProxyInfo {
                    proxy_address: proxy_addr,
                    stake_amount: proxy.stake_amount,
                    withdrawable_stake_amount: Uint128::new(maximum_withdrawable_stake_amount(
                        &state, &proxy,
                    )),
                    proxy_state: proxy.state,
                })
            }

            Ok(to_binary(&GetProxyInfoResponse { proxy_info })?)
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
    if let Some(correct_delegator_addr) = get_delegator_address(storage, delegator_pubkey) {
        // Check if delegator_pubkey is registered with delegator_address

        if &correct_delegator_addr != delegator_address {
            return generic_err!(format!(
                "Delegator {} already registered with this pubkey.",
                correct_delegator_addr
            ));
        }
    } else {
        // Reserve delegator_pubkey for current delegator_address
        set_delegator_address(storage, delegator_pubkey, delegator_address);
    }
    Ok(())
}

fn ensure_stake(state: &State, funds: &[Coin], required_stake: &u128) -> StdResult<()> {
    if funds.len() != 1 || funds[0].denom != state.stake_denom {
        return generic_err!(format!("Expected 1 Coin with denom {}", state.stake_denom));
    }

    if &funds[0].amount.u128() < required_stake {
        return generic_err!(format!(
            "Requires at least {} {}.",
            required_stake, state.stake_denom
        ));
    }
    Ok(())
}

fn select_proxy_pubkeys(store: &dyn Storage) -> StdResult<Vec<String>> {
    let state: State = get_state(store)?;
    let proxy_pubkeys = get_all_active_proxy_pubkeys(store);

    // Select n_max_proxies or maximum possible
    let n_proxies = std::cmp::min(state.n_max_proxies as usize, proxy_pubkeys.len());

    if n_proxies < state.threshold as usize {
        return generic_err!("Less proxies than threshold registered");
    }

    Ok(proxy_pubkeys[0..n_proxies].to_vec())
}
