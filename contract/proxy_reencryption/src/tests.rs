use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};

use crate::contract::{
    execute, get_next_proxy_task, instantiate, DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT,
    DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT, DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT,
};
use crate::delegations::{
    get_delegation_state, get_n_available_proxies_from_delegation, store_add_per_proxy_delegation,
    store_get_delegation, store_get_proxy_delegation_id, store_is_proxy_delegation,
    store_set_delegation, store_set_delegation_id, DelegationState, ProxyDelegation,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, ProxyDelegationString, ProxyTask};
use crate::proxies::{
    store_get_all_active_proxy_pubkeys, store_get_is_proxy_active, store_get_proxy_address,
    store_get_proxy_entry, ProxyState,
};
use crate::reencryption_requests::{
    get_all_fragments, get_reencryption_request_state,
    store_get_all_delegatee_proxy_reencryption_requests,
    store_get_all_proxy_reencryption_requests_in_queue,
    store_get_delegatee_proxy_reencryption_request, store_get_parent_reencryption_request,
    store_get_proxy_reencryption_request, store_is_proxy_reencryption_request_in_queue,
    ReencryptionRequestState,
};
use crate::state::{
    store_get_data_entry, store_get_delegator_address, store_get_state, DataEntry, State,
};

fn mock_env_height(signer: &Addr, height: u64, coins: &Vec<Coin>) -> (Env, MessageInfo) {
    let mut env = mock_env();
    env.block.height = height;
    let info = mock_info(signer.as_str(), &coins);

    return (env, info);
}

fn is_err(result: StdResult<Response>, must_contain: &str) -> bool {
    // Returns true if error message contains specific string
    match result {
        Ok(_) => false,
        Err(err) => match err {
            StdError::GenericErr { msg } => msg.contains(must_contain),
            _ => false,
        },
    }
}

fn init_contract(
    deps: DepsMut,
    creator: &Addr,
    threshold: &Option<u32>,
    admin: &Option<Addr>,
    n_max_proxies: &Option<u32>,
    proxies: &Option<Vec<Addr>>,
    stake_denom: &String,
    minimum_proxy_stake_amount: &Option<Uint128>,
    minimum_request_reward_amount: &Option<Uint128>,
    per_request_slash_stake_amount: &Option<Uint128>,
) -> StdResult<Response> {
    let init_msg = InstantiateMsg {
        threshold: threshold.clone(),
        admin: admin.clone(),
        n_max_proxies: n_max_proxies.clone(),
        proxies: proxies.clone(),
        stake_denom: stake_denom.clone(),
        minimum_proxy_stake_amount: minimum_proxy_stake_amount.clone(),
        minimum_request_reward_amount: minimum_request_reward_amount.clone(),
        per_request_slash_stake_amount: per_request_slash_stake_amount.clone(),
    };
    let env = mock_env_height(&creator, 450, &vec![]);
    return instantiate(deps, env.0, env.1, init_msg);
}

fn add_proxy(deps: DepsMut, creator: &Addr, proxy_addr: &Addr) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &vec![]);

    let msg = ExecuteMsg::AddProxy {
        proxy_addr: proxy_addr.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn remove_proxy(deps: DepsMut, creator: &Addr, proxy_addr: &Addr) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &vec![]);

    let msg = ExecuteMsg::RemoveProxy {
        proxy_addr: proxy_addr.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn register_proxy(
    deps: DepsMut,
    creator: &Addr,
    proxy_pubkey: &String,
    coins: &Vec<Coin>,
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &coins);

    let msg = ExecuteMsg::RegisterProxy {
        proxy_pubkey: proxy_pubkey.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn unregister_proxy(deps: DepsMut, creator: &Addr) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &vec![]);

    let msg = ExecuteMsg::UnregisterProxy {};

    return execute(deps, env.0, env.1, msg);
}

fn deactivate_proxy(deps: DepsMut, creator: &Addr) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &vec![]);

    let msg = ExecuteMsg::DeactivateProxy {};

    return execute(deps, env.0, env.1, msg);
}

fn withdraw_stake(
    deps: DepsMut,
    creator: &Addr,
    stake_amount: &Option<Uint128>,
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &vec![]);

    let msg = ExecuteMsg::WithdrawStake {
        stake_amount: stake_amount.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn add_stake(deps: DepsMut, creator: &Addr, coins: &Vec<Coin>) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &coins);

    let msg = ExecuteMsg::AddStake {};

    return execute(deps, env.0, env.1, msg);
}

fn provide_reencrypted_fragment(
    deps: DepsMut,
    creator: &Addr,
    data_id: &String,
    delegatee_pubkey: &String,
    fragment: &String,
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &vec![]);

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
    let env = mock_env_height(&creator, 450, &vec![]);

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
    proxy_delegations: &[ProxyDelegationString],
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &vec![]);

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
    let env = mock_env_height(&creator, 450, &vec![]);

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
    coins: &Vec<Coin>,
) -> StdResult<Response> {
    let env = mock_env_height(&creator, 450, &coins);

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
        &String::from("atestfet"),
        &None,
        &None,
        &None,
    )
    .is_ok());

    let state: State = store_get_state(&deps.storage).unwrap();
    let available_proxies = store_get_all_active_proxy_pubkeys(&deps.storage);

    assert_eq!(available_proxies.len(), 0);

    assert_eq!(&state.admin, &creator);
    assert_eq!(state.n_max_proxies, u32::MAX);
    assert_eq!(&state.threshold, &1u32);
    assert_eq!(&state.next_proxy_request_id, &0u64);
    assert_eq!(&state.next_delegation_id, &0u64);
}

#[test]
fn test_new_contract_custom_values() {
    let mut deps = mock_dependencies(&[]);
    let creator = Addr::unchecked("creator".to_string());
    let proxy = Addr::unchecked("proxy".to_string());

    let proxies: Vec<Addr> = vec![creator.clone(), proxy.clone()];

    // n_max_proxies cannot be less than threshold
    assert!(is_err(
        init_contract(
            deps.as_mut(),
            &creator,
            &Some(123),
            &Some(proxy.clone()),
            &Some(122),
            &Some(proxies.clone()),
            &String::from("atestfet"),
            &None,
            &None,
            &None,
        ),
        "lower than threshold",
    ));

    // Threshold cannot be zero
    assert!(is_err(
        init_contract(
            deps.as_mut(),
            &creator,
            &Some(0),
            &Some(proxy.clone()),
            &Some(456),
            &Some(proxies.clone()),
            &String::from("atestfet"),
            &None,
            &None,
            &None,
        ),
        "cannot be 0",
    ));

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(123),
        &Some(proxy.clone()),
        &Some(456),
        &Some(proxies.clone()),
        &String::from("atestfet"),
        &None,
        &None,
        &None,
    )
    .is_ok());

    let state: State = store_get_state(&deps.storage).unwrap();
    let available_proxies = store_get_all_active_proxy_pubkeys(&deps.storage);

    assert_eq!(available_proxies.len(), 0);

    assert_eq!(&state.admin, &proxy);
    assert_eq!(&state.n_max_proxies, &456);
    assert_eq!(&state.threshold, &123);
    assert_eq!(&state.next_proxy_request_id, &0u64);
    assert_eq!(&state.next_delegation_id, &0u64);
}

#[test]
fn test_add_remove_proxy() {
    let mut deps = mock_dependencies(&[]);
    let creator = Addr::unchecked("creator".to_string());
    let admin = Addr::unchecked("admin".to_string());
    let proxy = Addr::unchecked("proxy".to_string());
    let proxy_pubkey = String::from("proxy_pubkey");

    let stake_denom = String::from("atestfet");
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &Some(admin.clone()),
        &None,
        &None,
        &stake_denom,
        &None,
        &None,
        &None,
    )
    .is_ok());

    // Only admin can add proxies
    assert!(is_err(
        add_proxy(deps.as_mut(), &creator, &proxy),
        "Only admin",
    ));
    assert!(add_proxy(deps.as_mut(), &admin, &proxy).is_ok());

    // Already added
    assert!(is_err(
        add_proxy(deps.as_mut(), &admin, &proxy),
        "already proxy",
    ));

    // Only admin can remove proxies
    assert!(is_err(
        remove_proxy(deps.as_mut(), &creator, &proxy),
        "Only admin",
    ));

    let remove_proxy_response = remove_proxy(deps.as_mut(), &admin, &proxy).unwrap();
    // When stake is 0 no BankMsg is created
    assert_eq!(remove_proxy_response.messages.len(), 0);

    // Already removed
    assert!(is_err(
        remove_proxy(deps.as_mut(), &admin, &proxy),
        "not a proxy",
    ));

    assert!(add_proxy(deps.as_mut(), &admin, &proxy).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy, &proxy_pubkey, &proxy_stake).is_ok());

    // Check if stake gets returned to proxy
    let remove_proxy_response = remove_proxy(deps.as_mut(), &admin, &proxy).unwrap();
    assert_eq!(
        remove_proxy_response.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT,
                stake_denom.as_str(),
            )],
        })
    );
}

#[test]
fn test_register_unregister_proxy() {
    let mut deps = mock_dependencies(&[]);
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let proxy_pubkey: String = String::from("proxy_pubkey");

    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];

    // Staking
    let stake_denom = String::from("atestfet");
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let insufficient_proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 1),
    }];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &None,
        &Some(proxies.clone()),
        &stake_denom,
        &None,
        &None,
        &None,
    )
    .is_ok());

    assert_eq!(store_get_all_active_proxy_pubkeys(&deps.storage).len(), 0);

    // Check proxy state
    assert!(!store_get_is_proxy_active(
        deps.as_mut().storage,
        &proxy_pubkey
    ));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy_pubkey)
        .unwrap()
        .is_none());
    assert!(store_get_proxy_address(deps.as_mut().storage, &proxy_pubkey).is_none());
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(proxy.state, ProxyState::Authorised);
    assert!(proxy.proxy_pubkey.is_none());

    // Only proxy can add pubkeys
    assert!(is_err(
        register_proxy(deps.as_mut(), &creator, &proxy_pubkey, &proxy_stake),
        "not a proxy",
    ));

    // Insufficient stake amount
    assert!(is_err(
        register_proxy(
            deps.as_mut(),
            &proxy1,
            &proxy_pubkey,
            &insufficient_proxy_stake,
        ),
        "Requires at least 1000 atestfet",
    ));

    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy_pubkey, &proxy_stake).is_ok());
    // Already registered
    assert!(is_err(
        register_proxy(deps.as_mut(), &proxy1, &proxy_pubkey, &proxy_stake),
        "already registered",
    ));

    // Check proxy state
    assert!(store_get_is_proxy_active(
        deps.as_mut().storage,
        &proxy_pubkey
    ));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy_pubkey)
        .unwrap()
        .is_none());
    assert_eq!(
        store_get_proxy_address(deps.as_mut().storage, &proxy_pubkey).unwrap(),
        proxy1
    );
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(proxy.state, ProxyState::Registered);
    assert_eq!(proxy.proxy_pubkey.unwrap(), proxy_pubkey);

    let available_proxy_pubkeys = store_get_all_active_proxy_pubkeys(&deps.storage);
    assert_eq!(available_proxy_pubkeys.len(), 1);
    assert_eq!(&available_proxy_pubkeys, &[proxy_pubkey.clone()]);

    // Register different proxy with existing pubkey
    assert!(is_err(
        register_proxy(deps.as_mut(), &proxy2, &proxy_pubkey, &proxy_stake),
        "Pubkey already used",
    ));

    // Number of available pubkeys remains the same
    let available_proxy_pubkeys = store_get_all_active_proxy_pubkeys(&deps.storage);
    assert_eq!(available_proxy_pubkeys.len(), 1);
    assert_eq!(&available_proxy_pubkeys, &[proxy_pubkey.clone()]);

    // Only proxy can remove pubkeys
    assert!(is_err(
        unregister_proxy(deps.as_mut(), &creator),
        "not a proxy",
    ));

    // Check if stake gets returned to proxy
    let unregister_response = unregister_proxy(deps.as_mut(), &proxy1).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT,
                stake_denom.as_str(),
            )],
        })
    );

    // Already unregistered
    assert!(is_err(
        unregister_proxy(deps.as_mut(), &proxy1),
        "already unregistered",
    ));

    // All proxies unregistered
    assert_eq!(store_get_all_active_proxy_pubkeys(&deps.storage).len(), 0);
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
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &None,
        &None,
        &String::from("atestfet"),
        &None,
        &None,
        &None,
    )
    .is_ok());

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
    assert!(is_err(
        add_data(
            deps.as_mut(),
            &delegator1,
            &data_id1,
            &data_entry.delegator_pubkey,
        ),
        "already exist",
    ));

    assert_eq!(
        &store_get_data_entry(deps.as_mut().storage, &data_id1).unwrap(),
        &data_entry
    );
    assert_eq!(
        store_get_delegator_address(deps.as_mut().storage, &delegator1_pubkey).unwrap(),
        delegator1
    );

    // Delgator2 cannot use delegator1 pubkey
    assert!(is_err(
        add_data(deps.as_mut(), &delegator2, &data_id2, &delegator1_pubkey),
        "already registered with this pubkey",
    ));
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

    // Staking
    let proxy_stake = vec![Coin {
        denom: String::from("atestfet"),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward = vec![Coin {
        denom: String::from("atestfet"),
        amount: Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT * 2),
    }];
    let insufficient_request_reward = vec![Coin {
        denom: String::from("atestfet"),
        amount: Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT - 1),
    }];

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &Some(1),
        &Some(vec![proxy1.clone(), proxy2.clone()]),
        &String::from("atestfet"),
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey, &proxy_stake).is_ok());

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

    let proxy_delegations: Vec<ProxyDelegationString> = vec![ProxyDelegationString {
        proxy_pubkey: proxy1_pubkey.clone(),
        delegation_string: proxy1_delegation_string.clone(),
    }];

    let different_proxy_delegations: Vec<ProxyDelegationString> = vec![ProxyDelegationString {
        proxy_pubkey: proxy2_pubkey.clone(),
        delegation_string: proxy2_delegation_string.clone(),
    }];

    let different_proxy_amount_delegations: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_pubkey: proxy2_pubkey.clone(),
            delegation_string: proxy2_delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_pubkey: proxy2_pubkey.clone(),
            delegation_string: proxy2_delegation_string.clone(),
        },
    ];

    // Reencryption can't be requested yet
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            &data_id,
            &delegatee_pubkey,
            &request_reward,
        ),
        "ProxyDelegation doesn't exist",
    ));

    // Proxies not requested
    assert!(is_err(
        add_delegation(
            deps.as_mut(),
            &delegator1,
            &delegator1_pubkey,
            &delegatee_pubkey,
            &proxy_delegations,
        ),
        "No proxies selected",
    ));

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

    // Delegation already crated
    assert!(is_err(
        request_proxies_for_delegation(
            deps.as_mut(),
            &delegator1,
            &delegator1_pubkey,
            &delegatee_pubkey,
        ),
        "Delegation already exist"
    ));

    // Reencryption can't be requested yet - No delegation strings added
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            &data_id,
            &delegatee_pubkey,
            &request_reward,
        ),
        "Not all delegation strings provided",
    ));

    // Add delegation with different proxy than selected one
    assert!(is_err(
        add_delegation(
            deps.as_mut(),
            &delegator1,
            &delegator1_pubkey,
            &delegatee_pubkey,
            &different_proxy_delegations,
        ),
        "Proxy proxy2_pubkey not selected for delegation.",
    ));

    // Add delegation with different amount of proxies than selected one
    assert!(is_err(
        add_delegation(
            deps.as_mut(),
            &delegator1,
            &delegator1_pubkey,
            &delegatee_pubkey,
            &different_proxy_amount_delegations,
        ),
        "Provided wrong number of delegation strings, expected 1 got 2.",
    ));

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
    assert!(is_err(
        add_delegation(
            deps.as_mut(),
            &delegator1,
            &delegator1_pubkey,
            &delegatee_pubkey,
            &proxy_delegations,
        ),
        "ProxyDelegation strings already provided",
    ));

    // Reencryption cannot be requested by delegator2
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator2,
            &data_id,
            &delegatee_pubkey,
            &request_reward,
        ),
        "Delegator delegator1 already registered with this pubkey.",
    ));

    // Insufficient stake amount
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            &data_id,
            &delegatee_pubkey,
            &insufficient_request_reward,
        ),
        "Requires at least 100 atestfet.",
    ));

    // Reencryption can be requested only after add_delegation
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        &data_id,
        &delegatee_pubkey,
        &request_reward,
    )
    .is_ok());

    // Reencryption already requested
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            &data_id,
            &delegatee_pubkey,
            &request_reward,
        ),
        "Reencryption already requested",
    ));

    // Check if request was created
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id,
            &delegatee_pubkey,
            &proxy1_pubkey,
        ),
        Some(0u64)
    );

    assert_eq!(
        store_get_state(deps.as_mut().storage)
            .unwrap()
            .next_proxy_request_id,
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

    // Staking
    let proxy_stake = vec![Coin {
        denom: String::from("atestfet"),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &Some(1),
        &Some(vec![proxy1.clone(), proxy2.clone()]),
        &String::from("atestfet"),
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey, &proxy_stake).is_ok());

    // Add delegation for proxy
    let proxy1_delegation_string = String::from("DS_P1");

    let proxy_delegations: Vec<ProxyDelegationString> = vec![ProxyDelegationString {
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
    assert!(is_err(
        add_data(deps.as_mut(), &delegator2, &data_id1, &delegator1_pubkey),
        "Delegator delegator1 already registered with this pubkey.",
    ));

    assert!(add_data(deps.as_mut(), &delegator2, &data_id2, &delegator2_pubkey).is_ok());

    // Cannot add delegation by delegator1 using delegator2 pubkey
    assert!(is_err(
        request_proxies_for_delegation(
            deps.as_mut(),
            &delegator1,
            &delegator2_pubkey,
            &delegatee_pubkey,
        ),
        "Delegator delegator2 already registered with this pubkey.",
    ));
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

    // Staking
    let proxy_stake = vec![Coin {
        denom: String::from("atestfet"),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward = vec![Coin {
        denom: String::from("atestfet"),
        amount: Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT * 2),
    }];

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &None,
        &Some(vec![proxy.clone()]),
        &String::from("atestfet"),
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy, &proxy_pubkey, &proxy_stake).is_ok());

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
    let proxy_delegations: Vec<ProxyDelegationString> = vec![ProxyDelegationString {
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
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator,
        &data_id,
        &delegatee_pubkey,
        &request_reward,
    )
    .is_ok());

    /*************** Provide reencrypted fragment *************/
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id,
            &delegatee_pubkey,
            &proxy_pubkey,
        )
        .unwrap(),
        0u64
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy_pubkey,
        &0u64,
    ));

    let proxy_fragment: String = String::from("PR1_FRAG1");
    // Provide unwanted fragment
    assert!(is_err(
        provide_reencrypted_fragment(
            deps.as_mut(),
            &proxy,
            &data_id,
            &other_delegatee_pubkey,
            &proxy_fragment,
        ),
        "This fragment was not requested.",
    ));

    // Not a proxy
    assert!(is_err(
        provide_reencrypted_fragment(
            deps.as_mut(),
            &creator,
            &data_id,
            &delegatee_pubkey,
            &proxy_fragment,
        ),
        "Proxy not registered",
    ));

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
    assert!(is_err(
        provide_reencrypted_fragment(
            deps.as_mut(),
            &proxy,
            &data_id,
            &delegatee_pubkey,
            &proxy_fragment,
        ),
        "Fragment already provided.",
    ));

    // This entry is removed when proxy task is done
    assert!(!store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy_pubkey,
        &0u64,
    ));

    let request = store_get_proxy_reencryption_request(deps.as_mut().storage, &0u64).unwrap();
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

    // Staking
    let stake_denom = String::from("atestfet");
    let proxy_stake = vec![Coin {
        denom: String::from("atestfet"),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward = vec![Coin {
        denom: String::from("atestfet"),
        amount: Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT * 2),
    }];

    /*************** Initialise *************/
    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(2),
        &None,
        &None,
        &Some(proxies.clone()),
        &stake_denom,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey, &proxy_stake).is_ok());

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

    let proxy_delegations: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_pubkey: proxy1_pubkey.clone(),
            delegation_string: proxy1_delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_pubkey: proxy2_pubkey.clone(),
            delegation_string: proxy2_delegation_string.clone(),
        },
    ];

    assert_eq!(
        get_delegation_state(deps.as_mut().storage, &delegator_pubkey, &delegatee1_pubkey),
        DelegationState::NonExisting
    );

    assert!(request_proxies_for_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee1_pubkey,
    )
    .is_ok());

    assert_eq!(
        get_delegation_state(deps.as_mut().storage, &delegator_pubkey, &delegatee1_pubkey),
        DelegationState::WaitingForDelegationStrings
    );

    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        &delegator_pubkey,
        &delegatee1_pubkey,
        &proxy_delegations,
    )
    .is_ok());

    assert_eq!(
        get_delegation_state(deps.as_mut().storage, &delegator_pubkey, &delegatee1_pubkey),
        DelegationState::Active
    );

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

    assert_eq!(
        get_reencryption_request_state(deps.as_mut().storage, &data_id, &delegatee1_pubkey),
        ReencryptionRequestState::Inaccessible
    );

    /*************** Request reencryption by delegator *************/

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator,
        &data_id,
        &delegatee1_pubkey,
        &request_reward,
    )
    .is_ok());
    assert_eq!(
        get_reencryption_request_state(deps.as_mut().storage, &data_id, &delegatee1_pubkey),
        ReencryptionRequestState::Ready
    );

    // Check number of requests
    assert_eq!(
        store_get_all_proxy_reencryption_requests_in_queue(deps.as_mut().storage, &proxy1_pubkey)
            .len(),
        1
    );
    assert_eq!(
        store_get_all_proxy_reencryption_requests_in_queue(deps.as_mut().storage, &proxy2_pubkey)
            .len(),
        1
    );

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator,
        &data_id,
        &delegatee2_pubkey,
        &request_reward,
    )
    .is_ok());

    // Check number of requests
    assert_eq!(
        store_get_all_proxy_reencryption_requests_in_queue(deps.as_mut().storage, &proxy1_pubkey)
            .len(),
        2
    );
    assert_eq!(
        store_get_all_proxy_reencryption_requests_in_queue(deps.as_mut().storage, &proxy2_pubkey)
            .len(),
        2
    );

    /*************** Process reencryption by proxies *************/
    let all_requests =
        store_get_all_proxy_reencryption_requests_in_queue(deps.as_mut().storage, &proxy1_pubkey);
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

    // Check stake before finishing re-encryption
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 2 * DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT
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

    // Check if proxy got reward
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 1 * DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT
            + DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT
    );

    // Check numbers of requests
    assert_eq!(
        store_get_all_proxy_reencryption_requests_in_queue(deps.as_mut().storage, &proxy1_pubkey)
            .len(),
        1
    );
    assert_eq!(
        store_get_all_proxy_reencryption_requests_in_queue(deps.as_mut().storage, &proxy2_pubkey)
            .len(),
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

    // Check if proxy got reward
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + 2 * DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT
    );

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

    assert_eq!(
        get_reencryption_request_state(deps.as_mut().storage, &data_id, &delegatee1_pubkey),
        ReencryptionRequestState::Ready
    );
    assert_eq!(
        get_delegation_state(deps.as_mut().storage, &delegator_pubkey, &delegatee1_pubkey),
        DelegationState::Active
    );

    // Re-encryption was requested in past
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator,
            &data_id,
            &delegatee1_pubkey,
            &request_reward,
        ),
        "Reencryption already requested",
    ));

    // Proxy 2 leaves - all its delegations gets deleted
    assert!(unregister_proxy(deps.as_mut(), &proxy2).is_ok());

    // Check proxy stake amount
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(proxy.stake_amount.u128(), 0);

    // Proxy 2 gets back
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey, &proxy_stake).is_ok());

    // Check proxy stake amount
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT
    );

    assert_eq!(
        get_reencryption_request_state(deps.as_mut().storage, &data_id, &delegatee1_pubkey),
        ReencryptionRequestState::Abandoned
    );
    assert_eq!(
        get_delegation_state(deps.as_mut().storage, &delegator_pubkey, &delegatee1_pubkey),
        DelegationState::NonExisting
    );

    // ProxyDelegation can be re-created again
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

    // Check if all stake gets returned to proxy after un-registered
    let unregister_response = unregister_proxy(deps.as_mut(), &proxy1).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT * 2,
                stake_denom.as_str(),
            )],
        })
    );
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

    // Staking
    let stake_denom = String::from("atestfet");
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward_2_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT * 2),
    }];
    let request_reward_3_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT * 3),
    }];

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
        &stake_denom,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy3, &proxy3_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy4, &proxy4_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy5, &proxy5_pubkey, &proxy_stake).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(deps.as_mut(), &delegator1, &data_id1, &delegator1_pubkey).is_ok());
    assert!(add_data(deps.as_mut(), &delegator1, &data_id2, &delegator1_pubkey).is_ok());
    assert!(add_data(deps.as_mut(), &delegator2, &data_id3, &delegator2_pubkey).is_ok());

    // Add delegations manually

    let delegation1 = ProxyDelegation {
        delegator_pubkey: delegator1_pubkey.clone(),
        delegatee_pubkey: delegatee1_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee1
    store_set_delegation(deps.as_mut().storage, &0, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy1_pubkey,
        &0,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy1_pubkey, &0);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee1
    store_set_delegation(deps.as_mut().storage, &1, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy2_pubkey,
        &1,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy2_pubkey, &1);

    // delegator1 with delegator1_pubkey and proxy3 for delegatee1
    store_set_delegation(deps.as_mut().storage, &2, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy3_pubkey,
        &2,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy3_pubkey, &2);

    let delegation2 = ProxyDelegation {
        delegator_pubkey: delegator1_pubkey.clone(),
        delegatee_pubkey: delegatee2_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee2
    store_set_delegation(deps.as_mut().storage, &3, &delegation2);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy1_pubkey,
        &3,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy1_pubkey, &3);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee2
    store_set_delegation(deps.as_mut().storage, &4, &delegation2);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy2_pubkey,
        &4,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy2_pubkey, &4);

    let delegation3 = ProxyDelegation {
        delegator_pubkey: delegator2_pubkey.clone(),
        delegatee_pubkey: delegatee1_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator2 with delegator2_pubkey and proxy4 for delegatee1
    store_set_delegation(deps.as_mut().storage, &5, &delegation3);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy4_pubkey,
        &5,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy4_pubkey, &5);

    // delegator2 with delegator2_pubkey and proxy5 for delegatee1
    store_set_delegation(deps.as_mut().storage, &6, &delegation3);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy5_pubkey,
        &6,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy5_pubkey, &6);

    // Check proxies stake amount
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT
    );

    // Request re-encryptions

    // Re-encryption requests with DATA1 to delegatee1
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        &data_id1,
        &delegatee1_pubkey,
        &request_reward_3_proxies,
    )
    .is_ok());

    // Check proxy2 stake amount after creating reencryption request
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT
    );

    // Re-encryption requests with DATA1 to delegatee2
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        &data_id1,
        &delegatee2_pubkey,
        &request_reward_2_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA2 to delegatee2
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        &data_id2,
        &delegatee2_pubkey,
        &request_reward_2_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA3 to delegatee1
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator2,
        &data_id3,
        &delegatee1_pubkey,
        &request_reward_2_proxies,
    )
    .is_ok());

    // Check proxy2 stake amount after creating reencryption requests
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 3 * DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT
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

    // Check proxy2 stake amount after finishing one reencryption request
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 2 * DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT
            + DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT
    );

    // Unregister proxy2
    let unregister_response = unregister_proxy(deps.as_mut(), &proxy2).unwrap();

    // Check if stake from unfinished request get returned to delegator
    assert_eq!(
        unregister_response.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: delegator1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT * 3 + DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT,
                stake_denom.as_str(),
            )],
        })
    );

    // Check if stake gets returned to proxy and if proxy got slashed
    assert_eq!(
        unregister_response.messages[1],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy2.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT
                    - 2 * DEFAULT_PER_REQUEST_SLASH_STAKE_AMOUNT,
                stake_denom.as_str(),
            )],
        })
    );

    // Already unregistered
    assert!(is_err(
        unregister_proxy(deps.as_mut(), &proxy2),
        "Proxy already unregistered",
    ));

    // Check state of delegations

    // ProxyDelegation 1 - Removed because of proxy 2
    // delgator1, delegatee1, proxy1 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &0).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &0,
    ));

    // delgator1, delegatee1, proxy2 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &1).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &1,
    ));

    // delgator1, delegatee1, proxy3 - Removed because of proxy 2
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy3_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &2).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy3_pubkey,
        &2,
    ));

    // ProxyDelegation 2 - Number of proxies below threshold, removed entire delegation
    // delgator1, delegatee2, proxy1 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee2_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &3).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &3,
    ));

    // delgator1, delegatee2, proxy2 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee2_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &4).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &4,
    ));

    // ProxyDelegation 3 - Unaffected
    // delgator2, delegatee1, proxy4 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator2_pubkey,
        &delegatee1_pubkey,
        &proxy4_pubkey,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &5).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy4_pubkey,
        &5,
    ));

    // delgator2, delegatee1, proxy5 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator2_pubkey,
        &delegatee1_pubkey,
        &proxy5_pubkey,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &6).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy5_pubkey,
        &6,
    ));

    // Check state of re-encryption requests

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee1, proxy1 - unaffected
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &0).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee1_pubkey,
            &proxy1_pubkey,
        )
        .unwrap(),
        0
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &0,
    ));

    // DATA1, delegatee1, proxy2 - incomplete and removed
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &1).is_none());
    assert!(store_get_delegatee_proxy_reencryption_request(
        deps.as_mut().storage,
        &data_id1,
        &delegatee1_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(!store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &1,
    ));

    // DATA1, delegatee1, proxy3 - unaffected
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &2).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee1_pubkey,
            &proxy3_pubkey,
        )
        .unwrap(),
        2
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy3_pubkey,
        &2,
    ));

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee2, proxy1 - Not removed because it can still be completed
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &3).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee2_pubkey,
            &proxy1_pubkey,
        )
        .unwrap(),
        3
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &3,
    ));

    // DATA1, delegatee2, proxy2 - complete - unaffected
    // Request can still be obtained by delegatee
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &4).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee2_pubkey,
            &proxy2_pubkey,
        )
        .unwrap(),
        4
    );
    // Request is completed - won't appear in proxy tasks
    assert!(!store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &4,
    ));

    // Re-encryption requests with DATA2 to delegatee2
    // DATA2, delegatee2, proxy1 - incomplete and removed
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &5).is_none());
    assert!(store_get_delegatee_proxy_reencryption_request(
        deps.as_mut().storage,
        &data_id2,
        &delegatee2_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(!store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &5,
    ));

    // DATA1, delegatee2, proxy2 - complete - removed because more than threshold fragments cannot be provided
    // Request can still be obtained by delegatee
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &6).is_none());
    assert!(store_get_delegatee_proxy_reencryption_request(
        deps.as_mut().storage,
        &data_id2,
        &delegatee2_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    // Request is completed - won't appear in proxy tasks
    assert!(!store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &6,
    ));

    // Re-encryption requests with DATA3 to delegatee1
    // DATA3, delegatee1, proxy4 - unaffected
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &7).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id3,
            &delegatee1_pubkey,
            &proxy4_pubkey,
        )
        .unwrap(),
        7
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy4_pubkey,
        &7,
    ));

    // DATA3, delegatee1, proxy5 - unaffected
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &8).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id3,
            &delegatee1_pubkey,
            &proxy5_pubkey,
        )
        .unwrap(),
        8
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy5_pubkey,
        &8,
    ));

    // ProxyDelegation 1 was removed with all re-encryption requests
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            &data_id1,
            &delegatee1_pubkey,
            &request_reward_3_proxies,
        ),
        "ProxyDelegation doesn't exist.",
    ));
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

    // Staking
    let stake_denom = String::from("atestfet");
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward_2_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT * 2),
    }];
    let request_reward_3_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_REQUEST_REWARD_AMOUNT * 3),
    }];

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
        &stake_denom,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy3, &proxy3_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy4, &proxy4_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy5, &proxy5_pubkey, &proxy_stake).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(deps.as_mut(), &delegator1, &data_id1, &delegator1_pubkey).is_ok());
    assert!(add_data(deps.as_mut(), &delegator1, &data_id2, &delegator1_pubkey).is_ok());
    assert!(add_data(deps.as_mut(), &delegator2, &data_id3, &delegator2_pubkey).is_ok());

    // Add delegations manually

    let delegation1 = ProxyDelegation {
        delegator_pubkey: delegator1_pubkey.clone(),
        delegatee_pubkey: delegatee1_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee1
    store_set_delegation(deps.as_mut().storage, &0, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy1_pubkey,
        &0,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy1_pubkey, &0);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee1
    store_set_delegation(deps.as_mut().storage, &1, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy2_pubkey,
        &1,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy2_pubkey, &1);

    // delegator1 with delegator1_pubkey and proxy3 for delegatee1
    store_set_delegation(deps.as_mut().storage, &2, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy3_pubkey,
        &2,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy3_pubkey, &2);

    let delegation2 = ProxyDelegation {
        delegator_pubkey: delegator1_pubkey.clone(),
        delegatee_pubkey: delegatee2_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee2
    store_set_delegation(deps.as_mut().storage, &3, &delegation2);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy1_pubkey,
        &3,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy1_pubkey, &3);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee2
    store_set_delegation(deps.as_mut().storage, &4, &delegation2);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy2_pubkey,
        &4,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy2_pubkey, &4);

    let delegation3 = ProxyDelegation {
        delegator_pubkey: delegator2_pubkey.clone(),
        delegatee_pubkey: delegatee1_pubkey.clone(),
        delegation_string: Some(delegation_string.clone()),
    };

    // delegator2 with delegator2_pubkey and proxy4 for delegatee1
    store_set_delegation(deps.as_mut().storage, &5, &delegation3);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy4_pubkey,
        &5,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy4_pubkey, &5);

    // delegator2 with delegator2_pubkey and proxy5 for delegatee1
    store_set_delegation(deps.as_mut().storage, &6, &delegation3);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy5_pubkey,
        &6,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy5_pubkey, &6);

    // Request re-encryptions

    // Re-encryption requests with DATA1 to delegatee1
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        &data_id1,
        &delegatee1_pubkey,
        &request_reward_3_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA1 to delegatee2
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        &data_id1,
        &delegatee2_pubkey,
        &request_reward_2_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA2 to delegatee2
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        &data_id2,
        &delegatee2_pubkey,
        &request_reward_2_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA3 to delegatee1
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator2,
        &data_id3,
        &delegatee1_pubkey,
        &request_reward_2_proxies,
    )
    .is_ok());

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
    assert!(store_get_is_proxy_active(
        deps.as_mut().storage,
        &proxy2_pubkey
    ));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey)
        .unwrap()
        .is_some());
    assert_eq!(
        store_get_proxy_address(deps.as_mut().storage, &proxy2_pubkey).unwrap(),
        proxy2
    );
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(proxy.state, ProxyState::Registered);
    assert_eq!(proxy.proxy_pubkey.unwrap(), proxy2_pubkey);

    // Sender is not a proxy
    assert!(is_err(
        deactivate_proxy(deps.as_mut(), &creator),
        "Sender is not a proxy",
    ));

    // Deactivate proxy2
    assert!(deactivate_proxy(deps.as_mut(), &proxy2).is_ok());
    // Already deactivated
    assert!(is_err(
        deactivate_proxy(deps.as_mut(), &proxy2),
        "Proxy already deactivated",
    ));

    // Check proxy state
    assert!(!store_get_is_proxy_active(
        deps.as_mut().storage,
        &proxy2_pubkey
    ));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey)
        .unwrap()
        .is_some());
    assert_eq!(
        store_get_proxy_address(deps.as_mut().storage, &proxy2_pubkey).unwrap(),
        proxy2
    );
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(proxy.state, ProxyState::Leaving);
    assert_eq!(proxy.proxy_pubkey.unwrap(), proxy2_pubkey);

    // Check state of delegations

    // ProxyDelegation 1 - Removed because of proxy 2
    // delgator1, delegatee1, proxy1 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &0).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &0,
    ));

    // delgator1, delegatee1, proxy2 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &1).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &1,
    ));

    // delgator1, delegatee1, proxy3 - Removed because of proxy 2
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy3_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &2).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy3_pubkey,
        &2,
    ));

    // ProxyDelegation 2 - Number of proxies below threshold, removed entire delegation
    // delgator1, delegatee2, proxy1 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee2_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &3).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &3,
    ));

    // delgator1, delegatee2, proxy2 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator1_pubkey,
        &delegatee2_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &4).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &4,
    ));

    // ProxyDelegation 3 - Unaffected
    // delgator2, delegatee1, proxy4 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator2_pubkey,
        &delegatee1_pubkey,
        &proxy4_pubkey,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &5).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy4_pubkey,
        &5,
    ));

    // delgator2, delegatee1, proxy5 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &delegator2_pubkey,
        &delegatee1_pubkey,
        &proxy5_pubkey,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &6).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy5_pubkey,
        &6,
    ));

    // Remove proxy by admin
    assert!(remove_proxy(deps.as_mut(), &creator, &proxy2).is_ok());
    // Already removed
    assert!(is_err(
        remove_proxy(deps.as_mut(), &creator, &proxy2),
        "proxy_2 is not a proxy",
    ));

    // Check proxy state
    assert!(!store_get_is_proxy_active(
        deps.as_mut().storage,
        &proxy2_pubkey
    ));
    assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey)
        .unwrap()
        .is_none());
    assert!(store_get_proxy_address(deps.as_mut().storage, &proxy2_pubkey).is_none());
    assert!(store_get_proxy_entry(deps.as_mut().storage, &proxy2).is_none());

    // Check state of re-encryption requests

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee1, proxy1 - unaffected
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &0).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee1_pubkey,
            &proxy1_pubkey,
        )
        .unwrap(),
        0
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &0,
    ));

    // DATA1, delegatee1, proxy2 - incomplete and removed
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &1).is_none());
    assert!(store_get_delegatee_proxy_reencryption_request(
        deps.as_mut().storage,
        &data_id1,
        &delegatee1_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    assert!(!store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &1,
    ));

    // DATA1, delegatee1, proxy3 - unaffected
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &2).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee1_pubkey,
            &proxy3_pubkey,
        )
        .unwrap(),
        2
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy3_pubkey,
        &2,
    ));

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee2, proxy1 - Not removed because it can still be completed
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &3).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee2_pubkey,
            &proxy1_pubkey,
        )
        .unwrap(),
        3
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &3,
    ));

    // DATA1, delegatee2, proxy2 - complete - unaffected
    // Request can still be obtained by delegatee
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &4).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id1,
            &delegatee2_pubkey,
            &proxy2_pubkey,
        )
        .unwrap(),
        4
    );
    // Request is completed - won't appear in proxy tasks
    assert!(!store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &4,
    ));

    // Re-encryption requests with DATA2 to delegatee2
    // DATA2, delegatee2, proxy1 - incomplete and removed
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &5).is_none());
    assert!(store_get_delegatee_proxy_reencryption_request(
        deps.as_mut().storage,
        &data_id2,
        &delegatee2_pubkey,
        &proxy1_pubkey,
    )
    .is_none());
    assert!(!store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy1_pubkey,
        &5,
    ));

    // DATA1, delegatee2, proxy2 - complete - removed because more than threshold fragments cannot be provided
    // Request can still be obtained by delegatee
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &6).is_none());
    assert!(store_get_delegatee_proxy_reencryption_request(
        deps.as_mut().storage,
        &data_id2,
        &delegatee2_pubkey,
        &proxy2_pubkey,
    )
    .is_none());
    // Request is completed - won't appear in proxy tasks
    assert!(!store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy2_pubkey,
        &6,
    ));

    // Re-encryption requests with DATA3 to delegatee1
    // DATA3, delegatee1, proxy4 - unaffected
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &7).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id3,
            &delegatee1_pubkey,
            &proxy4_pubkey,
        )
        .unwrap(),
        7
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy4_pubkey,
        &7,
    ));

    // DATA3, delegatee1, proxy5 - unaffected
    assert!(store_get_proxy_reencryption_request(deps.as_mut().storage, &8).is_some());
    assert_eq!(
        store_get_delegatee_proxy_reencryption_request(
            deps.as_mut().storage,
            &data_id3,
            &delegatee1_pubkey,
            &proxy5_pubkey,
        )
        .unwrap(),
        8
    );
    assert!(store_is_proxy_reencryption_request_in_queue(
        deps.as_mut().storage,
        &proxy5_pubkey,
        &8,
    ));
}

#[test]
fn test_proxy_stake_withdrawal() {
    let mut deps = mock_dependencies(&[]);
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let proxy1_pubkey: String = String::from("proxy1_pubkey");
    let proxy2_pubkey: String = String::from("proxy2_pubkey");

    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];

    // Staking
    let stake_denom = String::from("atestfet");
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + 50),
    }];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &None,
        &Some(proxies.clone()),
        &stake_denom,
        &None,
        &None,
        &None,
    )
    .is_ok());

    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey, &proxy_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey, &proxy_stake).is_ok());

    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + 50
    );

    // Proxy1 is trying to withdraw more than is available gives you maximum available stake
    let withdraw_res = withdraw_stake(deps.as_mut(), &proxy1, &Some(Uint128::new(51))).unwrap();
    assert_eq!(
        withdraw_res.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(50, stake_denom.as_str())],
        })
    );

    // All stake of proxy1 withdrawn
    assert!(is_err(
        withdraw_stake(deps.as_mut(), &proxy1, &None),
        "Not enough stake to withdraw",
    ));

    // Proxy2 is trying to withdraw maximum available stake
    let withdraw_res = withdraw_stake(deps.as_mut(), &proxy2, &None).unwrap();
    assert_eq!(
        withdraw_res.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy2.to_string(),
            amount: vec![Coin::new(50, stake_denom.as_str())],
        })
    );

    // All stake of proxy2 withdrawn
    assert!(is_err(
        withdraw_stake(deps.as_mut(), &proxy2, &None),
        "Not enough stake to withdraw",
    ));

    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT
    );

    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT
    );

    // Insufficient stake
    assert!(is_err(
        withdraw_stake(deps.as_mut(), &proxy1, &Some(Uint128::new(1))),
        "Not enough stake to withdraw",
    ));

    // Remaining stake can be withdrawn only by unregistering
    let unregister_res = unregister_proxy(deps.as_mut(), &proxy1).unwrap();
    assert_eq!(
        unregister_res.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT,
                stake_denom.as_str(),
            )],
        })
    );

    // Proxy unregistered
    assert!(is_err(
        withdraw_stake(deps.as_mut(), &proxy1, &Some(Uint128::new(1))),
        "Not enough stake to withdraw",
    ));
}

#[test]
fn test_proxy_add_stake() {
    let mut deps = mock_dependencies(&[]);
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());

    let proxy1_pubkey: String = String::from("proxy1_pubkey");

    let proxies: Vec<Addr> = vec![proxy1.clone()];

    // Staking
    let stake_denom = String::from("atestfet");
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];

    let proxy_additional_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(50),
    }];

    let proxy_wrong_coins_additional_stake = vec![
        Coin {
            denom: stake_denom.clone(),
            amount: Uint128::new(50),
        },
        Coin {
            denom: String::from("othercoin"),
            amount: Uint128::new(20),
        },
    ];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &None,
        &None,
        &None,
        &Some(proxies.clone()),
        &stake_denom,
        &None,
        &None,
        &None,
    )
    .is_ok());

    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey, &proxy_stake).is_ok());

    assert!(is_err(
        add_stake(deps.as_mut(), &proxy1, &proxy_wrong_coins_additional_stake),
        "Expected 1 Coin with denom atestfet"
    ));
    assert!(add_stake(deps.as_mut(), &proxy1, &proxy_additional_stake).is_ok());

    // Check if withdrawn stake amount
    let unregister_res = unregister_proxy(deps.as_mut(), &proxy1).unwrap();
    assert_eq!(
        unregister_res.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + 50,
                stake_denom.as_str(),
            )],
        })
    );
}

#[test]
fn test_proxy_insufficient_funds_request_skip() {
    let mut deps = mock_dependencies(&[]);

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());
    let proxy3 = Addr::unchecked("proxy_3".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());

    // Pubkeys
    let delegator1_pubkey: String = String::from("DRK1");

    let delegatee1_pubkey: String = String::from("DEK1");

    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");
    let data_id3 = String::from("DATA3");

    let delegation_string = String::from("DELESTRING");

    // Staking
    let stake_denom = String::from("atestfet");
    let minimum_proxy_stake_amount: u128 = 100;
    let minimum_request_reward_amount: u128 = 99;
    let per_request_slash_stake_amount: u128 = 98;

    let proxy1_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(minimum_proxy_stake_amount),
    }];

    let proxy2_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(2 * minimum_proxy_stake_amount),
    }];

    let proxy3_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(3 * minimum_proxy_stake_amount),
    }];

    let request_reward_1_proxy = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(minimum_request_reward_amount * 1),
    }];
    let request_reward_2_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(minimum_request_reward_amount * 2),
    }];
    let request_reward_3_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(minimum_request_reward_amount * 3),
    }];

    /*************** Initialise *************/
    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone(), proxy3.clone()];
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        &Some(2),
        &None,
        &None,
        &Some(proxies.clone()),
        &stake_denom,
        &Some(Uint128::new(minimum_proxy_stake_amount)),
        &Some(Uint128::new(minimum_request_reward_amount)),
        &Some(Uint128::new(per_request_slash_stake_amount)),
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(deps.as_mut(), &proxy1, &proxy1_pubkey, &proxy1_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy2, &proxy2_pubkey, &proxy2_stake).is_ok());
    assert!(register_proxy(deps.as_mut(), &proxy3, &proxy3_pubkey, &proxy3_stake).is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(deps.as_mut(), &delegator1, &data_id1, &delegator1_pubkey,).is_ok());
    assert!(add_data(deps.as_mut(), &delegator1, &data_id2, &delegator1_pubkey,).is_ok());
    assert!(add_data(deps.as_mut(), &delegator1, &data_id3, &delegator1_pubkey,).is_ok());

    assert!(request_proxies_for_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee1_pubkey
    )
    .is_ok());

    let proxy_delegations: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_pubkey: proxy1_pubkey.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_pubkey: proxy2_pubkey.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_pubkey: proxy3_pubkey.clone(),
            delegation_string: delegation_string.clone(),
        },
    ];

    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        &delegator1_pubkey,
        &delegatee1_pubkey,
        &proxy_delegations
    )
    .is_ok());

    // Request first reencryption -- All proxies has enough stake
    assert_eq!(
        get_n_available_proxies_from_delegation(
            deps.as_mut().storage,
            &delegator1_pubkey,
            &delegatee1_pubkey,
            &per_request_slash_stake_amount
        ),
        3
    );
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        &data_id1,
        &delegatee1_pubkey,
        &request_reward_3_proxies
    )
    .is_ok());

    let parent_request1 =
        store_get_parent_reencryption_request(deps.as_mut().storage, &data_id1, &delegatee1_pubkey)
            .unwrap();
    assert_eq!(parent_request1.n_proxy_requests, 3);
    assert_eq!(
        store_get_proxy_entry(deps.as_mut().storage, &proxy2)
            .unwrap()
            .stake_amount
            .u128(),
        102
    );

    // Request second reencryption -- 1 proxies skipped because of insufficient funds
    assert_eq!(
        get_n_available_proxies_from_delegation(
            deps.as_mut().storage,
            &delegator1_pubkey,
            &delegatee1_pubkey,
            &per_request_slash_stake_amount
        ),
        2
    );
    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &delegator1_pubkey,
            &delegatee1_pubkey
        ),
        DelegationState::Active
    );

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        &data_id2,
        &delegatee1_pubkey,
        &request_reward_2_proxies
    )
    .is_ok());

    let parent_request1 =
        store_get_parent_reencryption_request(deps.as_mut().storage, &data_id2, &delegatee1_pubkey)
            .unwrap();
    assert_eq!(parent_request1.n_proxy_requests, 2);

    // Request third reencryption request fails -- 2 proxies skipped because of insufficient funds
    assert_eq!(
        get_n_available_proxies_from_delegation(
            deps.as_mut().storage,
            &delegator1_pubkey,
            &delegatee1_pubkey,
            &per_request_slash_stake_amount
        ),
        1
    );

    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &delegator1_pubkey,
            &delegatee1_pubkey
        ),
        DelegationState::ProxiesAreBusy
    );
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            &data_id2,
            &delegatee1_pubkey,
            &request_reward_1_proxy
        ),
        "Proxies are too busy, try again later. Available 1 proxies out of 3, threshold is 2"
    ));

    // Requests:
    // req1 - (0)proxy1, (1)proxy2, (2)proxy3
    // req2 - (3)proxy2, (4)proxy3

    // Check proxy2 unregister
    // Proxy3 gets slashed for 2 unfinished requests - 1 portion is returned to delegator, other portion is stored in parent request
    let unregister_response = unregister_proxy(deps.as_mut(), &proxy2).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: delegator1.to_string(),
            amount: vec![Coin::new(
                per_request_slash_stake_amount + minimum_request_reward_amount * 3,
                stake_denom.as_str(),
            )],
        })
    );
    assert_eq!(
        unregister_response.messages[1],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy2.to_string(),
            amount: vec![Coin::new(
                2 * minimum_proxy_stake_amount - 2 * per_request_slash_stake_amount,
                stake_denom.as_str(),
            )],
        })
    );

    // Requests:
    // req1 - (0)proxy1, (2)proxy3
    // req2 - deleted

    // Check if only request2 is deleted
    assert_eq!(
        store_get_all_delegatee_proxy_reencryption_requests(
            deps.as_mut().storage,
            &data_id1,
            &delegatee1_pubkey
        )
        .len(),
        2
    );
    assert!(store_get_parent_reencryption_request(
        deps.as_mut().storage,
        &data_id1,
        &delegatee1_pubkey
    )
    .is_some());

    assert!(store_get_all_delegatee_proxy_reencryption_requests(
        deps.as_mut().storage,
        &data_id2,
        &delegatee1_pubkey
    )
    .is_empty());
    assert_eq!(
        store_get_parent_reencryption_request(deps.as_mut().storage, &data_id2, &delegatee1_pubkey)
            .unwrap()
            .state,
        ReencryptionRequestState::Abandoned
    );

    // Check if 1 portion of slashed stake is stored in parent request
    assert_eq!(
        store_get_parent_reencryption_request(deps.as_mut().storage, &data_id1, &delegatee1_pubkey)
            .unwrap()
            .slashed_stake_amount
            .u128(),
        per_request_slash_stake_amount
    );

    // Check proxy3 unregister
    // Proxy3 gets slashed for 1 unfinished request, delegator is repaid from proxy3 slashed amount + amount stored in parent request
    let unregister_response = unregister_proxy(deps.as_mut(), &proxy3).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: delegator1.to_string(),
            amount: vec![Coin::new(
                per_request_slash_stake_amount * 2 + minimum_request_reward_amount * 2,
                stake_denom.as_str(),
            )],
        })
    );
    assert_eq!(
        unregister_response.messages[1],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy3.to_string(),
            amount: vec![Coin::new(
                3 * minimum_proxy_stake_amount - per_request_slash_stake_amount,
                stake_denom.as_str(),
            )],
        })
    );

    // Requests:
    // req1 - deleted
    // req2 - deleted

    // Check if all requests are deleted
    assert!(store_get_all_delegatee_proxy_reencryption_requests(
        deps.as_mut().storage,
        &data_id1,
        &delegatee1_pubkey
    )
    .is_empty());
    assert_eq!(
        store_get_parent_reencryption_request(deps.as_mut().storage, &data_id1, &delegatee1_pubkey)
            .unwrap()
            .state,
        ReencryptionRequestState::Abandoned
    );

    assert!(store_get_all_delegatee_proxy_reencryption_requests(
        deps.as_mut().storage,
        &data_id2,
        &delegatee1_pubkey
    )
    .is_empty());
    assert_eq!(
        store_get_parent_reencryption_request(deps.as_mut().storage, &data_id2, &delegatee1_pubkey)
            .unwrap()
            .state,
        ReencryptionRequestState::Abandoned
    );

    // Check proxy1 unregister
    // Proxy1 gets all initial stake back
    let unregister_response = unregister_proxy(deps.as_mut(), &proxy1).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        CosmosMsg::Bank(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(minimum_proxy_stake_amount, stake_denom.as_str(),)],
        })
    );
}
