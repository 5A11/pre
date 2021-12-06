use crate::msg::{
    ExecuteMsg, GetAvailableProxiesResponse, GetDataIDResponse, GetDoesDelegationExistRepsonse,
    GetFragmentsResponse, GetNextProxyTaskResponse, GetSelectedProxiesForDelegationResponse,
    GetThresholdResponse, InstantiateMsg, ProxyDelegation, ProxyTask, QueryMsg,
};
use crate::state::{
    get_all_available_proxy_pubkeys, get_data_entry, get_is_proxy, get_proxy_address,
    get_proxy_pubkey, get_state, remove_proxy_address, remove_proxy_pubkey, set_data_entry,
    set_is_proxy, set_proxy_address, set_proxy_pubkey, set_state, DataEntry, State,
};

use crate::delegations::{
    add_proxy_delegation, get_all_proxies_from_delegation, get_all_proxy_delegations,
    get_delegation, get_delegation_id, is_delegation_empty, remove_delegation,
    remove_delegation_id, remove_proxy_delegation, set_delegation, set_delegation_id, Delegation,
};
use crate::reencryption_requests::{
    add_delegatee_reencryption_request, add_proxy_reencryption_request,
    get_all_delegatee_reencryption_requests, get_all_proxy_reencryption_requests,
    get_delegatee_reencryption_request, get_reencryption_request,
    remove_delegatee_reencryption_request, remove_proxy_reencryption_request,
    remove_reencryption_request, set_reencryption_request, ReencryptionRequest,
};
use cosmwasm_std::{
    attr, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Storage,
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
        next_delegation_id: 0,
    };

    if state.threshold == 0 {
        return generic_err!("Threshold cannot be 0");
    }

    if state.n_max_proxies < state.threshold {
        return generic_err!("Value of n_max_proxies cannot be lower than threshold.");
    }

    set_state(deps.storage, &state)?;

    if let Some(proxies) = msg.proxies {
        for proxy in proxies {
            set_is_proxy(deps.storage, &proxy, true);
        }
    };

    Ok(Response::default())
}

// Admin actions

fn try_add_proxy(deps: DepsMut, _env: Env, info: MessageInfo, proxy: &Addr) -> StdResult<Response> {
    let state: State = get_state(deps.storage)?;

    ensure_admin(&state, &info.sender)?;

    if get_is_proxy(deps.storage, proxy) {
        return generic_err!(format!("{} is already proxy", proxy));
    }

    set_is_proxy(deps.storage, proxy, true);

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "add_proxy"),
            attr("admin", info.sender.as_str()),
            attr("proxy", proxy.as_str()),
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

    if !get_is_proxy(deps.storage, proxy_addr) {
        return generic_err!(format!("{} is not a proxy", proxy_addr));
    }

    set_is_proxy(deps.storage, proxy_addr, false);

    // Unregister proxy if registered
    if let Some(proxy_pubkey) = get_proxy_pubkey(deps.storage, &info.sender) {
        unregister_proxy(deps, proxy_addr, &proxy_pubkey)?;
    };

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "remove_proxy"),
            attr("admin", info.sender.as_str()),
            attr("proxy_addr", proxy_addr.as_str()),
        ],
        data: None,
    };
    Ok(res)
}

// Proxy actions

fn try_register_proxy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proxy_pubkey: String,
) -> StdResult<Response> {
    ensure_proxy(deps.storage, &info.sender)?;

    if get_proxy_pubkey(deps.storage, &info.sender).is_some() {
        return generic_err!("Proxy already registered.");
    }

    if get_proxy_address(deps.storage, &proxy_pubkey).is_some() {
        return generic_err!("Pubkey already used.");
    }

    set_proxy_address(deps.storage, &proxy_pubkey, &info.sender);
    set_proxy_pubkey(deps.storage, &info.sender, &proxy_pubkey);

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
    ensure_proxy(deps.storage, &info.sender)?;

    let proxy_pubkey = match get_proxy_pubkey(deps.storage, &info.sender) {
        Some(pubkey) => Ok(pubkey),
        None => generic_err!("Proxy not registered"),
    }?;

    unregister_proxy(deps, &info.sender, &proxy_pubkey)?;

    // Return response
    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "unregister_proxy"),
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
    let proxy_pubkey = match get_proxy_pubkey(deps.storage, &info.sender) {
        Some(pubkey) => Ok(pubkey),
        None => generic_err!("Proxy not registered"),
    }?;

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

    // Add fragment to fragments store
    request.fragment = Some(fragment.to_string());
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
    data_id: &str,
    delegator_pubkey: &str,
) -> StdResult<Response> {
    if get_data_entry(deps.storage, data_id).is_some() {
        return generic_err!(format!("Entry with ID {} already exist.", data_id));
    }

    let entry = DataEntry {
        delegator_pubkey: delegator_pubkey.to_string(),
        delegator_addr: info.sender.clone(),
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

    if !is_delegation_empty(
        deps.storage,
        &info.sender,
        delegator_pubkey,
        delegatee_pubkey,
    ) {
        return generic_err!("Delegation already exist");
    }

    let selected_proxy_pubkeys = select_proxy_pubkeys(deps.storage)?;
    let mut selected_proxy_pubkeys_str: String = String::from("[");

    let delegation = Delegation {
        delegator_addr: info.sender.clone(),
        delegator_pubkey: delegator_pubkey.to_string(),
        delegatee_pubkey: delegatee_pubkey.to_string(),
        delegation_string: None,
    };

    for proxy_pubkey in selected_proxy_pubkeys {
        set_delegation(deps.storage, &state.next_delegation_id, &delegation);
        set_delegation_id(
            deps.storage,
            &info.sender,
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
    let n_expected_strings = get_all_proxies_from_delegation(
        deps.storage,
        &info.sender,
        delegator_pubkey,
        delegatee_pubkey,
    )
    .len();
    let n_provided_strings = proxy_delegations.len();

    if n_expected_strings == 0 {
        return generic_err!("No proxies selected for this delegation");
    }

    if n_expected_strings != n_provided_strings {
        return generic_err!(format!(
            "Provided wrong number of delegation strings, expected {} got {}.",
            n_expected_strings, n_provided_strings
        ));
    }

    for proxy_delegation in proxy_delegations {
        match get_delegation_id(
            deps.storage,
            &info.sender,
            delegator_pubkey,
            delegatee_pubkey,
            &proxy_delegation.proxy_pubkey,
        ) {
            // Delegation not requested
            None => {
                return generic_err!(format!(
                    "Proxy {} not selected for delegation.",
                    proxy_delegation.proxy_pubkey
                ));
            }
            // Delegation requested
            Some(delegation_id) => {
                let mut delegation = get_delegation(deps.storage, &delegation_id).unwrap();
                match delegation.delegation_string {
                    // Delegation requested and strings already provided
                    Some(_) => {
                        return generic_err!("Delegation strings already provided");
                    }
                    // Delegation requested and strings not provided - correct case
                    None => {
                        delegation.delegation_string =
                            Some(proxy_delegation.delegation_string.clone());
                        set_delegation(deps.storage, &delegation_id, &delegation);
                    }
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
    ensure_data_owner(deps.storage, data_id, &info.sender)?;

    let data_entry: DataEntry = get_data_entry(deps.storage, data_id).unwrap();

    let mut state = get_state(deps.storage)?;

    // Get selected proxies for current delegation
    let proxy_pubkeys = get_all_proxies_from_delegation(
        deps.storage,
        &info.sender,
        &data_entry.delegator_pubkey,
        delegatee_pubkey,
    );

    if proxy_pubkeys.is_empty() {
        return generic_err!("Delegation doesn't exist.");
    }

    for proxy_pubkey in proxy_pubkeys {
        let delegation_id = get_delegation_id(
            deps.storage,
            &data_entry.delegator_addr,
            &data_entry.delegator_pubkey,
            delegatee_pubkey,
            &proxy_pubkey,
        )
        .unwrap();
        let delegation = get_delegation(deps.storage, &delegation_id).unwrap();

        // Can happen only when you request re-encryption before providing delegation strings for proxies
        if delegation.delegation_string.is_none() {
            return generic_err!("Not all delegation strings provided");
        }

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
        let new_request = ReencryptionRequest {
            delegatee_pubkey: delegatee_pubkey.to_string(),
            data_id: data_id.to_string(),
            fragment: None,
            proxy_pubkey: proxy_pubkey.clone(),
        };

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

    let delegation_id = get_delegation_id(
        store,
        &data_entry.delegator_addr,
        &data_entry.delegator_pubkey,
        &request.delegatee_pubkey,
        proxy_pubkey,
    )
    .unwrap();
    let delegation = get_delegation(store, &delegation_id).unwrap();

    let proxy_task = ProxyTask {
        data_id: request.data_id.clone(),
        delegatee_pubkey: request.delegatee_pubkey,
        delegator_pubkey: data_entry.delegator_pubkey,
        delegation_string: delegation.delegation_string.unwrap(),
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
        ExecuteMsg::RegisterProxy { proxy_pubkey } => {
            try_register_proxy(deps, env, info, proxy_pubkey)
        }
        ExecuteMsg::UnregisterProxy {} => try_unregister_proxy(deps, env, info),
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
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAvailableProxies {} => Ok(to_binary(&GetAvailableProxiesResponse {
            proxy_pubkeys: get_all_available_proxy_pubkeys(deps.storage),
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
                fragments: get_all_fragments(deps.storage, &data_id, &delegatee_pubkey),
                threshold: state.threshold,
            })?)
        }
        QueryMsg::GetThreshold {} => Ok(to_binary(&GetThresholdResponse {
            threshold: get_state(deps.storage)?.threshold,
        })?),

        QueryMsg::GetNextProxyTask { proxy_pubkey } => Ok(to_binary(&GetNextProxyTaskResponse {
            proxy_task: get_next_proxy_task(deps.storage, &proxy_pubkey)?,
        })?),

        QueryMsg::GetDoesDelegationExist {
            delegator_addr,
            delegator_pubkey,
            delegatee_pubkey,
        } => Ok(to_binary(&GetDoesDelegationExistRepsonse {
            delegation_exists: !is_delegation_empty(
                deps.storage,
                &delegator_addr,
                &delegator_pubkey,
                &delegatee_pubkey,
            ),
        })?),

        QueryMsg::GetSelectedProxiesForDelegation {
            delegator_addr,
            delegator_pubkey,
            delegatee_pubkey,
        } => Ok(to_binary(&GetSelectedProxiesForDelegationResponse {
            proxy_pubkeys: get_all_proxies_from_delegation(
                deps.storage,
                &delegator_addr,
                &delegator_pubkey,
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

fn ensure_proxy(store: &dyn Storage, addr: &Addr) -> StdResult<()> {
    if !get_is_proxy(store, addr) {
        return generic_err!("Only proxy can execute this method.");
    }
    Ok(())
}

fn ensure_data_owner(store: &dyn Storage, data_id: &str, addr: &Addr) -> StdResult<()> {
    match get_data_entry(store, data_id) {
        None => generic_err!(format!("DataID {} doesn't exist.", data_id)),
        Some(data_entry) => {
            if &data_entry.delegator_addr != addr {
                generic_err!(format!(
                    "Only data owner {} can execute.",
                    data_entry.delegator_addr
                ))
            } else {
                Ok(())
            }
        }
    }
}

fn select_proxy_pubkeys(store: &dyn Storage) -> StdResult<Vec<String>> {
    let state: State = get_state(store)?;
    let proxy_pubkeys = get_all_available_proxy_pubkeys(store);

    // Select n_max_proxies or maximum possible
    let n_proxies = std::cmp::min(state.n_max_proxies as usize, proxy_pubkeys.len());

    if n_proxies < state.threshold as usize {
        return generic_err!("Less proxies than threshold registered");
    }

    Ok(proxy_pubkeys[0..n_proxies].to_vec())
}

pub fn get_all_fragments(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> Vec<String> {
    let mut fragments: Vec<String> = Vec::new();
    for request_id in get_all_delegatee_reencryption_requests(storage, data_id, delegatee_pubkey) {
        let request = get_reencryption_request(storage, &request_id).unwrap();

        match request.fragment {
            None => continue,
            Some(fragment) => fragments.push(fragment.clone()),
        }
    }
    fragments
}

pub fn unregister_proxy(deps: DepsMut, proxy_addr: &Addr, proxy_pubkey: &str) -> StdResult<()> {
    let state = get_state(deps.storage)?;

    // Delete all proxy delegations -- Make proxy inactive / stop requests factory
    for delegation_id in get_all_proxy_delegations(deps.storage, proxy_pubkey) {
        let delegation = get_delegation(deps.storage, &delegation_id).unwrap();

        // Check if number of proxies under delegation is below the threshold
        let all_delegation_proxies = get_all_proxies_from_delegation(
            deps.storage,
            &delegation.delegator_addr,
            &delegation.delegator_pubkey,
            &delegation.delegatee_pubkey,
        );
        if all_delegation_proxies.len() < (state.threshold as usize + 1) {
            // Delete entire delegation = delete each proxy delegation in delegation
            // And delete all related re-encryption requests
            for i_proxy_pubkey in all_delegation_proxies {
                let i_delegation_id = get_delegation_id(
                    deps.storage,
                    &delegation.delegator_addr,
                    &delegation.delegator_pubkey,
                    &delegation.delegatee_pubkey,
                    &i_proxy_pubkey,
                )
                .unwrap();

                remove_delegation(deps.storage, &i_delegation_id);
                remove_delegation_id(
                    deps.storage,
                    &delegation.delegator_addr,
                    &delegation.delegator_pubkey,
                    &delegation.delegatee_pubkey,
                    &i_proxy_pubkey,
                );
                remove_proxy_delegation(deps.storage, &i_proxy_pubkey, &i_delegation_id);
            }
        } else {
            // Delete proxy delegation only for current proxy with all active requests
            remove_delegation(deps.storage, &delegation_id);
            remove_delegation_id(
                deps.storage,
                &delegation.delegator_addr,
                &delegation.delegator_pubkey,
                &delegation.delegatee_pubkey,
                proxy_pubkey,
            );
            remove_proxy_delegation(deps.storage, proxy_pubkey, &delegation_id);
        }
    }

    // Delete all unfinished current proxy re-encryption requests
    for re_request_id in get_all_proxy_reencryption_requests(deps.storage, proxy_pubkey) {
        let re_request = get_reencryption_request(deps.storage, &re_request_id).unwrap();

        let all_related_requests_ids = get_all_delegatee_reencryption_requests(
            deps.storage,
            &re_request.data_id,
            &re_request.delegatee_pubkey,
        );

        if all_related_requests_ids.len() < (state.threshold as usize + 1) {
            // Delete other proxies related requests because request cannot be completed without this proxy

            for i_re_request_id in all_related_requests_ids {
                let i_re_request =
                    get_reencryption_request(deps.storage, &i_re_request_id).unwrap();
                remove_delegatee_reencryption_request(
                    deps.storage,
                    &i_re_request.data_id,
                    &i_re_request.delegatee_pubkey,
                    &i_re_request.proxy_pubkey,
                );
                remove_proxy_reencryption_request(
                    deps.storage,
                    &i_re_request.proxy_pubkey,
                    &i_re_request_id,
                );
                remove_reencryption_request(deps.storage, &i_re_request_id);
            }
        } else {
            // Delete only current proxy unfinished request

            remove_delegatee_reencryption_request(
                deps.storage,
                &re_request.data_id,
                &re_request.delegatee_pubkey,
                proxy_pubkey,
            );
            remove_proxy_reencryption_request(deps.storage, proxy_pubkey, &re_request_id);
            remove_reencryption_request(deps.storage, &re_request_id);
        }
    }

    // Remove proxy from registered proxies
    remove_proxy_address(deps.storage, proxy_pubkey);
    remove_proxy_pubkey(deps.storage, proxy_addr);
    Ok(())
}
