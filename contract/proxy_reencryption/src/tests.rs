use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::contract::{execute, get_all_fragments, get_next_proxy_task, instantiate};
use crate::delegations::{
    add_proxy_delegation, get_delegation, get_delegation_id, is_proxy_delegation, set_delegation,
    set_delegation_id, Delegation,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, ProxyDelegation, ProxyTask};
use crate::proxies::{
    get_all_active_proxy_pubkeys, get_is_proxy_active, get_proxy, get_proxy_address, ProxyState,
};
use crate::reencryption_requests::{
    get_all_proxy_reencryption_requests, get_delegatee_reencryption_request,
    get_reencryption_request, is_proxy_reencryption_request,
};
use crate::state::{get_data_entry, get_delegator_address, get_state, DataEntry, State};

fn mock_env_height(signer: &Addr, height: u64) -> (Env, MessageInfo) {
    let mut env = mock_env();
    env.block.height = height;
    let info = mock_info(signer.as_str(), &[]);

    return (env, info);
}

fn init_contract(
    deps: DepsMut,
    creator: &Addr,
    threshold: &Option<u32>,
    admin: &Option<Addr>,
    n_max_proxies: &Option<u32>,
    proxies: &Option<Vec<Addr>>,
) -> StdResult<Response> {
    let init_msg = InstantiateMsg {
        threshold: threshold.clone(),
        admin: admin.clone(),
        n_max_proxies: n_max_proxies.clone(),
        proxies: proxies.clone(),
    };
    let env = mock_env_height(&creator, 450);
    return instantiate(deps, env.0, env.1, init_msg);
}

fn add_proxy(deps: DepsMut, creator: &Addr, proxy_addr: &Addr) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::AddProxy {
        proxy_addr: proxy_addr.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn remove_proxy(deps: DepsMut, creator: &Addr, proxy_addr: &Addr) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::RemoveProxy {
        proxy_addr: proxy_addr.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn register_proxy(deps: DepsMut, creator: &Addr, proxy_pubkey: &String) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::RegisterProxy {
        proxy_pubkey: proxy_pubkey.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn unregister_proxy(deps: DepsMut, creator: &Addr) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::UnregisterProxy {};

    return execute(deps, env.0, env.1, msg);
}

fn deactivate_proxy(deps: DepsMut, creator: &Addr) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::DeactivateProxy {};

    return execute(deps, env.0, env.1, msg);
}

fn provide_reencrypted_fragment(
    deps: DepsMut,
    creator: &Addr,
    data_id: &String,
    delegatee_pubkey: &String,
    fragment: &String,
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::ProvideReencryptedFragment {
        data_id: data_id.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
        fragment: fragment.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn add_data(
    deps: DepsMut,
    creator: &Addr,
    data_id: &String,
    delegator_pubkey: &String,
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::AddData {
        data_id: data_id.clone(),
        delegator_pubkey: delegator_pubkey.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn add_delegation(
    deps: DepsMut,
    creator: &Addr,
    delegator_pubkey: &String,
    delegatee_pubkey: &String,
    proxy_delegations: &[ProxyDelegation],
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::AddDelegation {
        delegator_pubkey: delegator_pubkey.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
        proxy_delegations: proxy_delegations.to_vec(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn request_proxies_for_delegation(
    deps: DepsMut,
    creator: &Addr,
    delegator_pubkey: &String,
    delegatee_pubkey: &String,
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::RequestProxiesForDelegation {
        delegator_pubkey: delegator_pubkey.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn request_reencryption(
    deps: DepsMut,
    creator: &Addr,
    data_id: &String,
    delegatee_pubkey: &String,
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450);

    let msg = ExecuteMsg::RequestReencryption {
        data_id: data_id.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

#[test]
fn test_new_contract_default_values() {
    let mut deps = mock_dependencies(&[]);
    let creator = Addr::unchecked("creator".to_string());
    let proxy = Addr::unchecked("proxy".to_string());

    let proxies: Vec<Addr> = vec![creator.clone(), proxy.clone()];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &None,
        &Some(proxies.clone()),
    )
    .is_ok());

    let state: State = get_state(&deps.storage).unwrap();
    let available_proxies = get_all_active_proxy_pubkeys(&deps.storage);

    assert_eq!(available_proxies.len(), 0);

    assert_eq!(&state.admin, &creator);
    assert_eq!(state.n_max_proxies, u32::MAX);
    assert_eq!(&state.threshold, &1u32);
    assert_eq!(&state.next_request_id, &0u64);
    assert_eq!(&state.next_delegation_id, &0u64);
}

#[test]
fn test_new_contract_custom_values() {
    let mut deps = mock_dependencies(&[]);
    let creator = Addr::unchecked("creator".to_string());
    let proxy = Addr::unchecked("proxy".to_string());

    let proxies: Vec<Addr> = vec![creator.clone(), proxy.clone()];

    // n_max_proxies cannot be less than threshold
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(123),
        &Some(proxy.clone()),
        &Some(122),
        &Some(proxies.clone()),
    )
    .is_err());

    // Threshold cannot be zero
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(0),
        &Some(proxy.clone()),
        &Some(456),
        &Some(proxies.clone()),
    )
    .is_err());

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(123),
        &Some(proxy.clone()),
        &Some(456),
        &Some(proxies.clone()),
    )
    .is_ok());

    let state: State = get_state(&deps.storage).unwrap();
    let available_proxies = get_all_active_proxy_pubkeys(&deps.storage);

    assert_eq!(available_proxies.len(), 0);

    assert_eq!(&state.admin, &proxy);
    assert_eq!(&state.n_max_proxies, &456);
    assert_eq!(&state.threshold, &123);
    assert_eq!(&state.next_request_id, &0u64);
    assert_eq!(&state.next_delegation_id, &0u64);
}

#[test]
fn test_add_remove_proxy() {
    let mut deps = mock_dependencies(&[]);
    let creator = Addr::unchecked("creator".to_string());
    let admin = Addr::unchecked("admin".to_string());
    let proxy = Addr::unchecked("proxy".to_string());

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &Some(admin.clone()),
        &None,
        &None,
    )
    .is_ok());

    // Only admin can add proxies
    assert!(add_proxy(deps.as_mut(), &creator, &proxy).is_err());
    assert!(add_proxy(deps.as_mut(), &admin, &proxy).is_ok());

    // Already added
    assert!(add_proxy(deps.as_mut(), &admin, &proxy).is_err());

    // Only admin can remove proxies
    assert!(remove_proxy(deps.as_mut(), &creator, &proxy).is_err());
    assert!(remove_proxy(deps.as_mut(), &admin, &proxy).is_ok());

    // Already removed
    assert!(remove_proxy(deps.as_mut(), &admin, &proxy).is_err());
}

#[test]
fn test_register_unregister_proxy() {
    let mut deps = mock_dependencies(&[]);
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let proxy_pubkey: String = String::from("proxy_pubkey");

    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &None,
        &Some(proxies.clone()),
    )
    .is_ok());

    assert_eq!(get_all_active_proxy_pubkeys(&deps.storage).len(), 0);

    // Check proxy state
    assert!(!get_is_proxy_active(deps.as_mut().storage, &proxy_pubkey));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy_pubkey)
        .unwrap()
        .is_none());
    assert!(get_proxy_address(deps.as_mut().storage, &proxy_pubkey).is_none());
    let proxy = get_proxy(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(proxy.state, ProxyState::Authorised);
    assert!(proxy.proxy_pubkey.is_none());

    // Only proxy can add pubkeys
    assert!(register_proxy(deps.as_mut(), &creator, &proxy_pubkey).is_err());
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy_pubkey).is_ok());
    // Already registered
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy_pubkey).is_err());

    // Check proxy state
    assert!(get_is_proxy_active(deps.as_mut().storage, &proxy_pubkey));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy_pubkey)
        .unwrap()
        .is_none());
    assert_eq!(
        get_proxy_address(deps.as_mut().storage, &proxy_pubkey).unwrap(),
        proxy1
    );
    let proxy = get_proxy(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(proxy.state, ProxyState::Registered);
    assert_eq!(proxy.proxy_pubkey.unwrap(), proxy_pubkey);

    let available_proxy_pubkeys = get_all_active_proxy_pubkeys(&deps.storage);
    assert_eq!(available_proxy_pubkeys.len(), 1);
    assert_eq!(&available_proxy_pubkeys, &[proxy_pubkey.clone()]);

    // Register different proxy with existing pubkey
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy_pubkey).is_err());

    // Number of available pubkeys remains the same
    let available_proxy_pubkeys = get_all_active_proxy_pubkeys(&deps.storage);
    assert_eq!(available_proxy_pubkeys.len(), 1);
    assert_eq!(&available_proxy_pubkeys, &[proxy_pubkey.clone()]);

    // Only proxy can remove pubkeys
    assert!(unregister_proxy(deps.as_mut(), &creator).is_err());
    assert!(unregister_proxy(deps.as_mut(), &proxy1).is_ok());
    // Already unregistered
    assert!(unregister_proxy(deps.as_mut(), &proxy1).is_err());

    // All proxies unregistered
    assert_eq!(get_all_active_proxy_pubkeys(&deps.storage).len(), 0);
}

#[test]
fn test_add_data() {
    let mut deps = mock_dependencies(&[]);

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let delegator1_pubkey: String = String::from("DRK1");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");

    let data_entry = DataEntry {
        delegator_pubkey: delegator1_pubkey.clone(),
    };

    /*************** Initialise *************/
    assert!(init_contract(deps.as_mut(), &creator, &None, &None, &None, &None).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        &data_id1,
        &data_entry.delegator_pubkey,
    )
    .is_ok());

    // Data already added
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        &data_id1,
        &data_entry.delegator_pubkey,
    )
    .is_err());

    assert_eq!(
        &get_data_entry(deps.as_mut().storage, &data_id1).unwrap(),
        &data_entry
    );
    assert_eq!(
        get_delegator_address(deps.as_mut().storage, &delegator1_pubkey).unwrap(),
        delegator1
    );

    // Delgator2 cannot use delegator1 pubkey
    assert!(add_data(deps.as_mut(), &delegator2, &data_id2, &delegator1_pubkey).is_err());
}

#[test]
fn test_select_proxies_add_delegation_and_request_reencryption() {
    let mut deps = mock_dependencies(&[]);

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let delegator1_pubkey: String = String::from("DRK1");

    let delegatee_pubkey: String = String::from("DEK1");
    let proxy1_pubkey: String = String::from("proxy1_pubkey");
    let proxy2_pubkey: String = String::from("proxy2_pubkey");

    let data_id = String::from("DATA");
    let data_entry = DataEntry {
        delegator_pubkey: delegator1_pubkey.clone(),
    };

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &Some(1),
        &Some(vec![proxy1.clone(), proxy2.clone()]),
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        &data_id,
        &data_entry.delegator_pubkey,
    )
    .is_ok());

    // Add delegation for proxy
    let proxy1_delegation_string = String::from("DS_P1");
    let proxy2_delegation_string = String::from("DS_P2");

    let proxy_delegations: Vec<ProxyDelegation> = vec![ProxyDelegation {
        proxy_pubkey: proxy1_pubkey.clone(),
        delegation_string: proxy1_delegation_string.clone(),
    }];

    let different_proxy_delegations: Vec<ProxyDelegation> = vec![ProxyDelegation {
        proxy_pubkey: proxy2_pubkey.clone(),
        delegation_string: proxy2_delegation_string.clone(),
    }];

    let different_proxy_amount_delegations: Vec<ProxyDelegation> = vec![
        ProxyDelegation {
            proxy_pubkey: proxy2_pubkey.clone(),
            delegation_string: proxy2_delegation_string.clone(),
        },
        ProxyDelegation {
            proxy_pubkey: proxy2_pubkey.clone(),
            delegation_string: proxy2_delegation_string.clone(),
        },
    ];

    // Reencryption can't be requested yet
    assert!(request_reencryption(deps.as_mut(), &delegator1, &data_id, &delegatee_pubkey).is_err());

    // Proxies not requested
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee_pubkey,
        &proxy_delegations,
    )
    .is_err());

    let res = request_proxies_for_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee_pubkey,
    )
    .unwrap();
    // Check if proxy 1 was selected
    assert_eq!(
        format!("[\"{}\", ]", proxy1_pubkey),
        res.attributes[4].value
    );

    // Reencryption can't be requested yet - No delegation strings added
    assert!(request_reencryption(deps.as_mut(), &delegator1, &data_id, &delegatee_pubkey).is_err());

    // Add delegation with different proxy than selected one
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee_pubkey,
        &different_proxy_delegations,
    )
    .is_err());

    // Add delegation with different amount of proxies than selected one
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee_pubkey,
        &different_proxy_amount_delegations,
    )
    .is_err());

    // Add delegation
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee_pubkey,
        &proxy_delegations,
    )
    .is_ok());

    // Cannot add same delegation twice
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee_pubkey,
        &proxy_delegations,
    )
    .is_err());

    // Reencryption cannot be requested by delegator2
    assert!(request_reencryption(deps.as_mut(), &delegator2, &data_id, &delegatee_pubkey).is_err());

    // Reencryption can be requested only after add_delegation
    assert!(request_reencryption(deps.as_mut(), &delegator1, &data_id, &delegatee_pubkey).is_ok());

    // Reencryption already requested
    assert!(request_reencryption(deps.as_mut(), &delegator1, &data_id, &delegatee_pubkey).is_err());

    // Check if request was created
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id,
            &delegatee_pubkey,
            &proxy1_pubkey,
        ),
        Some(0u64)
    );

    assert_eq!(
        get_state(deps.as_mut().storage).unwrap().next_request_id,
        1u64
    );
}

#[test]
fn test_add_delegation_and_then_data_with_diffent_proxy_same_pubkey() {
    let mut deps = mock_dependencies(&[]);

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let delegator1_pubkey: String = String::from("DRK1");
    let delegator2_pubkey: String = String::from("DRK2");

    let delegatee_pubkey: String = String::from("DEK1");
    let proxy1_pubkey: String = String::from("proxy1_pubkey");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &Some(1),
        &Some(vec![proxy1.clone(), proxy2.clone()]),
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey).is_ok());

    // Add delegation for proxy
    let proxy1_delegation_string = String::from("DS_P1");

    let proxy_delegations: Vec<ProxyDelegation> = vec![ProxyDelegation {
        proxy_pubkey: proxy1_pubkey.clone(),
        delegation_string: proxy1_delegation_string.clone(),
    }];

    // Request proxies
    assert!(request_proxies_for_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee_pubkey,
    )
    .is_ok());

    // Add delegation
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee_pubkey,
        &proxy_delegations,
    )
    .is_ok());

    // Add data by delegator2 with already used delegator1_pubkey is prevented
    assert!(add_data(deps.as_mut(), &delegator2, &data_id1, &delegator1_pubkey).is_err());

    assert!(add_data(deps.as_mut(), &delegator2, &data_id2, &delegator2_pubkey).is_ok());

    // Cannot add delegation by delegator1 using delegator2 pubkey
    assert!(request_proxies_for_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator2_pubkey,
        &delegatee_pubkey,
    )
    .is_err());
}

#[test]
fn test_provide_reencrypted_fragment() {
    let mut deps = mock_dependencies(&[]);

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy = Addr::unchecked("proxy".to_string());

    let delegator = Addr::unchecked("delegator".to_string());

    // Pubkeys
    let delegator_pubkey: String = String::from("DRK");
    let delegatee_pubkey: String = String::from("DEK1");
    let other_delegatee_pubkey: String = String::from("DEK2");
    let proxy_pubkey: String = String::from("proxy_pubkey");

    let data_id = String::from("DATA");

    let data_entry = DataEntry {
        delegator_pubkey: delegator_pubkey.clone(),
    };

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &None,
        &Some(vec![proxy.clone()]),
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy, &proxy_pubkey).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator,
        &data_id,
        &data_entry.delegator_pubkey,
    )
    .is_ok());

    // Add delegation for proxy
    let proxy_delegation_string = String::from("DS_P1");
    let proxy_delegations: Vec<ProxyDelegation> = vec![ProxyDelegation {
        proxy_pubkey: proxy_pubkey.clone(),
        delegation_string: proxy_delegation_string.clone(),
    }];

    assert!(request_proxies_for_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee_pubkey,
    )
    .is_ok());

    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee_pubkey,
        &proxy_delegations,
    )
    .is_ok());

    /*************** Request re-encryption *************/
    assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee_pubkey).is_ok());

    /*************** Provide reencrypted fragment *************/
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id,
            &delegatee_pubkey,
            &proxy_pubkey,
        )
        .unwrap(),
        0u64
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy_pubkey,
        &0u64,
    ));

    let proxy_fragment: String = String::from("PR1_FRAG1");
    // Provide unwanted fragment
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy,
        &data_id,
        &other_delegatee_pubkey,
        &proxy_fragment,
    )
    .is_err());

    // Not a proxy
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &creator,
        &data_id,
        &delegatee_pubkey,
        &proxy_fragment,
    )
    .is_err());

    // Provide fragment correctly
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy,
        &data_id,
        &delegatee_pubkey,
        &proxy_fragment,
    )
    .is_ok());
    // Fragment already provided
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy,
        &data_id,
        &delegatee_pubkey,
        &proxy_fragment,
    )
    .is_err());

    // This entry is removed when proxy task is done
    assert!(!is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy_pubkey,
        &0u64,
    ));

    let request = get_reencryption_request(deps.as_mut().storage, &0u64).unwrap();
    assert_eq!(request.fragment, Some(proxy_fragment));
}

#[test]
fn test_contract_lifecycle() {
    let mut deps = mock_dependencies(&[]);

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());

    let delegator = Addr::unchecked("delegator".to_string());

    // Pubkeys
    let delegator_pubkey: String = String::from("DRK");
    let delegatee1_pubkey: String = String::from("DEK1");
    let delegatee2_pubkey: String = String::from("DEK2");
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");

    let data_id = String::from("DATA");
    let data_entry = DataEntry {
        delegator_pubkey: delegator_pubkey.clone(),
    };

    /*************** Initialise *************/
    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(2),
        &None,
        &None,
        &Some(proxies.clone()),
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator,
        &data_id,
        &data_entry.delegator_pubkey,
    )
    .is_ok());

    // Add 2 delegations for 2 proxies
    let proxy1_delegation_string = String::from("DS_P1");
    let proxy2_delegation_string = String::from("DS_P2");

    let proxy_delegations: Vec<ProxyDelegation> = vec![
        ProxyDelegation {
            proxy_pubkey: proxy1_pubkey.clone(),
            delegation_string: proxy1_delegation_string.clone(),
        },
        ProxyDelegation {
            proxy_pubkey: proxy2_pubkey.clone(),
            delegation_string: proxy2_delegation_string.clone(),
        },
    ];

    assert!(request_proxies_for_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee1_pubkey,
    )
    .is_ok());
    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee1_pubkey,
        &proxy_delegations,
    )
    .is_ok());

    assert!(request_proxies_for_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee2_pubkey,
    )
    .is_ok());
    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee2_pubkey,
        &proxy_delegations,
    )
    .is_ok());

    // No tasks yet
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy1_pubkey)
        .unwrap()
        .is_none());
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey)
        .unwrap()
        .is_none());

    /*************** Request reencryption by delegator *************/

    assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee1_pubkey).is_ok());

    // Check number of requests
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey).len(),
        1
    );
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy2_pubkey).len(),
        1
    );

    assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee2_pubkey).is_ok());

    // Check number of requests
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey).len(),
        2
    );
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy2_pubkey).len(),
        2
    );

    /*************** Process reencryption by proxies *************/
    let all_requests = get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey);
    assert_eq!(all_requests.len(), 2);

    // Check if proxy got task 1
    let proxy1_task1 = get_next_proxy_task(deps.as_mut().storage, &proxy1_pubkey)
        .unwrap()
        .unwrap();
    assert_eq!(
        proxy1_task1,
        ProxyTask {
            data_id: data_id.clone(),
            delegatee_pubkey: delegatee1_pubkey.clone(),
            delegator_pubkey: delegator_pubkey.clone(),
            delegation_string: proxy1_delegation_string.clone(),
        }
    );

    // Proxy1 provides fragment for task1
    let proxy1_fragment1: String = String::from("PR1_FRAG1");
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy1,
        &data_id,
        &delegatee1_pubkey,
        &proxy1_fragment1,
    )
    .is_ok());

    // Check numbers of requests
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey).len(),
        1
    );
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy2_pubkey).len(),
        2
    );

    // Check available fragments
    assert_eq!(
        get_all_fragments(deps.as_mut().storage, &data_id, &delegatee1_pubkey),
        vec![proxy1_fragment1.clone()]
    );
    assert_eq!(
        get_all_fragments(deps.as_mut().storage, &data_id, &delegatee2_pubkey).len(),
        0
    );

    // Check if proxy got task 2
    let proxy1_task2 = get_next_proxy_task(deps.as_mut().storage, &proxy1_pubkey)
        .unwrap()
        .unwrap();
    assert_eq!(
        proxy1_task2,
        ProxyTask {
            data_id: data_id.clone(),
            delegatee_pubkey: delegatee2_pubkey.clone(),
            delegator_pubkey: delegator_pubkey.clone(),
            delegation_string: proxy1_delegation_string.clone(),
        }
    );

    // Proxy1 provides fragment for task1
    let proxy1_fragment2: String = String::from("PR1_FRAG2");
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy1,
        &data_id,
        &delegatee2_pubkey,
        &proxy1_fragment2,
    )
    .is_ok());

    // All tasks completed for proxy1
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy1_pubkey)
        .unwrap()
        .is_none());
    // But not for proxy2
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey)
        .unwrap()
        .is_some());

    // Check available fragments
    assert_eq!(
        get_all_fragments(deps.as_mut().storage, &data_id, &delegatee1_pubkey),
        vec![proxy1_fragment1]
    );
    assert_eq!(
        get_all_fragments(deps.as_mut().storage, &data_id, &delegatee2_pubkey),
        vec![proxy1_fragment2]
    );

    // Re-encryption was requested in past
    assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee1_pubkey).is_err());

    // Proxy 1 leaves - all its delegations gets deleted
    assert!(unregister_proxy(deps.as_mut(), &proxy1).is_ok());

    // Proxy 1 gets back
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey).is_ok());

    // Delegation cannot be created again
    assert!(request_proxies_for_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee1_pubkey,
    )
    .is_err());
    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee1_pubkey,
        &proxy_delegations,
    )
    .is_err());
}

#[test]
fn test_proxy_unregister_with_requests() {
    let mut deps = mock_dependencies(&[]);

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());
    let proxy3 = Addr::unchecked("proxy_3".to_string());
    let proxy4 = Addr::unchecked("proxy_4".to_string());
    let proxy5 = Addr::unchecked("proxy_5".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let delegator1_pubkey: String = String::from("DRK1");
    let delegator2_pubkey: String = String::from("DRK2");

    let delegatee1_pubkey: String = String::from("DEK1");
    let delegatee2_pubkey: String = String::from("DEK2");

    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");
    let proxy4_pubkey: String = String::from("proxy_pubkey4");
    let proxy5_pubkey: String = String::from("proxy_pubkey5");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");
    let data_id3 = String::from("DATA3");

    let delegation_string = String::from("DELESTRING");
    let re_encrypted_fragment = String::from("FRAGMENT");

    /*************** Initialise *************/
    let proxies: Vec<Addr> = vec![
        proxy1.clone(),
        proxy2.clone(),
        proxy3.clone(),
        proxy4.clone(),
        proxy5.clone(),
    ];
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(2),
        &None,
        &None,
        &Some(proxies.clone()),
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy3, &proxy3_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy4, &proxy4_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy5, &proxy5_pubkey).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(deps.as_mut(), &delegator1, &data_id1, &delegator1_pubkey).is_ok());
    assert!(add_data(deps.as_mut(), &delegator1, &data_id2, &delegator1_pubkey).is_ok());
    assert!(add_data(deps.as_mut(), &delegator2, &data_id3, &delegator2_pubkey).is_ok());

    // Add delegations manually

    let delegation1 = Delegation {
        delegator_pubkey: delegator1_pubkey.clone(),
        delegatee_pubkey: delegatee1_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee1
    set_delegation(deps.as_mut().storage, &0, &delegation1);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy1_pubkey,
        &0,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy1_pubkey, &0);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee1
    set_delegation(deps.as_mut().storage, &1, &delegation1);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy2_pubkey,
        &1,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy2_pubkey, &1);

    // delegator1 with delegator1_pubkey and proxy3 for delegatee1
    set_delegation(deps.as_mut().storage, &2, &delegation1);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy3_pubkey,
        &2,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy3_pubkey, &2);

    let delegation2 = Delegation {
        delegator_pubkey: delegator1_pubkey.clone(),
        delegatee_pubkey: delegatee2_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee2
    set_delegation(deps.as_mut().storage, &3, &delegation2);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy1_pubkey,
        &3,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy1_pubkey, &3);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee2
    set_delegation(deps.as_mut().storage, &4, &delegation2);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy2_pubkey,
        &4,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy2_pubkey, &4);

    let delegation3 = Delegation {
        delegator_pubkey: delegator2_pubkey.clone(),
        delegatee_pubkey: delegatee1_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator2 with delegator2_pubkey and proxy4 for delegatee1
    set_delegation(deps.as_mut().storage, &5, &delegation3);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy4_pubkey,
        &5,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy4_pubkey, &5);

    // delegator2 with delegator2_pubkey and proxy5 for delegatee1
    set_delegation(deps.as_mut().storage, &6, &delegation3);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy5_pubkey,
        &6,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy5_pubkey, &6);

    // Request re-encryptions

    // Re-encryption requests with DATA1 to delegatee1
    assert!(
        request_reencryption(deps.as_mut(), &delegator1, &data_id1, &delegatee1_pubkey).is_ok()
    );
    // Re-encryption requests with DATA1 to delegatee2
    assert!(
        request_reencryption(deps.as_mut(), &delegator1, &data_id1, &delegatee2_pubkey).is_ok()
    );
    // Re-encryption requests with DATA2 to delegatee2
    assert!(
        request_reencryption(deps.as_mut(), &delegator1, &data_id2, &delegatee2_pubkey).is_ok()
    );
    // Re-encryption requests with DATA3 to delegatee1
    assert!(
        request_reencryption(deps.as_mut(), &delegator2, &data_id3, &delegatee1_pubkey).is_ok()
    );

    // Complete requests
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy2,
        &data_id1,
        &delegatee2_pubkey,
        &re_encrypted_fragment,
    )
    .is_ok());

    // Unregister proxy2
    assert!(unregister_proxy(deps.as_mut(), &proxy2).is_ok());
    // Already unregistered
    assert!(unregister_proxy(deps.as_mut(), &proxy2).is_err());

    // Check state of delegations

    // Delegation 1 - Removed because of proxy 2
    // delgator1, delegatee1, proxy1 - Unaffected
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &0).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &0,
    ));

    // delgator1, delegatee1, proxy2 - Removed
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &1).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &1,
    ));

    // delgator1, delegatee1, proxy3 - Removed because of proxy 2
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy3_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &2).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy3_pubkey,
        &2,
    ));

    // Delegation 2 - Number of proxies below threshold, removed entire delegation
    // delgator1, delegatee2, proxy1 - Removed
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee2_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &3).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &3,
    ));

    // delgator1, delegatee2, proxy2 - Removed
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee2_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &4).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &4,
    ));

    // Delegation 3 - Unaffected
    // delgator2, delegatee1, proxy4 - Unaffected
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator2_pubkey,
        &delegatee1_pubkey,
        &proxy4_pubkey,
    )
    .is_some());
    assert!(get_delegation(deps.as_mut().storage, &5).is_some());
    assert!(is_proxy_delegation(
        deps.as_mut().storage,
        &proxy4_pubkey,
        &5,
    ));

    // delgator2, delegatee1, proxy5 - Unaffected
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator2_pubkey,
        &delegatee1_pubkey,
        &proxy5_pubkey,
    )
    .is_some());
    assert!(get_delegation(deps.as_mut().storage, &6).is_some());
    assert!(is_proxy_delegation(
        deps.as_mut().storage,
        &proxy5_pubkey,
        &6,
    ));

    // Check state of re-encryption requests

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee1, proxy1 - unaffected
    assert!(get_reencryption_request(deps.as_mut().storage, &0).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee1_pubkey,
            &proxy1_pubkey,
        )
        .unwrap(),
        0
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &0,
    ));

    // DATA1, delegatee1, proxy2 - incomplete and removed
    assert!(get_reencryption_request(deps.as_mut().storage, &1).is_none());
    assert!(get_delegatee_reencryption_request(
        deps.as_mut().storage,
        &data_id1,
        &delegatee1_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(!is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &1,
    ));

    // DATA1, delegatee1, proxy3 - unaffected
    assert!(get_reencryption_request(deps.as_mut().storage, &2).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee1_pubkey,
            &proxy3_pubkey,
        )
        .unwrap(),
        2
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy3_pubkey,
        &2,
    ));

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee2, proxy1 - Not removed because it can still be completed
    assert!(get_reencryption_request(deps.as_mut().storage, &3).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee2_pubkey,
            &proxy1_pubkey,
        )
        .unwrap(),
        3
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &3,
    ));

    // DATA1, delegatee2, proxy2 - complete - unaffected
    // Request can still be obtained by delegatee
    assert!(get_reencryption_request(deps.as_mut().storage, &4).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee2_pubkey,
            &proxy2_pubkey,
        )
        .unwrap(),
        4
    );
    // Request is completed - won't appear in proxy tasks
    assert!(!is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &4,
    ));

    // Re-encryption requests with DATA2 to delegatee2
    // DATA2, delegatee2, proxy1 - incomplete and removed
    assert!(get_reencryption_request(deps.as_mut().storage, &5).is_none());
    assert!(get_delegatee_reencryption_request(
        deps.as_mut().storage,
        &data_id2,
        &delegatee2_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(!is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &5,
    ));

    // DATA1, delegatee2, proxy2 - complete - removed because more than threshold fragments cannot be provided
    // Request can still be obtained by delegatee
    assert!(get_reencryption_request(deps.as_mut().storage, &6).is_none());
    assert!(get_delegatee_reencryption_request(
        deps.as_mut().storage,
        &data_id2,
        &delegatee2_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    // Request is completed - won't appear in proxy tasks
    assert!(!is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &6,
    ));

    // Re-encryption requests with DATA3 to delegatee1
    // DATA3, delegatee1, proxy4 - unaffected
    assert!(get_reencryption_request(deps.as_mut().storage, &7).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id3,
            &delegatee1_pubkey,
            &proxy4_pubkey,
        )
        .unwrap(),
        7
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy4_pubkey,
        &7,
    ));

    // DATA3, delegatee1, proxy5 - unaffected
    assert!(get_reencryption_request(deps.as_mut().storage, &8).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id3,
            &delegatee1_pubkey,
            &proxy5_pubkey,
        )
        .unwrap(),
        8
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy5_pubkey,
        &8,
    ));

    // Delegation 1 was removed with all re-encryption requests
    assert!(
        request_reencryption(deps.as_mut(), &delegator1, &data_id1, &delegatee1_pubkey).is_err()
    );
}

#[test]
fn test_proxy_deactivate_and_remove_with_requests() {
    let mut deps = mock_dependencies(&[]);

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());
    let proxy3 = Addr::unchecked("proxy_3".to_string());
    let proxy4 = Addr::unchecked("proxy_4".to_string());
    let proxy5 = Addr::unchecked("proxy_5".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let delegator1_pubkey: String = String::from("DRK1");
    let delegator2_pubkey: String = String::from("DRK2");

    let delegatee1_pubkey: String = String::from("DEK1");
    let delegatee2_pubkey: String = String::from("DEK2");

    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");
    let proxy4_pubkey: String = String::from("proxy_pubkey4");
    let proxy5_pubkey: String = String::from("proxy_pubkey5");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");
    let data_id3 = String::from("DATA3");

    let delegation_string = String::from("DELESTRING");
    let re_encrypted_fragment = String::from("FRAGMENT");

    /*************** Initialise *************/
    let proxies: Vec<Addr> = vec![
        proxy1.clone(),
        proxy2.clone(),
        proxy3.clone(),
        proxy4.clone(),
        proxy5.clone(),
    ];
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(2),
        &None,
        &None,
        &Some(proxies.clone()),
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy3, &proxy3_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy4, &proxy4_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy5, &proxy5_pubkey).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(deps.as_mut(), &delegator1, &data_id1, &delegator1_pubkey).is_ok());
    assert!(add_data(deps.as_mut(), &delegator1, &data_id2, &delegator1_pubkey).is_ok());
    assert!(add_data(deps.as_mut(), &delegator2, &data_id3, &delegator2_pubkey).is_ok());

    // Add delegations manually

    let delegation1 = Delegation {
        delegator_pubkey: delegator1_pubkey.clone(),
        delegatee_pubkey: delegatee1_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee1
    set_delegation(deps.as_mut().storage, &0, &delegation1);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy1_pubkey,
        &0,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy1_pubkey, &0);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee1
    set_delegation(deps.as_mut().storage, &1, &delegation1);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy2_pubkey,
        &1,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy2_pubkey, &1);

    // delegator1 with delegator1_pubkey and proxy3 for delegatee1
    set_delegation(deps.as_mut().storage, &2, &delegation1);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy3_pubkey,
        &2,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy3_pubkey, &2);

    let delegation2 = Delegation {
        delegator_pubkey: delegator1_pubkey.clone(),
        delegatee_pubkey: delegatee2_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee2
    set_delegation(deps.as_mut().storage, &3, &delegation2);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy1_pubkey,
        &3,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy1_pubkey, &3);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee2
    set_delegation(deps.as_mut().storage, &4, &delegation2);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy2_pubkey,
        &4,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy2_pubkey, &4);

    let delegation3 = Delegation {
        delegator_pubkey: delegator2_pubkey.clone(),
        delegatee_pubkey: delegatee1_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator2 with delegator2_pubkey and proxy4 for delegatee1
    set_delegation(deps.as_mut().storage, &5, &delegation3);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy4_pubkey,
        &5,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy4_pubkey, &5);

    // delegator2 with delegator2_pubkey and proxy5 for delegatee1
    set_delegation(deps.as_mut().storage, &6, &delegation3);
    set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy5_pubkey,
        &6,
    );
    add_proxy_delegation(deps.as_mut().storage, &proxy5_pubkey, &6);

    // Request re-encryptions

    // Re-encryption requests with DATA1 to delegatee1
    assert!(
        request_reencryption(deps.as_mut(), &delegator1, &data_id1, &delegatee1_pubkey).is_ok()
    );
    // Re-encryption requests with DATA1 to delegatee2
    assert!(
        request_reencryption(deps.as_mut(), &delegator1, &data_id1, &delegatee2_pubkey).is_ok()
    );
    // Re-encryption requests with DATA2 to delegatee2
    assert!(
        request_reencryption(deps.as_mut(), &delegator1, &data_id2, &delegatee2_pubkey).is_ok()
    );
    // Re-encryption requests with DATA3 to delegatee1
    assert!(
        request_reencryption(deps.as_mut(), &delegator2, &data_id3, &delegatee1_pubkey).is_ok()
    );

    // Complete requests
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy2,
        &data_id1,
        &delegatee2_pubkey,
        &re_encrypted_fragment,
    )
    .is_ok());

    // Check proxy state
    assert!(get_is_proxy_active(deps.as_mut().storage, &proxy2_pubkey));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey)
        .unwrap()
        .is_some());
    assert_eq!(
        get_proxy_address(deps.as_mut().storage, &proxy2_pubkey).unwrap(),
        proxy2
    );
    let proxy = get_proxy(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(proxy.state, ProxyState::Registered);
    assert_eq!(proxy.proxy_pubkey.unwrap(), proxy2_pubkey);

    // Sender is not a proxy
    assert!(deactivate_proxy(deps.as_mut(), &creator).is_err());

    // Deactivate proxy2
    assert!(deactivate_proxy(deps.as_mut(), &proxy2).is_ok());
    // Already deactivated
    assert!(deactivate_proxy(deps.as_mut(), &proxy2).is_err());

    // Check proxy state
    assert!(!get_is_proxy_active(deps.as_mut().storage, &proxy2_pubkey));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey)
        .unwrap()
        .is_some());
    assert_eq!(
        get_proxy_address(deps.as_mut().storage, &proxy2_pubkey).unwrap(),
        proxy2
    );
    let proxy = get_proxy(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(proxy.state, ProxyState::Leaving);
    assert_eq!(proxy.proxy_pubkey.unwrap(), proxy2_pubkey);

    // Check state of delegations

    // Delegation 1 - Removed because of proxy 2
    // delgator1, delegatee1, proxy1 - Unaffected
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &0).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &0,
    ));

    // delgator1, delegatee1, proxy2 - Removed
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &1).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &1,
    ));

    // delgator1, delegatee1, proxy3 - Removed because of proxy 2
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy3_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &2).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy3_pubkey,
        &2,
    ));

    // Delegation 2 - Number of proxies below threshold, removed entire delegation
    // delgator1, delegatee2, proxy1 - Removed
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee2_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &3).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &3,
    ));

    // delgator1, delegatee2, proxy2 - Removed
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee2_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(get_delegation(deps.as_mut().storage, &4).is_none());
    assert!(!is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &4,
    ));

    // Delegation 3 - Unaffected
    // delgator2, delegatee1, proxy4 - Unaffected
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator2_pubkey,
        &delegatee1_pubkey,
        &proxy4_pubkey,
    )
    .is_some());
    assert!(get_delegation(deps.as_mut().storage, &5).is_some());
    assert!(is_proxy_delegation(
        deps.as_mut().storage,
        &proxy4_pubkey,
        &5,
    ));

    // delgator2, delegatee1, proxy5 - Unaffected
    assert!(get_delegation_id(
        deps.as_mut().storage,
        &delegator2_pubkey,
        &delegatee1_pubkey,
        &proxy5_pubkey,
    )
    .is_some());
    assert!(get_delegation(deps.as_mut().storage, &6).is_some());
    assert!(is_proxy_delegation(
        deps.as_mut().storage,
        &proxy5_pubkey,
        &6,
    ));

    // Remove proxy by admin
    assert!(remove_proxy(deps.as_mut(), &creator, &proxy2).is_ok());
    // Already removed
    assert!(remove_proxy(deps.as_mut(), &creator, &proxy2).is_err());

    // Check proxy state
    assert!(!get_is_proxy_active(deps.as_mut().storage, &proxy2_pubkey));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey)
        .unwrap()
        .is_none());
    assert!(get_proxy_address(deps.as_mut().storage, &proxy2_pubkey).is_none());
    assert!(get_proxy(deps.as_mut().storage, &proxy2).is_none());

    // Check state of re-encryption requests

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee1, proxy1 - unaffected
    assert!(get_reencryption_request(deps.as_mut().storage, &0).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee1_pubkey,
            &proxy1_pubkey,
        )
        .unwrap(),
        0
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &0,
    ));

    // DATA1, delegatee1, proxy2 - incomplete and removed
    assert!(get_reencryption_request(deps.as_mut().storage, &1).is_none());
    assert!(get_delegatee_reencryption_request(
        deps.as_mut().storage,
        &data_id1,
        &delegatee1_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(!is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &1,
    ));

    // DATA1, delegatee1, proxy3 - unaffected
    assert!(get_reencryption_request(deps.as_mut().storage, &2).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee1_pubkey,
            &proxy3_pubkey,
        )
        .unwrap(),
        2
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy3_pubkey,
        &2,
    ));

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee2, proxy1 - Not removed because it can still be completed
    assert!(get_reencryption_request(deps.as_mut().storage, &3).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee2_pubkey,
            &proxy1_pubkey,
        )
        .unwrap(),
        3
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &3,
    ));

    // DATA1, delegatee2, proxy2 - complete - unaffected
    // Request can still be obtained by delegatee
    assert!(get_reencryption_request(deps.as_mut().storage, &4).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee2_pubkey,
            &proxy2_pubkey,
        )
        .unwrap(),
        4
    );
    // Request is completed - won't appear in proxy tasks
    assert!(!is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &4,
    ));

    // Re-encryption requests with DATA2 to delegatee2
    // DATA2, delegatee2, proxy1 - incomplete and removed
    assert!(get_reencryption_request(deps.as_mut().storage, &5).is_none());
    assert!(get_delegatee_reencryption_request(
        deps.as_mut().storage,
        &data_id2,
        &delegatee2_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(!is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &5,
    ));

    // DATA1, delegatee2, proxy2 - complete - removed because more than threshold fragments cannot be provided
    // Request can still be obtained by delegatee
    assert!(get_reencryption_request(deps.as_mut().storage, &6).is_none());
    assert!(get_delegatee_reencryption_request(
        deps.as_mut().storage,
        &data_id2,
        &delegatee2_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    // Request is completed - won't appear in proxy tasks
    assert!(!is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &6,
    ));

    // Re-encryption requests with DATA3 to delegatee1
    // DATA3, delegatee1, proxy4 - unaffected
    assert!(get_reencryption_request(deps.as_mut().storage, &7).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id3,
            &delegatee1_pubkey,
            &proxy4_pubkey,
        )
        .unwrap(),
        7
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy4_pubkey,
        &7,
    ));

    // DATA3, delegatee1, proxy5 - unaffected
    assert!(get_reencryption_request(deps.as_mut().storage, &8).is_some());
    assert_eq!(
        get_delegatee_reencryption_request(
            deps.as_mut().storage,
            &data_id3,
            &delegatee1_pubkey,
            &proxy5_pubkey,
        )
        .unwrap(),
        8
    );
    assert!(is_proxy_reencryption_request(
        deps.as_mut().storage,
        &proxy5_pubkey,
        &8,
    ));
}

#[test]
fn test_deleting_re_requests_threshold_amount_fragments_provided() {
    let mut deps = mock_dependencies(&[]);

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());

    let delegator = Addr::unchecked("delegator".to_string());

    // Pubkeys
    let delegator_pubkey: String = String::from("DRK");
    let delegatee1_pubkey: String = String::from("DEK1");
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");

    let data_id = String::from("DATA");
    let data_entry = DataEntry {
        delegator_pubkey: delegator_pubkey.clone(),
    };

    /*************** Initialise *************/
    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(1),
        &None,
        &None,
        &Some(proxies.clone()),
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator,
        &data_id,
        &data_entry.delegator_pubkey,
    )
    .is_ok());

    // Add 2 delegations for 2 proxies
    let proxy1_delegation_string = String::from("DS_P1");
    let proxy2_delegation_string = String::from("DS_P2");

    let proxy_delegations: Vec<ProxyDelegation> = vec![
        ProxyDelegation {
            proxy_pubkey: proxy1_pubkey.clone(),
            delegation_string: proxy1_delegation_string.clone(),
        },
        ProxyDelegation {
            proxy_pubkey: proxy2_pubkey.clone(),
            delegation_string: proxy2_delegation_string.clone(),
        },
    ];

    assert!(request_proxies_for_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee1_pubkey,
    )
    .is_ok());
    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee1_pubkey,
        &proxy_delegations,
    )
    .is_ok());

    /*************** Request reencryption by delegator *************/

    assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee1_pubkey).is_ok());

    // Check number of requests
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey).len(),
        1
    );
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy2_pubkey).len(),
        1
    );

    /*************** Provide fragment *************/

    // Proxy1 provides fragment for task1
    let proxy1_fragment1: String = String::from("PR1_FRAG1");
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy1,
        &data_id,
        &delegatee1_pubkey,
        &proxy1_fragment1,
    )
    .is_ok());

    // The threshold is 1 so task gets completed
    // Check numbers of requests
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey).len(),
        0
    );
    assert_eq!(
        get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy2_pubkey).len(),
        0
    );
}
