use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Env, Addr, Response, StdResult, MessageInfo, DepsMut, Uint128, Uint64, Coin};

use crate::contract::{execute, instantiate, get_next_proxy_task, get_all_fragments};
use crate::msg::{ExecuteMsg, InstantiateMsg, ProxyDelegation, ProxyTask};
use crate::state::{get_state, State, get_all_proxies, DataEntry, HashID, get_data_entry, get_all_available_proxy_pubkeys, get_delegatee_reencryption_request, get_reencryption_request, get_all_proxy_reencryption_requests, is_proxy_reencryption_request};

fn mock_env_height(signer: &Addr, height: u64, coins: &Vec<Coin>) -> (Env, MessageInfo) {
    let mut env = mock_env();
    env.block.height = height;
    let info = mock_info(signer.as_str(), &coins);

    return (env, info);
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
    minimum_request_stake_amount: &Option<Uint128>,
    request_timeout_height: &Option<Uint64>,
) -> StdResult<Response>
{
    let init_msg = InstantiateMsg {
        threshold: threshold.clone(),
        admin: admin.clone(),
        n_max_proxies: n_max_proxies.clone(),
        proxies: proxies.clone(),
        stake_denom: stake_denom.clone(),
        minimum_proxy_stake_amount: minimum_proxy_stake_amount.clone(),
        minimum_request_stake_amount: minimum_request_stake_amount.clone(),
        request_timeout_height: request_timeout_height.clone(),
    };
    let env = mock_env_height(&creator, 450, &vec!());
    return instantiate(deps, env.0, env.1, init_msg);
}

fn add_proxy(
    deps: DepsMut,
    creator: &Addr,
    proxy_addr: &Addr) -> StdResult<Response>
{
    let env = mock_env_height(&creator, 450, &vec!());

    let msg = ExecuteMsg::AddProxy {
        proxy_addr: proxy_addr.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn remove_proxy(
    deps: DepsMut,
    creator: &Addr,
    proxy_addr: &Addr) -> StdResult<Response>
{
    let env = mock_env_height(&creator, 450, &vec!());

    let msg = ExecuteMsg::RemoveProxy {
        proxy_addr: proxy_addr.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}

fn register_proxy(
    deps: DepsMut,
    creator: &Addr,
    stake: &Vec<Coin>,
    proxy_pubkey: &String) -> StdResult<Response>
{
    let env = mock_env_height(&creator, 450, &stake);

    let msg = ExecuteMsg::RegisterProxy { proxy_pubkey: proxy_pubkey.clone() };

    return execute(deps, env.0, env.1, msg);
}

fn unregister_proxy(
    deps: DepsMut,
    creator: &Addr) -> StdResult<Response>
{
    let env = mock_env_height(&creator, 450, &vec!());

    let msg = ExecuteMsg::UnregisterProxy {};

    return execute(deps, env.0, env.1, msg);
}

fn provide_reencrypted_fragment(
    deps: DepsMut,
    creator: &Addr,
    data_id: &HashID,
    delegatee_pubkey: &String,
    fragment: &HashID) -> StdResult<Response>
{
    let env = mock_env_height(&creator, 450, &vec!());

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
    data_id: &HashID,
    delegator_pubkey: &String) -> StdResult<Response>
{
    let env = mock_env_height(&creator, 450, &vec!());

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
    proxy_delegations: &Vec<ProxyDelegation>) -> StdResult<Response>
{
    let env = mock_env_height(&creator, 450, &vec!());

    let msg = ExecuteMsg::AddDelegation {
        delegator_pubkey: delegator_pubkey.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
        proxy_delegations: proxy_delegations.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}


fn request_proxies_for_delegation(
    deps: DepsMut,
    creator: &Addr,
    delegator_pubkey: &String,
    delegatee_pubkey: &String) -> StdResult<Response>
{
    let env = mock_env_height(&creator, 450, &vec!());

    let msg = ExecuteMsg::RequestProxiesForDelegation {
        delegator_pubkey: delegator_pubkey.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}


fn request_reencryption(
    deps: DepsMut,
    creator: &Addr,
    data_id: &HashID,
    delegatee_pubkey: &String) -> StdResult<Response>
{
    let env = mock_env_height(&creator, 450, &vec!());

    let msg = ExecuteMsg::RequestReencryption {
        data_id: data_id.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
    };

    return execute(deps, env.0, env.1, msg);
}


mod init {
    use super::*;

    #[test]
    fn test_new_contract() {
        let mut deps = mock_dependencies(&[]);
        let creator = Addr::unchecked("creator".to_string());
        let proxy = Addr::unchecked("proxy".to_string());
        let denom = "atestfet".to_string();

        let proxies: Vec<Addr> = vec![creator.clone(), proxy.clone()];

        assert!(init_contract(deps.as_mut(), &creator, &None, &None, &None, &Some(proxies.clone()), &denom, &None, &None, &None).is_ok());


        let state: State = get_state(&deps.storage).unwrap();
        let available_proxies = get_all_available_proxy_pubkeys(&deps.storage);
        let all_proxies = get_all_proxies(&deps.storage);

        assert_eq!(available_proxies.len(), 0);
        assert_eq!(all_proxies.len(), 2);
        assert_eq!(&all_proxies, &proxies);

        assert_eq!(&state.admin, &creator);
        assert_eq!(state.n_max_proxies, u32::MAX);
        assert_eq!(&state.threshold, &1u32);
        assert_eq!(&state.next_request_id, &0u64);
    }

    #[test]
    fn test_add_remove_proxy() {
        let mut deps = mock_dependencies(&[]);
        let creator = Addr::unchecked("creator".to_string());
        let admin = Addr::unchecked("admin".to_string());
        let proxy = Addr::unchecked("proxy".to_string());
        let denom = "atestfet".to_string();

        assert!(init_contract(deps.as_mut(), &creator, &None, &Some(admin.clone()), &None, &None, &denom, &None, &None, &None).is_ok());

        let all_proxies = get_all_proxies(&deps.storage);
        assert_eq!(all_proxies.len(), 0);

        // Only admin can add proxies
        assert!(add_proxy(deps.as_mut(), &creator, &proxy).is_err());
        assert!(add_proxy(deps.as_mut(), &admin, &proxy).is_ok());

        let all_proxies = get_all_proxies(&deps.storage);
        assert_eq!(all_proxies.len(), 1);
        assert_eq!(&all_proxies[0], &proxy);

        // Only admin can remove proxies
        assert!(remove_proxy(deps.as_mut(), &creator, &proxy).is_err());
        assert!(remove_proxy(deps.as_mut(), &admin, &proxy).is_ok());

        let all_proxies = get_all_proxies(&deps.storage);
        assert_eq!(all_proxies.len(), 0);
    }

    #[test]
    fn test_register_unregister_proxy() {
        let mut deps = mock_dependencies(&[]);
        let creator = Addr::unchecked("creator".to_string());
        let proxy1 = Addr::unchecked("proxy1".to_string());
        let proxy2 = Addr::unchecked("proxy2".to_string());
        let denom = "atestfet".to_string();
        let minimum_stake = 100u128;
        let min_stake_coins = vec![Coin::new(minimum_stake, denom.clone())];
        let less_than_min_stake_coins = vec![Coin::new(minimum_stake-1, denom.clone())];

        let proxy_pubkey: String = String::from("proxy_pubkey");

        let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];

        assert!(init_contract(deps.as_mut(), &creator, &None, &None, &None, &Some(proxies.clone()), &denom, &Some(Uint128::new(minimum_stake)), &Some(Uint128::new(minimum_stake)), &None).is_ok());

        assert_eq!(get_all_available_proxy_pubkeys(&deps.storage).len(), 0);

        // Only proxy can add pubkeys
        assert!(register_proxy(deps.as_mut(), &creator, &min_stake_coins, &proxy_pubkey).is_err());

        // Not enough stake
        assert!(register_proxy(deps.as_mut(), &proxy1, &less_than_min_stake_coins, &proxy_pubkey).is_err());

        assert!(register_proxy(deps.as_mut(), &proxy1, &min_stake_coins, &proxy_pubkey).is_ok());
        // Already registered
        assert!(register_proxy(deps.as_mut(), &proxy1, &min_stake_coins, &proxy_pubkey).is_err());

        let available_proxy_pubkeys = get_all_available_proxy_pubkeys(&deps.storage);
        assert_eq!(available_proxy_pubkeys.len(), 1);
        assert_eq!(&available_proxy_pubkeys, &[proxy_pubkey.clone()]);

        // Register different proxy with existing pubkey
        assert!(register_proxy(deps.as_mut(), &proxy2, &min_stake_coins, &proxy_pubkey).is_err());

        // Number of available pubkeys remains the same
        let available_proxy_pubkeys = get_all_available_proxy_pubkeys(&deps.storage);
        assert_eq!(available_proxy_pubkeys.len(), 1);
        assert_eq!(&available_proxy_pubkeys, &[proxy_pubkey.clone()]);

        // Only proxy can remove pubkeys
        assert!(unregister_proxy(deps.as_mut(), &creator).is_err());
        assert!(unregister_proxy(deps.as_mut(), &proxy1).is_ok());
        // Already unregistered
        assert!(unregister_proxy(deps.as_mut(), &proxy1).is_err());

        // All proxies unregistered
        assert_eq!(get_all_available_proxy_pubkeys(&deps.storage).len(), 0);
    }

    #[test]
    fn test_add_data() {
        let mut deps = mock_dependencies(&[]);
        let denom = "atestfet".to_string();

        // Addresses
        let creator = Addr::unchecked("creator".to_string());
        let delegator = Addr::unchecked("delegator".to_string());

        // Pubkeys
        let delegator_pubkey: String = String::from("DRK");

        let data_id = String::from("DATA");
        let data_entry = DataEntry {
            delegator_pubkey: delegator_pubkey.clone(),
            delegator_addr: delegator.clone(),
        };

        /*************** Initialise *************/
        assert!(init_contract(deps.as_mut(), &creator, &None, &None, &None, &None, &denom, &None, &None, &None).is_ok());

        /*************** Add data and delegations by delegator *************/
        // Add data by delegator
        assert!(add_data(deps.as_mut(), &delegator, &data_id, &data_entry.delegator_pubkey).is_ok());

        assert_eq!(&get_data_entry(deps.as_mut().storage, &data_id).unwrap(), &data_entry);
    }


    #[test]
    fn test_select_proxies_add_delegation_and_request_reencryption() {
        let mut deps = mock_dependencies(&[]);
        let denom = "atestfet".to_string();
        let minimum_stake = 100u128;
        let min_stake_coins = vec![Coin::new(minimum_stake, denom.clone())];

        // Addresses
        let creator = Addr::unchecked("creator".to_string());
        let proxy1 = Addr::unchecked("proxy1".to_string());
        let proxy2 = Addr::unchecked("proxy2".to_string());

        let delegator = Addr::unchecked("delegator".to_string());

        // Pubkeys
        let delegator_pubkey: String = String::from("DRK");
        let delegatee_pubkey: String = String::from("DEK1");
        let proxy1_pubkey: String = String::from("proxy1_pubkey");
        let proxy2_pubkey: String = String::from("proxy2_pubkey");

        let data_id = String::from("DATA");
        let data_entry = DataEntry {
            delegator_pubkey: delegator_pubkey.clone(),
            delegator_addr: delegator.clone(),
        };

        /*************** Initialise *************/
        assert!(init_contract(deps.as_mut(), &creator, &None, &None, &Some(1), &Some(vec![proxy1.clone(), proxy2.clone()]), &denom, &Some(Uint128::new(minimum_stake)), &Some(Uint128::new(minimum_stake)), &None).is_ok());

        /*************** Register proxies *************/
        // Proxies register -> submits pubkeys
        assert!(register_proxy(deps.as_mut(), &proxy1, &min_stake_coins, &proxy1_pubkey).is_ok());
        assert!(register_proxy(deps.as_mut(), &proxy2, &min_stake_coins, &proxy2_pubkey).is_ok());


        /*************** Add data and delegations by delegator *************/
        // Add data by delegator
        assert!(add_data(deps.as_mut(), &delegator, &data_id, &data_entry.delegator_pubkey).is_ok());

        // Add delegation for proxy
        let proxy1_delegation_string = String::from("DS_P1");
        let proxy2_delegation_string = String::from("DS_P2");

        let proxy_delegations: Vec<ProxyDelegation> = vec![
            ProxyDelegation { proxy_pubkey: proxy1_pubkey.clone(), delegation_string: proxy1_delegation_string.clone() },
        ];

        let different_proxy_delegations: Vec<ProxyDelegation> = vec![
            ProxyDelegation { proxy_pubkey: proxy2_pubkey.clone(), delegation_string: proxy2_delegation_string.clone() },
        ];


        // Reencryption can't be requested yet
        assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee_pubkey).is_err());


        // Proxies not requested
        assert!(add_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee_pubkey, &proxy_delegations).is_err());

        let res = request_proxies_for_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee_pubkey).unwrap();
        // Check if proxy 1 was selected
        assert_eq!(format!("[\"{}\", ]", proxy1_pubkey), res.attributes[4].value);


        // Reencryption can't be requested yet - No delegation strings added
        assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee_pubkey).is_err());


        // Add delegation with different proxy than selected one
        assert!(add_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee_pubkey, &different_proxy_delegations).is_err());

        // Add delegation
        assert!(add_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee_pubkey, &proxy_delegations).is_ok());

        // Cannot add same delegation twice
        assert!(add_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee_pubkey, &proxy_delegations).is_err());

        // Reencryption can be requested only after add_delegation
        assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee_pubkey).is_ok());

        // Reencryption already requested
        assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee_pubkey).is_err());

        // Check if request was created
        assert_eq!(get_delegatee_reencryption_request(deps.as_mut().storage, &data_id, &delegatee_pubkey, &proxy1_pubkey), Some(0u64));

        assert_eq!(get_state(deps.as_mut().storage).unwrap().next_request_id, 1u64);
    }


    #[test]
    fn test_provide_reencrypted_fragment() {
        let mut deps = mock_dependencies(&[]);
        let denom = "atestfet".to_string();
        let minimum_stake = 100u128;
        let min_stake_coins = vec![Coin::new(minimum_stake, denom.clone())];

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
            delegator_addr: delegator.clone(),
        };

        /*************** Initialise *************/
        assert!(init_contract(deps.as_mut(), &creator, &None, &None, &None, &Some(vec![proxy.clone()]), &denom, &Some(Uint128::new(minimum_stake)), &Some(Uint128::new(minimum_stake)), &None).is_ok());

        /*************** Register proxies *************/
        // Proxies register -> submits pubkeys
        assert!(register_proxy(deps.as_mut(), &proxy, &min_stake_coins, &proxy_pubkey).is_ok());


        /*************** Add data and delegations by delegator *************/
        // Add data by delegator
        assert!(add_data(deps.as_mut(), &delegator, &data_id, &data_entry.delegator_pubkey).is_ok());

        // Add delegation for proxy
        let proxy_delegation_string = String::from("DS_P1");
        let proxy_delegations: Vec<ProxyDelegation> = vec![
            ProxyDelegation { proxy_pubkey: proxy_pubkey.clone(), delegation_string: proxy_delegation_string.clone() },
        ];

        assert!(request_proxies_for_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee_pubkey).is_ok());

        assert!(add_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee_pubkey, &proxy_delegations).is_ok());

        /*************** Request re-encryption *************/
        assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee_pubkey).is_ok());

        /*************** Provide reencrypted fragment *************/
        assert_eq!(get_delegatee_reencryption_request(deps.as_mut().storage, &data_id, &delegatee_pubkey, &proxy_pubkey).unwrap(), 0u64);
        assert!(is_proxy_reencryption_request(deps.as_mut().storage, &proxy_pubkey, &0u64));

        let proxy_fragment: HashID = String::from("PR1_FRAG1");
        // Provide unwanted fragment
        assert!(provide_reencrypted_fragment(deps.as_mut(), &proxy, &data_id, &other_delegatee_pubkey, &proxy_fragment).is_err());
        // Provide fragment correctly
        assert!(provide_reencrypted_fragment(deps.as_mut(), &proxy, &data_id, &delegatee_pubkey, &proxy_fragment).is_ok());
        // Fragment already provided
        assert!(provide_reencrypted_fragment(deps.as_mut(), &proxy, &data_id, &delegatee_pubkey, &proxy_fragment).is_err());

        // This entry is removed when proxy task is done
        assert!(!is_proxy_reencryption_request(deps.as_mut().storage, &proxy_pubkey, &0u64));

        let request = get_reencryption_request(deps.as_mut().storage, &0u64).unwrap();
        assert_eq!(request.fragment, Some(proxy_fragment));
    }

    #[test]
    fn test_contract_lifecycle() {
        let mut deps = mock_dependencies(&[]);
        let denom = "atestfet".to_string();
        let minimum_stake = 100u128;
        let min_stake_coins = vec![Coin::new(minimum_stake, denom.clone())];

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
            delegator_addr: delegator.clone(),
        };

        /*************** Initialise *************/
        let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];
        assert!(init_contract(deps.as_mut(),
                              &creator,
                              &None,
                              &None,
                              &None,
                              &Some(proxies.clone()),
                              &denom,
                              &Some(Uint128::new(minimum_stake)),
                              &Some(Uint128::new(minimum_stake)),
                              &None).is_ok());

        /*************** Register proxies *************/
        // Proxies register -> submits pubkeys
        assert!(register_proxy(deps.as_mut(), &proxy1, &min_stake_coins, &proxy1_pubkey).is_ok());
        assert!(register_proxy(deps.as_mut(), &proxy2, &min_stake_coins, &proxy2_pubkey).is_ok());


        /*************** Add data and delegations by delegator *************/
        // Add data by delegator
        assert!(add_data(deps.as_mut(), &delegator, &data_id, &data_entry.delegator_pubkey).is_ok());

        // Add 2 delegations for 2 proxies
        let proxy1_delegation_string = String::from("DS_P1");
        let proxy2_delegation_string = String::from("DS_P2");

        let proxy_delegations: Vec<ProxyDelegation> = vec![
            ProxyDelegation { proxy_pubkey: proxy1_pubkey.clone(), delegation_string: proxy1_delegation_string.clone() },
            ProxyDelegation { proxy_pubkey: proxy2_pubkey.clone(), delegation_string: proxy2_delegation_string.clone() }
        ];

        assert!(request_proxies_for_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee1_pubkey).is_ok());
        assert!(add_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee1_pubkey, &proxy_delegations).is_ok());

        assert!(request_proxies_for_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee2_pubkey).is_ok());
        assert!(add_delegation(deps.as_mut(), &delegator, &delegator_pubkey, &delegatee2_pubkey, &proxy_delegations).is_ok());

        // No tasks yet
        assert!(get_next_proxy_task(deps.as_mut().storage, &proxy1_pubkey).unwrap().is_none());
        assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey).unwrap().is_none());


        /*************** Request reencryption by delegator *************/

        assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee1_pubkey).is_ok());

        // Check number of requests
        assert_eq!(get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey).len(), 1);
        assert_eq!(get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy2_pubkey).len(), 1);


        assert!(request_reencryption(deps.as_mut(), &delegator, &data_id, &delegatee2_pubkey).is_ok());


        // Check number of requests
        assert_eq!(get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey).len(), 2);
        assert_eq!(get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy2_pubkey).len(), 2);


        /*************** Process reencryption by proxies *************/
        let all_requests = get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey);
        assert_eq!(all_requests.len(), 2);

        // Check if proxy got task 1
        let proxy1_task1 = get_next_proxy_task(deps.as_mut().storage, &proxy1_pubkey).unwrap().unwrap();
        assert_eq!(proxy1_task1, ProxyTask
        {
            data_id: data_id.clone(),
            delegatee_pubkey: delegatee1_pubkey.clone(),
            delegator_pubkey: delegator_pubkey.clone(),
            delegation_string: proxy1_delegation_string.clone(),
        });

        // Proxy1 provides fragment for task1
        let proxy1_fragment1: HashID = String::from("PR1_FRAG1");
        assert!(provide_reencrypted_fragment(deps.as_mut(), &proxy1, &data_id, &delegatee1_pubkey, &proxy1_fragment1).is_ok());

        // Check numbers of requests
        assert_eq!(get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy1_pubkey).len(), 1);
        assert_eq!(get_all_proxy_reencryption_requests(deps.as_mut().storage, &proxy2_pubkey).len(), 2);

        // Check available fragments
        assert_eq!(
            get_all_fragments(deps.as_mut().storage, &data_id, &delegatee1_pubkey),
            vec![proxy1_fragment1.clone()]);
        assert_eq!(get_all_fragments(deps.as_mut().storage, &data_id, &delegatee2_pubkey).len(), 0);


        // Check if proxy got task 2
        let proxy1_task2 = get_next_proxy_task(deps.as_mut().storage, &proxy1_pubkey).unwrap().unwrap();
        assert_eq!(proxy1_task2, ProxyTask
        {
            data_id: data_id.clone(),
            delegatee_pubkey: delegatee2_pubkey.clone(),
            delegator_pubkey: delegator_pubkey.clone(),
            delegation_string: proxy1_delegation_string.clone(),
        });

        // Proxy1 provides fragment for task1
        let proxy1_fragment2: HashID = String::from("PR1_FRAG2");
        assert!(provide_reencrypted_fragment(deps.as_mut(), &proxy1, &data_id, &delegatee2_pubkey, &proxy1_fragment2).is_ok());

        // All tasks completed for proxy1
        assert!(get_next_proxy_task(deps.as_mut().storage, &proxy1_pubkey).unwrap().is_none());
        // But not for proxy2
        assert!(get_next_proxy_task(deps.as_mut().storage, &proxy2_pubkey).unwrap().is_some());

        // Check available fragments
        assert_eq!(
            get_all_fragments(deps.as_mut().storage, &data_id, &delegatee1_pubkey),
            vec![proxy1_fragment1]);
        assert_eq!(
            get_all_fragments(deps.as_mut().storage, &data_id, &delegatee2_pubkey),
            vec![proxy1_fragment2]);
    }
}