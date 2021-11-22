use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, GetAvailableProxiesResponse, ProxyDelegation, GetDataIDResponse, GetFragmentsResponse, GetThresholdResponse, GetNextProxyTaskResponse, GetDoesDelegationExistRepsonse, GetSelectedProxiesForDelegationResponse, ProxyTask};
use crate::state::{get_state, set_state, State, get_is_proxy, set_is_proxy, set_proxy_availability, remove_proxy_availability, DataEntry, set_data_entry, set_delegation_string, get_proxy_availability, get_all_proxies_from_delegation, set_reencryption_request, get_delegation_string, get_data_entry, HashID, get_all_available_proxy_pubkeys, increase_available_proxy_pubkeys, decrease_available_proxy_pubkeys, is_delegation_empty, get_delegatee_reencryption_request, remove_proxy_reencryption_request, get_reencryption_request, ReencryptionRequest, add_delegatee_reencryption_request, add_proxy_reencryption_request, get_all_proxy_reencryption_requests, get_all_delegatee_reencryption_requests};

use cosmwasm_std::{
    StdError, attr, to_binary, Addr, Env, Response,
    StdResult, DepsMut, Deps, MessageInfo, Storage, Binary,
};

macro_rules! generic_err {
    ($val:expr) => {
        Err(StdError::generic_err($val))
    };
}

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
    };

    if state.threshold == 0
    {
        return generic_err!("Threshold cannot be 0");
    }

    if state.n_max_proxies < state.threshold
    {
        return generic_err!("Value of n_max_proxies cannot be lower than threshold.");
    }

    set_state(deps.storage, &state)?;

    if let Some(proxies) = msg.proxies {
        for proxy in proxies
        {
            set_is_proxy(deps.storage, &proxy, true);
        }
    };

    Ok(Response::default())
}

// Admin actions

fn try_add_proxy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proxy: &Addr,
) -> StdResult<Response> {
    let state: State = get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    if get_is_proxy(deps.storage, proxy)
    {
        return generic_err!(format!("{} is already proxy",proxy));
    }

    set_is_proxy(deps.storage, &proxy, true);

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "add_proxy"),
            attr(
                "admin",
                info.sender.as_str(),
            ),
            attr("proxy", proxy.as_str()),
        ],
        data: None,
    };
    return Ok(res);
}

fn try_remove_proxy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proxy: &Addr,
) -> StdResult<Response> {
    let state: State = get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    if !get_is_proxy(deps.storage, proxy)
    {
        return generic_err!(format!("{} is not a proxy",proxy));
    }

    set_is_proxy(deps.storage, &proxy, false);

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "remove_proxy"),
            attr(
                "admin",
                info.sender.as_str(),
            ),
            attr("proxy", proxy.as_str()),
        ],
        data: None,
    };
    return Ok(res);
}

// Proxy actions

fn try_register_proxy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proxy_pubkey: String,
) -> StdResult<Response> {
    ensure_proxy(deps.storage, &info.sender)?;

    if get_proxy_availability(deps.storage, &info.sender).is_some()
    {
        return generic_err!("Proxy already registered");
    }

    set_proxy_availability(deps.storage, &info.sender, &proxy_pubkey);
    increase_available_proxy_pubkeys(deps.storage, &proxy_pubkey);

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "register_proxy"),
            attr(
                "proxy",
                info.sender.as_str(),
            ),
        ],
        data: None,
    };
    return Ok(res);
}


fn try_unregister_proxy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    ensure_proxy(deps.storage, &info.sender)?;

    let proxy_pubkey = match get_proxy_availability(deps.storage, &info.sender)
    {
        Some(pubkey) => Ok(pubkey),
        None => generic_err!("Proxy not registered")
    }?;

    decrease_available_proxy_pubkeys(deps.storage, &proxy_pubkey);
    remove_proxy_availability(deps.storage, &info.sender);

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "unregister_proxy"),
            attr(
                "proxy",
                info.sender.as_str(),
            ),
        ],
        data: None,
    };
    return Ok(res);
}


fn try_provide_reencrypted_fragment(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &HashID,
    delegatee_pubkey: &String,
    fragment: &HashID,
) -> StdResult<Response> {
    let proxy_pubkey = match get_proxy_availability(deps.storage, &info.sender)
    {
        Some(pubkey) => Ok(pubkey),
        None => generic_err!("Proxy not registered")
    }?;

    let request_id: u64 = match get_delegatee_reencryption_request(deps.storage, &data_id, delegatee_pubkey, &proxy_pubkey)
    {
        None => generic_err!("This fragment was not requested."),
        Some(request_id) => Ok(request_id)
    }?;

    // Request must exist - panic otherwise
    let mut request = get_reencryption_request(deps.storage, &request_id).unwrap();

    if request.fragment.is_some()
    {
        return generic_err!(format!("Fragment already provided by {}.", request.proxy_address.unwrap()));
    }

    // Add fragment to fragments store
    request.fragment = Some(fragment.clone());
    request.proxy_address = Some(info.sender.clone());
    set_reencryption_request(deps.storage, &request_id, &request);

    // Remove request as it's completed
    remove_proxy_reencryption_request(deps.storage, &proxy_pubkey, &request_id);

    // Return response
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
    Ok(res)
}

// Delegator actions

fn try_add_data(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &HashID,
    delegator_pubkey: &String,
) -> StdResult<Response> {
    if get_data_entry(deps.storage, &data_id).is_some()
    {
        return generic_err!(format!("Entry with ID {} already exist.",data_id));
    }

    let entry = DataEntry { delegator_pubkey: delegator_pubkey.clone(), delegator_addr: info.sender.clone() };
    set_data_entry(deps.storage, &data_id, &entry);

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "add_data"),
            attr(
                "owner",
                info.sender.as_str(),
            ),
            attr("data_id", data_id),
            attr("delegator_pubkey", delegator_pubkey),
        ],
        data: None,
    };
    return Ok(res);
}

fn try_request_proxies_for_delegation(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    delegator_pubkey: &String,
    delegatee_pubkey: &String,
) -> StdResult<Response>
{
    if !is_delegation_empty(deps.storage, &info.sender, &delegator_pubkey, &delegatee_pubkey)
    {
        return generic_err!("Delegation already exist");
    }

    let selected_proxy_pubkeys = select_proxy_pubkeys(deps.storage)?;
    let mut selected_proxy_pubkeys_str: String = String::from("[");

    for proxy_pubkey in selected_proxy_pubkeys
    {
        set_delegation_string(deps.storage, &info.sender, &delegator_pubkey, &delegatee_pubkey, &proxy_pubkey, &None);
        selected_proxy_pubkeys_str += format!("\"{}\", ", proxy_pubkey).as_str();
    }
    selected_proxy_pubkeys_str += "]";

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
    return Ok(res);
}

fn try_add_delegation(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    delegator_pubkey: &String,
    delegatee_pubkey: &String,
    proxy_delegations: &Vec<ProxyDelegation>,
) -> StdResult<Response> {
    let n_expected_strings = get_all_proxies_from_delegation(deps.storage, &info.sender, &delegator_pubkey, &delegatee_pubkey).len();
    let n_provided_strings = proxy_delegations.len();
    if n_expected_strings != n_provided_strings
    {
        return generic_err!(format!("Provided wrong number of delegation strings, expected {} got {}.",n_expected_strings, n_provided_strings));
    }

    for proxy_delegation in proxy_delegations
    {
        let optional_delegation_string = get_delegation_string(deps.storage, &info.sender, &delegator_pubkey, &delegatee_pubkey, &proxy_delegation.proxy_pubkey);
        match optional_delegation_string
        {
            // Delegation not requested
            None => { return generic_err!(format!("Proxy {} not selected for delegation.",proxy_delegation.proxy_pubkey)); }
            // Delegation requested
            Some(delegation_string) =>
                {
                    match delegation_string
                    {
                        // Delegation requested and strings already provided
                        Some(_) => { return generic_err!("Delegation strings already provided"); }
                        // Delegation requested and strings not provided - correct case
                        None => { set_delegation_string(deps.storage, &info.sender, &delegator_pubkey, &delegatee_pubkey, &proxy_delegation.proxy_pubkey, &Some(proxy_delegation.delegation_string.clone())); }
                    }
                }
        }
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
    return Ok(res);
}

fn try_request_reencryption(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &HashID,
    delegatee_pubkey: &String,
) -> StdResult<Response> {

    // Only data owner can request reencryption
    ensure_data_owner(deps.storage, &data_id, &info.sender)?;

    let data_entry: DataEntry = get_data_entry(deps.storage, &data_id).unwrap();

    let mut state = get_state(deps.storage)?;

    // Get selected proxies for current delegation
    let proxy_pubkeys = get_all_proxies_from_delegation(deps.storage, &info.sender, &data_entry.delegator_pubkey, &delegatee_pubkey);

    if proxy_pubkeys.len() == 0
    {
        return generic_err!("Delegation doesn't exist.");
    }

    let new_request = ReencryptionRequest
    {
        delegatee_pubkey: delegatee_pubkey.clone(),
        data_id: data_id.clone(),
        fragment: None,
        proxy_address: None,
    };

    for proxy_pubkey in proxy_pubkeys
    {
        // Can happen only when you request re-encryption before providing delegation strings for proxies
        if get_delegation_string(deps.storage, &data_entry.delegator_addr, &data_entry.delegator_pubkey, &delegatee_pubkey, &proxy_pubkey).unwrap().is_none()
        {
            return generic_err!("Not all delegation strings provided");
        }

        if get_delegatee_reencryption_request(deps.storage, &data_id, &delegatee_pubkey, &proxy_pubkey).is_some()
        {
            return generic_err!("Reencryption already requested");
        }

        // Add reencryption task for each proxy
        let request_id = state.next_request_id;
        set_reencryption_request(deps.storage, &request_id, &new_request);
        add_delegatee_reencryption_request(deps.storage, &data_id, &delegatee_pubkey, &proxy_pubkey, &request_id);
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
    return Ok(res);
}

pub fn get_next_proxy_task(store: &dyn Storage, proxy_pubkey: &String) -> StdResult<Option<ProxyTask>> {
    let requests = get_all_proxy_reencryption_requests(store, proxy_pubkey);

    if requests.len() == 0
    {
        return Ok(None);
    }
    // Request must exist
    let request = get_reencryption_request(store, &requests[0]).unwrap();

    let data_entry = get_data_entry(store, &request.data_id).unwrap();

    let delegation_string = get_delegation_string(store, &data_entry.delegator_addr, &data_entry.delegator_pubkey, &request.delegatee_pubkey, &proxy_pubkey).unwrap();

    let proxy_task = ProxyTask {
        data_id: request.data_id.clone(),
        delegatee_pubkey: request.delegatee_pubkey.clone(),
        delegator_pubkey: data_entry.delegator_pubkey.clone(),
        delegation_string: delegation_string.unwrap().clone(),
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
        ExecuteMsg::AddProxy { proxy_addr } => try_add_proxy(deps, env, info, &proxy_addr),
        ExecuteMsg::RemoveProxy { proxy_addr } => try_remove_proxy(deps, env, info, &proxy_addr),
        ExecuteMsg::RegisterProxy { proxy_pubkey } => try_register_proxy(deps, env, info, proxy_pubkey),
        ExecuteMsg::UnregisterProxy {} => try_unregister_proxy(deps, env, info),
        ExecuteMsg::AddData { data_id, delegator_pubkey } => try_add_data(deps, env, info, &data_id, &delegator_pubkey),
        ExecuteMsg::AddDelegation { delegator_pubkey, delegatee_pubkey, proxy_delegations } => try_add_delegation(deps, env, info, &delegator_pubkey, &delegatee_pubkey, &proxy_delegations),
        ExecuteMsg::RequestProxiesForDelegation { delegator_pubkey, delegatee_pubkey } => try_request_proxies_for_delegation(deps, env, info, &delegator_pubkey, &delegatee_pubkey),
        ExecuteMsg::RequestReencryption { data_id, delegatee_pubkey } => try_request_reencryption(deps, env, info, &data_id, &delegatee_pubkey),
        ExecuteMsg::ProvideReencryptedFragment { data_id, delegatee_pubkey, fragment } => try_provide_reencrypted_fragment(deps, env, info, &data_id, &delegatee_pubkey, &fragment),
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAvailableProxies {} => {
            Ok(to_binary(&GetAvailableProxiesResponse {
                proxy_pubkeys: get_all_available_proxy_pubkeys(deps.storage),
            })?)
        }
        QueryMsg::GetDataID { data_id } => {
            Ok(to_binary(&GetDataIDResponse {
                data_entry: get_data_entry(deps.storage, &data_id),
            })?)
        }
        QueryMsg::GetFragments { data_id, delegatee_pubkey } => {
            let state = get_state(deps.storage)?;
            Ok(to_binary(&GetFragmentsResponse {
                fragments: get_all_fragments(deps.storage, &data_id, &delegatee_pubkey),
                threshold: state.threshold,
            })?)
        }
        QueryMsg::GetThreshold {} => {
            Ok(to_binary(&GetThresholdResponse {
                threshold: get_state(deps.storage)?.threshold,
            })?)
        }

        QueryMsg::GetNextProxyTask { proxy_pubkey } => {
            Ok(to_binary(&GetNextProxyTaskResponse {
                proxy_task: get_next_proxy_task(deps.storage, &proxy_pubkey)?,
            })?)
        }

        QueryMsg::GetDoesDelegationExist { delegator_addr, delegator_pubkey, delegatee_pubkey } => {
            let delegations = get_all_proxies_from_delegation(deps.storage, &delegator_addr, &delegator_pubkey, &delegatee_pubkey);

            let delegation_exists: bool = if delegations.len() > 0 { true } else { false };

            Ok(to_binary(&GetDoesDelegationExistRepsonse {
                delegation_exists,
            })?)
        }

        QueryMsg::GetSelectedProxiesForDelegation { delegator_addr, delegator_pubkey, delegatee_pubkey } => {
            Ok(to_binary(&GetSelectedProxiesForDelegationResponse {
                proxy_pubkeys: get_all_proxies_from_delegation(deps.storage, &delegator_addr, &delegator_pubkey, &delegatee_pubkey),
            })?)
        }
    }
}

// Private functions

fn ensure_admin(state: &State, addr: &Addr) -> StdResult<()>
{
    if addr != &state.admin
    {
        return generic_err!("Only admin can execute this method.");
    }
    Ok(())
}

fn ensure_proxy(store: &dyn Storage, addr: &Addr) -> StdResult<()>
{
    if !get_is_proxy(store, addr)
    {
        return generic_err!("Only proxy can execute this method.");
    }
    Ok(())
}

fn ensure_data_owner(store: &dyn Storage, data_id: &HashID, addr: &Addr) -> StdResult<()>
{
    match get_data_entry(store, &data_id)
    {
        None => generic_err!(format!("DataID {} doesn't exist.",data_id)),
        Some(data_entry) => if &data_entry.delegator_addr != addr { generic_err!(format!("Only data owner {} can execute.",data_entry.delegator_addr)) } else { Ok(()) }
    }
}

fn select_proxy_pubkeys(store: &dyn Storage) -> StdResult<Vec<String>>
{
    let state: State = get_state(store)?;
    let proxy_pubkeys = get_all_available_proxy_pubkeys(store);

    // Select n_max_proxies or maximum possible
    let n_proxies = std::cmp::min(state.n_max_proxies as usize, proxy_pubkeys.len());

    if n_proxies < state.threshold as usize
    {
        return generic_err!("Less proxies than threshold registered");
    }

    return Ok(proxy_pubkeys[0..n_proxies].to_vec());
}

pub fn get_all_fragments(storage: &dyn Storage, data_id: &HashID, delegatee_pubkey: &String) -> Vec<HashID> {
    let mut fragments: Vec<HashID> = Vec::new();
    for request_id in get_all_delegatee_reencryption_requests(storage, &data_id, &delegatee_pubkey)
    {
        let request = get_reencryption_request(storage, &request_id).unwrap();

        match request.fragment
        {
            None => continue,
            Some(fragment) => fragments.push(fragment.clone())
        }
    }
    return fragments;
}
