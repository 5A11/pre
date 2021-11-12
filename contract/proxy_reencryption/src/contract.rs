use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, GetAvailableProxiesResponse, ProxyDelegation, GetDataIDResponse, GetFragmentsResponse, GetThresholdResponse, GetNextProxyTaskResponse, GetDoesDelegationExistRepsonse};
use crate::state::{get_state, set_state, State, get_is_proxy, set_is_proxy, set_proxy_availability, remove_proxy_availability, DataEntry, set_data_entry, set_delegation_string, get_proxy_availability, get_all_proxies_from_delegation, add_reencryption_request, get_delegation_string, set_fragment, get_data_entry, get_all_fragments, ProxyTask, get_all_reencryption_requests, remove_reencryption_request, is_reencryption_request, ReencryptionRequest, get_fragment, HashID, get_all_available_proxy_pubkeys, get_available_proxy_pubkeys, increase_available_proxy_pubkeys, decrease_available_proxy_pubkeys};

use cosmwasm_std::{
    StdError, attr, to_binary, Binary, Addr, Env, Response,
    StdResult, DepsMut, Deps, MessageInfo, Storage,
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
    };
    set_state(deps.storage, &state)?;

    if let Some(proxies) = msg.proxies {
        for proxy in proxies
        {
            set_is_proxy(deps.storage, &proxy, true)?;
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

    if get_is_proxy(deps.storage, proxy)?
    {
        return generic_err!(format!("{} is already proxy",proxy));
    }

    set_is_proxy(deps.storage, &proxy, true)?;

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

    if !get_is_proxy(deps.storage, proxy)?
    {
        return generic_err!(format!("{} is not a proxy",proxy));
    }

    set_is_proxy(deps.storage, &proxy, false)?;

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
    proxy_pubkey: Binary,
) -> StdResult<Response> {
    ensure_proxy(deps.storage, &info.sender)?;

    if !get_proxy_availability(deps.storage, &info.sender)?.is_none()
    {
        return generic_err!("Proxy already registered");
    }

    set_proxy_availability(deps.storage, &info.sender, &proxy_pubkey)?;
    increase_available_proxy_pubkeys(deps.storage, &proxy_pubkey)?;

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

    let proxy_pubkey = match get_proxy_availability(deps.storage, &info.sender)?
    {
        Some(pubkey) => Ok(pubkey),
        None => generic_err!("Proxy not registered")
    }?;

    decrease_available_proxy_pubkeys(deps.storage, &proxy_pubkey)?;
    remove_proxy_availability(deps.storage, &info.sender)?;

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
    delegatee_pubkey: &Binary,
    fragment: &HashID,
) -> StdResult<Response> {
    let request = ReencryptionRequest
    {
        data_id: data_id.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
    };

    let proxy_pubkey = match get_proxy_availability(deps.storage, &info.sender)?
    {
        Some(pubkey) => Ok(pubkey),
        None => generic_err!("Proxy not registered")
    }?;

    // Check if reencryption was requested
    if !is_reencryption_request(deps.storage, &proxy_pubkey, &request)?
    {
        return generic_err!("This fragment was not requested.");
    }

    // Add fragment to fragments store
    set_fragment(deps.storage, &data_id, &delegatee_pubkey, &proxy_pubkey, &fragment)?;

    // Remove request as it's completed
    remove_reencryption_request(deps.storage, &proxy_pubkey, &request).unwrap();

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
    return Ok(res);
}


// Delegator actions

fn try_add_data(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_id: &HashID,
    delegator_pubkey: &Binary,
) -> StdResult<Response> {
    if get_data_entry(deps.storage, &data_id)?.is_some()
    {
        return generic_err!(format!("Entry with ID {} already exist.",data_id));
    }

    let entry = DataEntry { delegator_pubkey: delegator_pubkey.clone(), delegator_addr: info.sender.clone() };
    set_data_entry(deps.storage, &data_id, &entry)?;

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
            attr("delegator_pubkey", delegator_pubkey.to_base64()),
        ],
        data: None,
    };
    return Ok(res);
}


fn try_add_delegation(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    delegator_pubkey: &Binary,
    delegatee_pubkey: &Binary,
    proxy_delegations: &Vec<ProxyDelegation>,
    // n_num_proxies: Option<u32> = threshold to max / maximum possible on None
    // Implement selection
) -> StdResult<Response> {
    let state = get_state(deps.storage)?;

    if proxy_delegations.len() < state.threshold as usize || proxy_delegations.len() > state.n_max_proxies as usize
    {
        let n_max_proxies_str: String =
            if state.n_max_proxies == u32::MAX
            { "unrestricted".to_string() } else { state.n_max_proxies.to_string() };
        return generic_err!(format!("Number of Delegations needs to be between {} and {}.",state.threshold, n_max_proxies_str));
    }

    for proxy_delegation in proxy_delegations
    {
        if get_available_proxy_pubkeys(deps.storage, &proxy_delegation.proxy_pubkey)? == 0
        {
            return generic_err!(format!("Proxy {} is not registered.",proxy_delegation.proxy_pubkey));
        }

        if get_delegation_string(deps.storage, &info.sender, &delegator_pubkey, &delegatee_pubkey, &proxy_delegation.proxy_pubkey)?.is_some()
        {
            return generic_err!(format!("Delegation already added for proxy {}.",proxy_delegation.proxy_pubkey));
        }

        set_delegation_string(deps.storage, &info.sender, &delegator_pubkey, &delegatee_pubkey, &proxy_delegation.proxy_pubkey, &proxy_delegation.delegation_string)?;
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
    delegatee_pubkey: &Binary,
) -> StdResult<Response> {

    // Only data owner can request reencryption
    ensure_data_owner(deps.storage, &data_id, &info.sender)?;

    let data_entry: DataEntry = get_data_entry(deps.storage, &data_id)?.unwrap();


    // Get selected proxies for current delegation
    let proxies = get_all_proxies_from_delegation(deps.storage, &info.sender, &data_entry.delegator_pubkey, &delegatee_pubkey)?;

    if proxies.len() == 0
    {
        return generic_err!("Delegation doesn't exist.");
    }

    for proxy_pubkey in proxies
    {
        let reencryption_request = ReencryptionRequest { data_id: data_id.clone(), delegatee_pubkey: delegatee_pubkey.clone() };
        if get_fragment(deps.storage, &data_id, &delegatee_pubkey, &proxy_pubkey)?.is_some() ||
            is_reencryption_request(deps.storage, &proxy_pubkey, &reencryption_request)?
        {
            return generic_err!("Reencryption already requested");
        }

        // Add reencryption task for each proxy
        add_reencryption_request(deps.storage, &proxy_pubkey, &reencryption_request)?;
    }

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


pub fn get_next_proxy_task(store: &dyn Storage, proxy_pubkey: &Binary) -> StdResult<Option<ProxyTask>> {
    let requests = get_all_reencryption_requests(store, proxy_pubkey)?;

    if requests.len() == 0
    {
        return Ok(None);
    }
    let request = &requests[0];

    let data_entry = get_data_entry(store, &request.data_id)?.unwrap();

    let delegation_string = get_delegation_string(store, &data_entry.delegator_addr, &data_entry.delegator_pubkey, &request.delegatee_pubkey, &proxy_pubkey)?.unwrap();

    let proxy_task = ProxyTask {
        data_id: request.data_id.clone(),
        delegatee_pubkey: request.delegatee_pubkey.clone(),
        delegator_pubkey: data_entry.delegator_pubkey.clone(),
        delegation_string: delegation_string.clone(),
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
        ExecuteMsg::RegisterProxy {proxy_pubkey} => try_register_proxy(deps, env, info, proxy_pubkey),
        ExecuteMsg::UnregisterProxy {} => try_unregister_proxy(deps, env, info),
        ExecuteMsg::AddData { data_id, delegator_pubkey } => try_add_data(deps, env, info, &data_id, &delegator_pubkey),
        ExecuteMsg::AddDelegation { delegator_pubkey, delegatee_pubkey, proxy_delegations } => try_add_delegation(deps, env, info, &delegator_pubkey, &delegatee_pubkey, &proxy_delegations),
        ExecuteMsg::RequestReencryption { data_id, delegatee_pubkey } => try_request_reencryption(deps, env, info, &data_id, &delegatee_pubkey),
        ExecuteMsg::ProvideReencryptedFragment { data_id, delegatee_pubkey, fragment } => try_provide_reencrypted_fragment(deps, env, info, &data_id, &delegatee_pubkey, &fragment),
    }
}


pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAvailableProxies {} => {
            Ok(to_binary(&GetAvailableProxiesResponse {
                proxy_pubkeys: get_all_available_proxy_pubkeys(deps.storage)?,
            })?)
        }
        QueryMsg::GetDataID { data_id } => {
            Ok(to_binary(&GetDataIDResponse {
                data_entry: get_data_entry(deps.storage, &data_id)?,
            })?)
        }
        QueryMsg::GetFragments { data_id, delegatee_pubkey } => {
            let state = get_state(deps.storage)?;
            Ok(to_binary(&GetFragmentsResponse {
                fragments: get_all_fragments(deps.storage, &data_id, &delegatee_pubkey)?,
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

        QueryMsg::GetDoesDelegationExist { delegator_addr, delegator_pubkey,  delegatee_pubkey } => {
            let delegations = get_all_proxies_from_delegation(deps.storage, &delegator_addr, &delegator_pubkey, &delegatee_pubkey)?;

            let delegation_exists: bool = if delegations.len() > 0 { true } else { false };

            Ok(to_binary(&GetDoesDelegationExistRepsonse {
                delegation_exists,
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
    if !get_is_proxy(store, addr)?
    {
        return generic_err!("Only proxy can execute this method.");
    }
    Ok(())
}


fn ensure_data_owner(store: &dyn Storage, data_id: &HashID, addr: &Addr) -> StdResult<()>
{
    match get_data_entry(store, &data_id)?
    {
        None => generic_err!(format!("DataID {} doesn't exist.",data_id)),
        Some(data_entry) => if &data_entry.delegator_addr != addr { generic_err!(format!("Only data owner {} can execute.",data_entry.delegator_addr)) } else { Ok(()) }
    }
}
