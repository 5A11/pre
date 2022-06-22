use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    Addr, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdError, StdResult, SubMsg, Uint128,
};

use crate::contract::{
    execute, get_proxies_availability, get_proxy_tasks, instantiate,
    DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT, DEFAULT_PER_TASK_SLASH_STAKE_AMOUNT,
    DEFAULT_TASK_REWARD_AMOUNT,
};

//use crate::contract::verify_fragment;
use crate::delegations::{
    get_delegation_state, get_n_available_proxies_from_delegation,
    get_n_minimum_proxies_for_refund, store_add_per_proxy_delegation, store_get_delegation,
    store_get_proxy_delegation_id, store_is_proxy_delegation, store_set_delegation,
    store_set_delegation_id, DelegationState, ProxyDelegation,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, ProxyDelegationString, ProxyTaskResponse};
use crate::proxies::{
    store_get_all_active_proxy_addresses, store_get_is_proxy_active, store_get_proxy_entry, Proxy,
    ProxyState,
};
use crate::reencryption_requests::{
    get_all_fragments, get_reencryption_request_state, store_get_all_proxy_tasks_in_queue,
    store_get_delegatee_proxy_task, store_get_proxy_task, store_is_proxy_task_in_queue,
    ReencryptionRequestState,
};
use crate::state::{
    store_get_data_entry, store_get_delegator_address, store_get_state, store_get_timeouts_config,
    DataEntry, StakingConfig, State,
};

// Test constants
const DEFAULT_STAKE_DENOM: &str = "atestfet";

const DELEGATOR1_PUBKEY: &str = "ApEPhAeq+TAL5aKiRkIpdoJ2pD+6qSt1RqHxGthT+XRY";
const DELEGATOR2_PUBKEY: &str = "AutXYFCdptyMx71HRDCyUm1eNDZVAFfDlbWTJQydiCBl";

const DELEGATEE1_PUBKEY: &str = "A5CYTfwD0EocpW4gCKtnP1lIFkMveO55v5+nbJaLqmLX";
const DELEGATEE2_PUBKEY: &str = "AnJgbnA4RwLalI9Vi2xQSTVD7MRGeb4kiRQ8/kqzJyIK";

const CAPSULE: &str = "Ax83HFfEW1e+DW3KlikFLELPOVqYnlS39baHHC+/vsB4AmV+m1r9eZ6nCV9KXv7dSH+bSdWFbsqWFTxfF5qsjwObLtgsZUVSt8iv8UtkP0bLJs2sguElu4Syek6Seh3ZTj4=";

const FRAGMENT_P1_DR1_DE1: &str = "Agn8MTBWSKzz277FLeNKvhOwa3juw7HBciLmyA/3kZ2hAtQv0l/B+Ej2vQLxZDx+MHDr5uevth9PzntoIz6gbPI1xJk3dVwZohs3YgdaXJsBXpAambF1FpOGrola7KcwjtQDOL6tYr3e6dlMgsW9GnONyZUWk15ixjxdrAIZfp8qWAMCbOd9fCO820cnEqBeQHpit75l8gxb6Al3s28p4uMFeq4Dzsh5SbQgRk7KjI9LEq2a9YzQ2ts3O5KEx3SuZoCOE0UDns625ayBRPD5BHdYwGaCGo/w6oJ5PvRp7rEpMSvxpOACu5HXcj2KNZnzAc2QGNrHmrAIxxS4pUbp7ffoPjSK/eGOs3Yh2IaeLQMzj2FNpUCYii6D3KJMT5sWqdKQV+5Aw6ebgujLY0o4Gs2aJ3toE3GuNuSfwFKzySmpq5CfSGaJJftZDYt72g7t8cRKVFXT6D8ugCXfMVL6GRE7adJEkYU=";
const FRAGMENT_P2_DR1_DE1: &str = "ArsxPns0fAKMt28wRjja8VL39o1qB2KMQNDAn6FFPslEAlEo4UNoSUXoxfDSkROjb1wuO4V6WqKA006R2W84g1asBaLS9NN4UFdAH7fLOr+SsyA7eMZABzLCp8r/jlS3P+4DOL6tYr3e6dlMgsW9GnONyZUWk15ixjxdrAIZfp8qWAMChJzdKNOmu/YU9gc7wJqJh00pojqHNp0c259JpteHu7sCSY6RGI61WSoded9gCnw2RjmgC7SGSLi1l00zku+09bMC89+mYLBp6Kn59mV2e/Kc3GtlZoA0ZEa8SEkCmwGqzfcCtpmg7uxd38Uylf6XXbA0bg4AVvWY5WtuD/oW0dCgYSacnjktP9RjQVzwEDvb0TSId67E80BSkdmX0g1mBmvYBy2MbnhFpC1BUC6PMPs5wSa+Mo8o3DP7kMJ1gHTqJYcJBGfgG4DtfcnjjOuJugZEloo+67P9mYzVFVXCJHTrldQ=";

const FRAGMENT_P1_DR1_DE2: &str = "A5fWxYyjkfJu/k2oq5A6w+pLgRtWRIKu2uEHe/i0AGSiAsK7jq0a7KjTeiBCBRTC64ATDb/QfYQ9CBoiF5FDdbJwh3Yb5RoZgkclP0cqNtftZnRCdVUuycy2UpQ7f4x5tFkCkymfOr+pOYAe56kPnK9cTGjuwGdgcrV3i5A1ocF5xYsD0mFd9APeYHeRjAIgPzM3na8xJuYgSdY9FA6upZOYqxkDGWHWjB6Uvjby3zTbN+A8vuQmHRx0NST4ICR5HfKCCPQCE6/D/Dep4lyf4v9E03VgMisZKFWW7+YP5qAIWgNDSPoCb8OqfDTrYzJSKGZg+ti4l/Cjo5PaqmlZlj1MCR/Rb906nywBtIQCU9iVAHGUnE8h9QV+kYWik4s2Vcq2W6r/Y6MXnYLK9JsYpIwny6zDwHzOwfTk4Wn9mLGENf4q2s/5ZeM/1TJIu6wECI+L5zmGMWl40iloHhHcxeCVKwKnM0s=";
const FRAGMENT_P2_DR1_DE2: &str = "AjLk43WgIexBQf0ABO3E6hd2BVZ1HCBrJQ0c8uRgclpgAlB2ijedmrkT1QNYSQ1oPVHE09/8uDGEyovAi4SmJDzKNv9py/jaCqGACwFebtl8knFosOcLok6K4+zM5opfBT4CkymfOr+pOYAe56kPnK9cTGjuwGdgcrV3i5A1ocF5xYsCZrz/src1xaM+pmvzfcEmgIcaDvoIn9Aht2WcWp2tVS8Dp//y8AxVoATdPzGMQYbXoC6CDEukMH3AP5BpfcteLcQDF5V+A0Jp7a2rbHhEE2nRjN1P0ADRf/8C4K5RC+UPMb4CaVYIM8PXJ9mIChdZqvwsnl9SytT8HMjuEYK4AO5HSDoNGFNeBeckZorwm15pgVRPozsNXA9cKaQLD1zAWTqTUi4hOciWZrQd9UIAwPh+UCLNYPUp/Br+G05KHPr2no5iIZuhd/ssq8XK9DHz03vsTSuOg4bB2YI7YgumrFKR4Yc=";

//const FRAGMENT_P1_DR2_DE1: &str = "At8lxfrtHYMW7Ggucuj1nlYZbYxFZw+gWncOpTEBP5m7ApS5Vtc8LyT1AxrFUX160D2Et199se837PzqLH9lh5fJlymgRquhUhz2UuANM9C3qlu6KXFBmRCWk20r3EuW7mUDImFyRCX5EEulxrzCUdlyW4WkaXYEoRhInWu/ZZBaC/UD77G9xuQvoqzw7UKus1E3CWBxNMtOO1XYDIgWphy71RwCPVcwIH3W7/blsds2AaATSCquOSgm6aXoi6zBm7BeH6ICPGL+pzZ0hgeLxwTEd8KDVAIJy8LJBjjt+BzGPa0DCrwCxrHiIEDQmEk5+QygrAnHjz0mGzI/iE3FFIk9vesZ5MAp3y1M66+9nXSdn3kAYgiVCt28+zBTzviEdaTVWeefii7+gh+rgG1j0xCO6KYIaa7t7mIBF8SMVgRILsAyx42lBSHr+kUjBGChjzDoTHDWOBEnB3XcTlzoFC3VufpZPsQ=";
//const FRAGMENT_P2_DR2_DE1: &str = "AtdmQFFcyKtslY4ocZnuIqpSisE2IqZanCWtOiCA/T5zA1MJ79pbT8xCnFjSOfTIbi6yRckRlH0Vy2qQ/PI+D9CdJgJ8dzdVmpvp0EgvjJIdc6TIsNnAPH9urI8Z2j4FL5YDImFyRCX5EEulxrzCUdlyW4WkaXYEoRhInWu/ZZBaC/UDcKATTVzEGtLGhAmqtJTbb1DFoDwBaq9oCd+uwvAqKjQCiURQGGkTwvYEVYNa+IRYi4IWLLE9BswFJLUO0kRCkdMDZ9ALbgi7PTgxER0Qut1zeWpSvIgD+ebHLDSFgUMv3KYDAp+CzqPK/NCqWmSc+Ystk3RKAqthQGXfBUHineHz+zS67gFeeLDmMYhmt+knBcoGUrNIKJW6vjYc0NiahjueM5AHc6kPIeXCePTrqDvqJttVWBB8bNo82HWvLrIMBNZ3D7MZDA43yeOumoemYrsLZlLNCQ6gb7MX1Se23mGIYMY=";

//const FRAGMENT_P1_DR2_DE2: &str = "AuYwuHs4w5F2thK/JDQ6WOBXAnxRKhzv53Lt8P/mJzaVA2ew70g/IdqDfKtLGnjQHWnA0kmBPo6TRq4lotajsC6vbe0QaAelnniHEpbYLP+g7CA6Dz58YAgx8OFL17xNJJgDrVSv5azExD+hZ7PMWE6JAkArWG452RjvQ9x9Quyl/YMC4MjqT6t3Puj9+oJvdfNpaxcUkQXx7tVix1B3NALOKG8C7xt8mwSl1wlxmgvqwf9c2EOEqNVXnm8b8vevmc3zdf4Db/9atkgY94bRt1/YLq0Qv8pS6yG6lQ6Z5TQ8hFYadigDOU6W1V8wAA9mIJn9Jo1lk3APS/u+GNLm03SAWsUgCb9svLYmK8UKJ5g3RlLrsdpFhxJc9AS41kIwd4JvQPFPjpLAjOFdIcoP+o7Q1LC9xj+DpmJWwPAtr6AvHhMGsSLfCjN9zDU7SKgYCFvWyv+l+8CT/heCCrfhDcNhUYiN6YI=";
//const FRAGMENT_P2_DR2_DE2: &str = "A7cqB41cE3itmN+sjCfhOkoCNIADq6QnlD0UyR8+hfeSA9rbkfY2weSSR8uahlWN7wXYejcN3SLuFNJqo60b4AbeWw8qARl55NWUyy/c/WWgYQGX3n6MEmA8Cy4hV2ginYoDrVSv5azExD+hZ7PMWE6JAkArWG452RjvQ9x9Quyl/YMCVKNFtLT3ZdukPOpedQvo0U5+KQm1m60keBXdHORdCwsDndeZL8061hMG8u8dVYhJwKDZBkQ5DW2fGisAxB/v5ysDe2qkKVZKdUPpoRrHFa5oGeZF+wFcu+fD8s2QNDW7QuMCzUkvs7F6N86KfY4HHQSfAk3bixe6WXyaRVTn7GhXqIECW5hY3VSviIqA5UgF9NZOKlqpw1GHQQpEtSefQWVa6op4BjYVbzKMWaAlwxAQYhv/0bmi9FmS4MkMnJ/vbBUtGyXEEpj3Bx5P7oXjzVIL2kJN7ZRFiNhnYDE2Igf3naU=";

const DEFAULT_BLOCK_HEIGHT: u64 = 100;

fn mock_env_height(signer: &Addr, height: u64, coins: &Vec<Coin>) -> (Env, MessageInfo) {
    let mut env = mock_env();

    env.block.height = height;
    let info = mock_info(signer.as_str(), coins);

    (env, info)
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
    block_height: u64,
    threshold: &Option<u32>,
    admin: &Option<Addr>,
    proxies: &Option<Vec<Addr>>,
    stake_denom: &String,
    minimum_proxy_stake_amount: &Option<Uint128>,
    per_proxy_task_reward_amount: &Option<Uint128>,
    per_task_slash_stake_amount: &Option<Uint128>,
    timeout_height: &Option<u64>,
    proxy_whitelisting: &Option<bool>,
) -> StdResult<Response> {
    let init_msg = InstantiateMsg {
        threshold: *threshold,
        admin: admin.clone(),
        proxies: proxies.clone(),
        stake_denom: stake_denom.clone(),
        minimum_proxy_stake_amount: *minimum_proxy_stake_amount,
        per_proxy_task_reward_amount: *per_proxy_task_reward_amount,
        per_task_slash_stake_amount: *per_task_slash_stake_amount,
        timeout_height: *timeout_height,
        proxy_whitelisting: *proxy_whitelisting,
    };
    let env = mock_env_height(creator, block_height, &vec![]);
    instantiate(deps, env.0, env.1, init_msg)
}

fn add_proxy(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    proxy_addr: &Addr,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::AddProxy {
        proxy_addr: proxy_addr.clone(),
    };

    execute(deps, env.0, env.1, msg)
}

fn terminate_contract(deps: DepsMut, creator: &Addr, block_height: u64) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::TerminateContract {};

    execute(deps, env.0, env.1, msg)
}

fn withdraw_contract(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    recipient_addr: &Addr,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::WithdrawContract {
        recipient_addr: recipient_addr.clone(),
    };

    execute(deps, env.0, env.1, msg)
}

fn remove_proxy(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    proxy_addr: &Addr,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::RemoveProxy {
        proxy_addr: proxy_addr.clone(),
    };

    execute(deps, env.0, env.1, msg)
}

fn register_proxy(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    proxy_pubkey: &String,
    coins: &Vec<Coin>,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, coins);

    let msg = ExecuteMsg::RegisterProxy {
        proxy_pubkey: proxy_pubkey.clone(),
    };

    execute(deps, env.0, env.1, msg)
}

fn unregister_proxy(deps: DepsMut, creator: &Addr, block_height: u64) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::UnregisterProxy {};

    execute(deps, env.0, env.1, msg)
}

fn deactivate_proxy(deps: DepsMut, creator: &Addr, block_height: u64) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::DeactivateProxy {};

    execute(deps, env.0, env.1, msg)
}

fn withdraw_stake(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    stake_amount: &Option<Uint128>,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::WithdrawStake {
        stake_amount: *stake_amount,
    };

    execute(deps, env.0, env.1, msg)
}

fn add_stake(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    coins: &Vec<Coin>,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, coins);

    let msg = ExecuteMsg::AddStake {};

    execute(deps, env.0, env.1, msg)
}

fn provide_reencrypted_fragment(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    data_id: &String,
    delegatee_pubkey: &String,
    fragment: &String,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::ProvideReencryptedFragment {
        data_id: data_id.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
        fragment: fragment.clone(),
    };

    execute(deps, env.0, env.1, msg)
}

fn skip_reencryption_task(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    data_id: &String,
    delegatee_pubkey: &String,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::SkipReencryptionTask {
        data_id: data_id.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
    };

    execute(deps, env.0, env.1, msg)
}

fn add_data(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    data_id: &String,
    delegator_pubkey: &String,
    capsule: &String,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::AddData {
        data_id: data_id.clone(),
        delegator_pubkey: delegator_pubkey.clone(),
        capsule: capsule.clone(),
        tags: None,
    };

    execute(deps, env.0, env.1, msg)
}

fn remove_data(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    data_id: &String,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::RemoveData {
        data_id: data_id.clone(),
    };

    execute(deps, env.0, env.1, msg)
}

fn add_delegation(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    delegator_pubkey: &String,
    delegatee_pubkey: &String,
    proxy_delegations: &[ProxyDelegationString],
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, &vec![]);

    let msg = ExecuteMsg::AddDelegation {
        delegator_pubkey: delegator_pubkey.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
        proxy_delegations: proxy_delegations.to_vec(),
    };

    execute(deps, env.0, env.1, msg)
}

fn request_reencryption(
    deps: DepsMut,
    creator: &Addr,
    block_height: u64,
    data_id: &String,
    delegatee_pubkey: &String,
    coins: &Vec<Coin>,
) -> StdResult<Response> {
    let env = mock_env_height(creator, block_height, coins);

    let msg = ExecuteMsg::RequestReencryption {
        data_id: data_id.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
    };

    execute(deps, env.0, env.1, msg)
}

#[test]
fn test_new_contract_default_values() {
    let mut deps = mock_dependencies();
    let creator = Addr::unchecked("creator".to_string());
    let proxy = Addr::unchecked("proxy".to_string());

    let proxies: Vec<Addr> = vec![creator.clone(), proxy];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &Some(proxies),
        &DEFAULT_STAKE_DENOM.to_string(),
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    let state: State = store_get_state(&deps.storage).unwrap();
    let available_proxies = store_get_all_active_proxy_addresses(&deps.storage);

    assert_eq!(available_proxies.len(), 0);

    assert_eq!(&state.admin, &creator);
    assert_eq!(&state.threshold, &1u32);
    assert_eq!(&state.next_proxy_task_id, &0u64);
    assert_eq!(&state.next_delegation_id, &0u64);
}

#[test]
fn test_new_contract_custom_values() {
    let mut deps = mock_dependencies();
    let creator = Addr::unchecked("creator".to_string());
    let proxy = Addr::unchecked("proxy".to_string());

    let proxies: Vec<Addr> = vec![creator.clone(), proxy.clone()];

    // Threshold cannot be zero
    assert!(is_err(
        init_contract(
            deps.as_mut(),
            &creator,
            DEFAULT_BLOCK_HEIGHT,
            &Some(0),
            &Some(proxy.clone()),
            &Some(proxies.clone()),
            &DEFAULT_STAKE_DENOM.to_string(),
            &None,
            &None,
            &None,
            &None,
            &None,
        ),
        "cannot be 0",
    ));

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &Some(123),
        &Some(proxy.clone()),
        &Some(proxies),
        &DEFAULT_STAKE_DENOM.to_string(),
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    let state: State = store_get_state(&deps.storage).unwrap();
    let available_proxies = store_get_all_active_proxy_addresses(&deps.storage);

    assert_eq!(available_proxies.len(), 0);

    assert_eq!(&state.admin, &proxy);
    assert_eq!(&state.threshold, &123);
    assert_eq!(&state.next_proxy_task_id, &0u64);
    assert_eq!(&state.next_delegation_id, &0u64);
}

#[test]
fn test_add_remove_proxy() {
    let mut deps = mock_dependencies();
    let creator = Addr::unchecked("creator".to_string());
    let admin = Addr::unchecked("admin".to_string());
    let proxy = Addr::unchecked("proxy".to_string());
    let proxy_pubkey = String::from("proxy_pubkey");

    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &Some(admin.clone()),
        &None,
        &stake_denom,
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    // Only admin can add proxies
    assert!(is_err(
        add_proxy(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT, &proxy),
        "Only admin",
    ));
    assert!(add_proxy(deps.as_mut(), &admin, DEFAULT_BLOCK_HEIGHT, &proxy).is_ok());

    // Already added
    assert!(is_err(
        add_proxy(deps.as_mut(), &admin, DEFAULT_BLOCK_HEIGHT, &proxy),
        "already proxy",
    ));

    // Only admin can remove proxies
    assert!(is_err(
        remove_proxy(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT, &proxy),
        "Only admin",
    ));

    let remove_proxy_response =
        remove_proxy(deps.as_mut(), &admin, DEFAULT_BLOCK_HEIGHT, &proxy).unwrap();
    // When stake is 0 no BankMsg is created
    assert_eq!(remove_proxy_response.messages.len(), 0);

    // Already removed
    assert!(is_err(
        remove_proxy(deps.as_mut(), &admin, DEFAULT_BLOCK_HEIGHT, &proxy),
        "not a proxy",
    ));

    assert!(add_proxy(deps.as_mut(), &admin, DEFAULT_BLOCK_HEIGHT, &proxy).is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy,
        DEFAULT_BLOCK_HEIGHT,
        &proxy_pubkey,
        &proxy_stake,
    )
    .is_ok());

    // Check if stake gets returned to proxy
    let remove_proxy_response =
        remove_proxy(deps.as_mut(), &admin, DEFAULT_BLOCK_HEIGHT, &proxy).unwrap();
    assert_eq!(
        remove_proxy_response.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT,
                stake_denom.as_str(),
            )],
        })
    );
}

#[test]
fn test_register_unregister_proxy_whitelisting() {
    let mut deps = mock_dependencies();
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let proxy_pubkey: String = String::from("proxy_pubkey");

    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
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
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &Some(proxies),
        &stake_denom,
        &None,
        &None,
        &None,
        &None,
        &Some(true),
    )
    .is_ok());

    assert_eq!(store_get_all_active_proxy_addresses(&deps.storage).len(), 0);

    // Check proxy state
    assert!(!store_get_is_proxy_active(deps.as_mut().storage, &proxy1,));
    assert!(get_proxy_tasks(deps.as_mut().storage, &proxy1, &0)
        .unwrap()
        .is_empty());
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(proxy.state, ProxyState::Authorised);
    assert!(proxy.proxy_pubkey.is_none());

    // Only proxy can add pubkeys
    assert!(is_err(
        register_proxy(
            deps.as_mut(),
            &creator,
            DEFAULT_BLOCK_HEIGHT,
            &proxy_pubkey,
            &proxy_stake,
        ),
        "not a proxy",
    ));

    // Insufficient stake amount
    assert!(is_err(
        register_proxy(
            deps.as_mut(),
            &proxy1,
            DEFAULT_BLOCK_HEIGHT,
            &proxy_pubkey,
            &insufficient_proxy_stake,
        ),
        "Requires at least 1000 atestfet",
    ));

    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy_pubkey,
        &proxy_stake,
    )
    .is_ok());
    // Already registered
    assert!(is_err(
        register_proxy(
            deps.as_mut(),
            &proxy1,
            DEFAULT_BLOCK_HEIGHT,
            &proxy_pubkey,
            &proxy_stake,
        ),
        "already registered",
    ));

    // Check proxy state
    assert!(store_get_is_proxy_active(deps.as_mut().storage, &proxy1,));
    assert!(
        get_proxy_tasks(deps.as_mut().storage, &proxy1, &DEFAULT_BLOCK_HEIGHT)
            .unwrap()
            .is_empty()
    );
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(proxy.state, ProxyState::Registered);
    assert_eq!(proxy.proxy_pubkey.unwrap(), proxy_pubkey);

    let available_proxy_addresses = store_get_all_active_proxy_addresses(&deps.storage);
    assert_eq!(available_proxy_addresses.len(), 1);
    assert_eq!(&available_proxy_addresses, &[proxy1.clone()]);

    // Number of available pubkeys remains the same
    let available_proxy_addresses = store_get_all_active_proxy_addresses(&deps.storage);
    assert_eq!(available_proxy_addresses.len(), 1);
    assert_eq!(&available_proxy_addresses, &[proxy1.clone()]);

    // Only proxy can remove pubkeys
    assert!(is_err(
        unregister_proxy(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT),
        "not a proxy",
    ));

    // Check if stake gets returned to proxy
    let unregister_response =
        unregister_proxy(deps.as_mut(), &proxy1, DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT,
                stake_denom.as_str(),
            )],
        })
    );

    // Already unregistered
    assert!(is_err(
        unregister_proxy(deps.as_mut(), &proxy1, DEFAULT_BLOCK_HEIGHT),
        "already unregistered",
    ));

    // All proxies unregistered
    assert_eq!(store_get_all_active_proxy_addresses(&deps.storage).len(), 0);
}

#[test]
fn test_register_unregister_proxy_no_whitelisting() {
    // Test register, unregister and deactivate behaviour
    let mut deps = mock_dependencies();
    let creator = Addr::unchecked("creator".to_string());
    let proxy = Addr::unchecked("proxy1".to_string());

    let proxy_pubkey: String = String::from("proxy_pubkey");
    let proxy_pubkey2: String = String::from("proxy_pubkey2");

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &None,
        &stake_denom,
        &None,
        &None,
        &None,
        &None,
        &Some(false),
    )
    .is_ok());

    assert!(is_err(
        deactivate_proxy(deps.as_mut(), &proxy, DEFAULT_BLOCK_HEIGHT),
        "Sender is not a proxy",
    ));

    // Test if proxy can register when whitelisting is off
    assert!(register_proxy(
        deps.as_mut(),
        &proxy,
        DEFAULT_BLOCK_HEIGHT,
        &proxy_pubkey,
        &proxy_stake,
    )
    .is_ok());

    // Already registered
    assert!(is_err(
        register_proxy(
            deps.as_mut(),
            &proxy,
            DEFAULT_BLOCK_HEIGHT,
            &proxy_pubkey,
            &proxy_stake,
        ),
        "already registered",
    ));

    assert!(deactivate_proxy(deps.as_mut(), &proxy, DEFAULT_BLOCK_HEIGHT).is_ok());

    assert!(is_err(
        deactivate_proxy(deps.as_mut(), &proxy, DEFAULT_BLOCK_HEIGHT),
        "Proxy already deactivated",
    ));

    let proxy_entry = store_get_proxy_entry(deps.as_mut().storage, &proxy).unwrap();
    assert_eq!(
        proxy_entry.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT
    );

    // Can't re-activate when providing different pubkey
    assert!(is_err(
        register_proxy(
            deps.as_mut(),
            &proxy,
            DEFAULT_BLOCK_HEIGHT,
            &proxy_pubkey2,
            &proxy_stake,
        ),
        "Proxy need to be unregistered to use a different public key",
    ));

    // Re-register/activate proxy
    assert!(register_proxy(
        deps.as_mut(),
        &proxy,
        DEFAULT_BLOCK_HEIGHT,
        &proxy_pubkey,
        &proxy_stake,
    )
    .is_ok());

    let proxy_entry = store_get_proxy_entry(deps.as_mut().storage, &proxy).unwrap();
    // Provided stake is added
    assert_eq!(
        proxy_entry.stake_amount.u128(),
        2 * DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT
    );
}

#[test]
fn test_add_data() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");

    let capsule = String::from("capsule");

    let data_entry = DataEntry {
        delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
        capsule: capsule.clone(),
    };

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &None,
        &DEFAULT_STAKE_DENOM.to_string(),
        &None,
        &None,
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
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &data_entry.delegator_pubkey,
        &capsule,
    )
    .is_ok());

    // Data already added
    assert!(is_err(
        add_data(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &data_id1,
            &data_entry.delegator_pubkey,
            &capsule,
        ),
        "already exist",
    ));

    assert_eq!(
        &store_get_data_entry(deps.as_mut().storage, &data_id1).unwrap(),
        &data_entry
    );
    assert_eq!(
        store_get_delegator_address(deps.as_mut().storage, &DELEGATOR1_PUBKEY.to_string()).unwrap(),
        delegator1
    );

    // Delgator2 cannot use delegator1 pubkey
    assert!(is_err(
        add_data(
            deps.as_mut(),
            &delegator2,
            DEFAULT_BLOCK_HEIGHT,
            &data_id2,
            &DELEGATOR1_PUBKEY.to_string(),
            &capsule,
        ),
        "already registered with this pubkey",
    ));
}

#[test]
fn test_remove_data() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let data_id1 = String::from("DATA1");

    let capsule = String::from("capsule");

    let data_entry = DataEntry {
        delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
        capsule: capsule.clone(),
    };

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &None,
        &DEFAULT_STAKE_DENOM.to_string(),
        &None,
        &None,
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
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &data_entry.delegator_pubkey,
        &capsule,
    )
    .is_ok());

    assert_eq!(
        &store_get_data_entry(deps.as_mut().storage, &data_id1).unwrap(),
        &data_entry
    );
    assert_eq!(
        store_get_delegator_address(deps.as_mut().storage, &DELEGATOR1_PUBKEY.to_string()).unwrap(),
        delegator1
    );

    assert!(is_err(
        remove_data(deps.as_mut(), &delegator2, DEFAULT_BLOCK_HEIGHT, &data_id1,),
        format!(
            "Delegator {} already registered with this pubkey.",
            delegator1
        )
        .as_str()
    ));

    assert!(remove_data(deps.as_mut(), &delegator1, DEFAULT_BLOCK_HEIGHT, &data_id1,).is_ok());

    assert_eq!(store_get_data_entry(deps.as_mut().storage, &data_id1), None,);
}

#[test]
fn test_add_delegation_and_request_reencryption() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegatee1 = Addr::unchecked("delegatee1".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy1_pubkey");
    let proxy2_pubkey: String = String::from("proxy2_pubkey");

    let data_id = String::from("DATA");
    let capsule = String::from("capsule");

    let data_entry = DataEntry {
        delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
        capsule: capsule.clone(),
    };

    // Staking
    let proxy_stake = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_TASK_REWARD_AMOUNT * 2),
    }];
    let insufficient_request_reward = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_TASK_REWARD_AMOUNT - 1),
    }];

    let higher_request_reward = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(50 + DEFAULT_TASK_REWARD_AMOUNT * 2),
    }];

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &Some(vec![proxy1.clone(), proxy2.clone()]),
        &DEFAULT_STAKE_DENOM.to_string(),
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id,
        &data_entry.delegator_pubkey,
        &capsule,
    )
    .is_ok());

    // Add delegation for proxy
    let proxy1_delegation_string = String::from("DS_P1");

    let proxy_delegations: Vec<ProxyDelegationString> = vec![ProxyDelegationString {
        proxy_addr: proxy1.clone(),
        delegation_string: proxy1_delegation_string,
    }];

    // Reencryption can't be requested yet
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &request_reward,
        ),
        "ProxyDelegation doesn't exist",
    ));

    // Add delegation
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    // Cannot add same delegation twice
    assert!(is_err(
        add_delegation(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy_delegations,
        ),
        "Delegation already exists.",
    ));

    // Insufficient stake amount
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &insufficient_request_reward,
        ),
        "Requires at least 100 atestfet.",
    ));

    // Only delegator can request re-encryption
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegatee1,
            DEFAULT_BLOCK_HEIGHT,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &higher_request_reward,
        ),
        "Reencryption is not permitted"
    ));

    // Reencryption can be requested only after add_delegation
    // Delegatee can request reencryption
    let res = request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id,
        &DELEGATEE1_PUBKEY.to_string(),
        &higher_request_reward,
    )
    .unwrap();

    // Check if given stake above required is returned back
    assert_eq!(
        res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: delegator1.to_string(),
            amount: vec![Coin::new(150, DEFAULT_STAKE_DENOM)],
        })
    );

    // Reencryption already requested
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &request_reward,
        ),
        "Reencryption already requested",
    ));

    // Check if task was created
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy1,
        ),
        Some(0u64)
    );

    assert_eq!(
        store_get_state(deps.as_mut().storage)
            .unwrap()
            .next_proxy_task_id,
        1u64
    );
}

#[test]
fn test_remove_data_when_reencryption_requested() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy1_pubkey");
    let proxy2_pubkey: String = String::from("proxy2_pubkey");

    let data_id1 = String::from("DATA1");

    let capsule = String::from("capsule");

    let data_entry = DataEntry {
        delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
        capsule: capsule.clone(),
    };

    let higher_request_reward = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(50 + DEFAULT_TASK_REWARD_AMOUNT * 2),
    }];

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &None,
        &DEFAULT_STAKE_DENOM.to_string(),
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    let proxy_stake = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    // Proxies register -> submits pubkeys
    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &data_entry.delegator_pubkey,
        &capsule,
    )
    .is_ok());

    assert_eq!(
        &store_get_data_entry(deps.as_mut().storage, &data_id1).unwrap(),
        &data_entry
    );
    assert_eq!(
        store_get_delegator_address(deps.as_mut().storage, &DELEGATOR1_PUBKEY.to_string()).unwrap(),
        delegator1
    );

    // Add delegation for proxy
    let proxy1_delegation_string = String::from("DS_P1");

    let proxy_delegations: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_addr: proxy1.clone(),
            delegation_string: proxy1_delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy2.clone(),
            delegation_string: proxy1_delegation_string,
        },
    ];

    // Add delegation
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    // Reencryption can be requested only after add_delegation
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &higher_request_reward,
    )
    .is_ok());

    // Provide fragment
    let proxy_fragment = String::from(FRAGMENT_P1_DR1_DE1);
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_fragment,
    )
    .is_ok());

    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy1.clone(),
        ),
        Some(0u64)
    );

    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy2.clone(),
        ),
        Some(1u64)
    );

    assert!(remove_data(deps.as_mut(), &delegator1, DEFAULT_BLOCK_HEIGHT, &data_id1,).is_ok());

    assert_eq!(store_get_data_entry(deps.as_mut().storage, &data_id1), None,);

    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy1,
        ),
        None
    );

    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy2,
        ),
        None
    );
}

#[test]
fn test_add_delegation_and_then_data_with_diffent_proxy_same_pubkey() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy1_pubkey");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");

    let capsule = String::from("capsule");

    // Staking
    let proxy_stake = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &Some(vec![proxy1.clone(), proxy2]),
        &DEFAULT_STAKE_DENOM.to_string(),
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());

    // Add delegation for proxy
    let proxy1_delegation_string = String::from("DS_P1");

    let proxy_delegations: Vec<ProxyDelegationString> = vec![ProxyDelegationString {
        proxy_addr: proxy1,
        delegation_string: proxy1_delegation_string,
    }];

    // Add delegation
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    // Add data by delegator2 with already used delegator1_pubkey is prevented
    assert!(is_err(
        add_data(
            deps.as_mut(),
            &delegator2,
            DEFAULT_BLOCK_HEIGHT,
            &data_id1,
            &DELEGATOR1_PUBKEY.to_string(),
            &capsule,
        ),
        "Delegator delegator1 already registered with this pubkey.",
    ));

    assert!(add_data(
        deps.as_mut(),
        &delegator2,
        DEFAULT_BLOCK_HEIGHT,
        &data_id2,
        &DELEGATOR2_PUBKEY.to_string(),
        &capsule,
    )
    .is_ok());
}

#[test]
fn test_add_delegation_edge_cases() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy1_pubkey");

    let proxy1_delegation_string = String::from("DS_P1");
    let proxy2_delegation_string = String::from("DS_P2");

    // Staking
    let proxy_stake = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &Some(vec![proxy1.clone(), proxy2.clone()]),
        &DEFAULT_STAKE_DENOM.to_string(),
        &None,
        &None,
        &None,
        &None,
        &Some(false),
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());

    // Try adding delegation for proxy 2 which is not registered
    let proxy_delegations: Vec<ProxyDelegationString> = vec![ProxyDelegationString {
        proxy_addr: proxy2,
        delegation_string: proxy2_delegation_string,
    }];

    assert!(is_err(
        add_delegation(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy_delegations,
        ),
        "Unregistered proxy with address proxy2",
    ));

    // Cannot add same pubkey twice
    let proxy_1_delegation = ProxyDelegationString {
        proxy_addr: proxy1,
        delegation_string: proxy1_delegation_string,
    };
    let proxy_delegations: Vec<ProxyDelegationString> =
        vec![proxy_1_delegation.clone(), proxy_1_delegation];

    assert!(is_err(
        add_delegation(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy_delegations,
        ),
        "Delegation string was already provided for proxy proxy1",
    ));

    // Cannot add delegation for less than minimum number of proxies
    assert!(is_err(
        add_delegation(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &DELEGATOR2_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
            &[],
        ),
        "Required at least 1 proxies.",
    ));
}

#[test]
fn test_provide_reencrypted_fragment() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy = Addr::unchecked("proxy".to_string());

    let delegator = Addr::unchecked("delegator".to_string());

    // Pubkeys
    let delegator_pubkey = String::from(DELEGATOR1_PUBKEY);
    let delegatee_pubkey = String::from(DELEGATEE1_PUBKEY);
    let other_delegatee_pubkey: String = String::from("DEK2");
    let proxy_pubkey: String = String::from("proxy_pubkey");

    let data_id = String::from("DATA");
    let capsule = String::from(CAPSULE);

    let data_entry = DataEntry {
        capsule: capsule.clone(),
        delegator_pubkey: delegator_pubkey.clone(),
    };

    // Staking
    let proxy_stake = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_TASK_REWARD_AMOUNT * 2),
    }];

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &Some(vec![proxy.clone()]),
        &DEFAULT_STAKE_DENOM.to_string(),
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(
        deps.as_mut(),
        &proxy,
        DEFAULT_BLOCK_HEIGHT,
        &proxy_pubkey,
        &proxy_stake,
    )
    .is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator,
        DEFAULT_BLOCK_HEIGHT,
        &data_id,
        &data_entry.delegator_pubkey,
        &capsule,
    )
    .is_ok());

    // Add delegation for proxy
    let proxy_delegation_string = String::from("DS_P1");
    let proxy_delegations: Vec<ProxyDelegationString> = vec![ProxyDelegationString {
        proxy_addr: proxy.clone(),
        delegation_string: proxy_delegation_string,
    }];

    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        DEFAULT_BLOCK_HEIGHT,
        &delegator_pubkey,
        &delegatee_pubkey,
        &proxy_delegations,
    )
    .is_ok());

    /*************** Request re-encryption *************/
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator,
        DEFAULT_BLOCK_HEIGHT,
        &data_id,
        &delegatee_pubkey,
        &request_reward,
    )
    .is_ok());

    /*************** Provide reencrypted fragment *************/
    assert_eq!(
        store_get_delegatee_proxy_task(deps.as_mut().storage, &data_id, &delegatee_pubkey, &proxy,)
            .unwrap(),
        0u64
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy,
        &0u64,
    ));

    let proxy_fragment = String::from(FRAGMENT_P1_DR1_DE1);
    // Provide unwanted fragment
    assert!(is_err(
        provide_reencrypted_fragment(
            deps.as_mut(),
            &proxy,
            DEFAULT_BLOCK_HEIGHT,
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
            DEFAULT_BLOCK_HEIGHT,
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
        DEFAULT_BLOCK_HEIGHT,
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
            DEFAULT_BLOCK_HEIGHT,
            &data_id,
            &delegatee_pubkey,
            &proxy_fragment,
        ),
        "Fragment already provided.",
    ));

    // This entry is removed when proxy task is done
    assert!(!store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy,
        &0u64,
    ));

    let task = store_get_proxy_task(deps.as_mut().storage, &0u64).unwrap();
    assert_eq!(task.fragment, Some(proxy_fragment));
}

#[test]
fn test_contract_lifecycle() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());

    let delegator = Addr::unchecked("delegator".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");

    let data_id = String::from("DATA");

    let data_entry = DataEntry {
        capsule: CAPSULE.to_string(),
        delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
    };

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let proxy_stake = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_TASK_REWARD_AMOUNT * 2),
    }];

    /*************** Initialise *************/
    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &Some(2),
        &None,
        &Some(proxies),
        &stake_denom,
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator,
        DEFAULT_BLOCK_HEIGHT,
        &data_id,
        &data_entry.delegator_pubkey,
        &CAPSULE.to_string(),
    )
    .is_ok());

    // Add 2 delegations for 2 proxies
    let proxy1_delegation_string = String::from("DS_P1");
    let proxy2_delegation_string = String::from("DS_P2");

    let proxy_delegations: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_addr: proxy1.clone(),
            delegation_string: proxy1_delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy2.clone(),
            delegation_string: proxy2_delegation_string,
        },
    ];

    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        DelegationState::NonExisting
    );

    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        DelegationState::Active
    );

    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    // No tasks yet
    assert!(
        get_proxy_tasks(deps.as_mut().storage, &proxy1, &DEFAULT_BLOCK_HEIGHT)
            .unwrap()
            .is_empty()
    );
    assert!(
        get_proxy_tasks(deps.as_mut().storage, &proxy2, &DEFAULT_BLOCK_HEIGHT)
            .unwrap()
            .is_empty()
    );

    let state = store_get_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &0u64,
        ),
        ReencryptionRequestState::Inaccessible
    );

    /*************** Request reencryption by delegator *************/

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator,
        DEFAULT_BLOCK_HEIGHT,
        &data_id,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward,
    )
    .is_ok());

    let state = store_get_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &0u64,
        ),
        ReencryptionRequestState::Ready
    );

    // Check number of tasks
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy1).len(),
        1
    );
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy2).len(),
        1
    );

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator,
        DEFAULT_BLOCK_HEIGHT,
        &data_id,
        &DELEGATEE2_PUBKEY.to_string(),
        &request_reward,
    )
    .is_ok());

    // Check number of requests
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy1).len(),
        2
    );
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy2).len(),
        2
    );

    /*************** Process reencryption by proxies *************/
    let all_tasks = store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy1);
    assert_eq!(all_tasks.len(), 2);

    // Check if proxy got task 1
    let proxy1_task1 =
        &get_proxy_tasks(deps.as_mut().storage, &proxy1, &DEFAULT_BLOCK_HEIGHT).unwrap()[0];
    assert_eq!(
        proxy1_task1,
        &ProxyTaskResponse {
            data_id: data_id.clone(),
            capsule: data_entry.capsule.clone(),
            delegatee_pubkey: DELEGATEE1_PUBKEY.to_string(),
            delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
            delegation_string: proxy1_delegation_string.clone(),
        }
    );

    // Check stake before finishing re-encryption
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 2 * DEFAULT_PER_TASK_SLASH_STAKE_AMOUNT
    );

    // Proxy1 provides fragment for task1
    let proxy1_fragment1: String = String::from("Agn8MTBWSKzz277FLeNKvhOwa3juw7HBciLmyA/3kZ2hAtQv0l/B+Ej2vQLxZDx+MHDr5uevth9PzntoIz6gbPI1xJk3dVwZohs3YgdaXJsBXpAambF1FpOGrola7KcwjtQDOL6tYr3e6dlMgsW9GnONyZUWk15ixjxdrAIZfp8qWAMCbOd9fCO820cnEqBeQHpit75l8gxb6Al3s28p4uMFeq4Dzsh5SbQgRk7KjI9LEq2a9YzQ2ts3O5KEx3SuZoCOE0UDns625ayBRPD5BHdYwGaCGo/w6oJ5PvRp7rEpMSvxpOACu5HXcj2KNZnzAc2QGNrHmrAIxxS4pUbp7ffoPjSK/eGOs3Yh2IaeLQMzj2FNpUCYii6D3KJMT5sWqdKQV+5Aw6ebgujLY0o4Gs2aJ3toE3GuNuSfwFKzySmpq5CfSGaJJftZDYt72g7t8cRKVFXT6D8ugCXfMVL6GRE7adJEkYU=");
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id,
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy1_fragment1,
    )
    .is_ok());

    // Check if proxy got reward
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 1 * DEFAULT_PER_TASK_SLASH_STAKE_AMOUNT
            + DEFAULT_TASK_REWARD_AMOUNT
    );

    // Proxy2 tries to provides fragment already provided by proxy1 for task1
    assert!(is_err(
        provide_reencrypted_fragment(
            deps.as_mut(),
            &proxy2,
            DEFAULT_BLOCK_HEIGHT,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy1_fragment1,
        ),
        "Fragment already provided by other proxy.",
    ));

    // Check numbers of tasks
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy1).len(),
        1
    );
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy2).len(),
        2
    );

    // Check available fragments
    assert_eq!(
        get_all_fragments(
            deps.as_mut().storage,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        vec![proxy1_fragment1.clone()]
    );
    assert_eq!(
        get_all_fragments(
            deps.as_mut().storage,
            &data_id,
            &DELEGATEE2_PUBKEY.to_string(),
        )
        .len(),
        0
    );

    // Check if proxy got task 2
    let proxy1_task2 =
        &get_proxy_tasks(deps.as_mut().storage, &proxy1, &DEFAULT_BLOCK_HEIGHT).unwrap()[0];
    assert_eq!(
        proxy1_task2,
        &ProxyTaskResponse {
            data_id: data_id.clone(),
            capsule: data_entry.capsule,
            delegatee_pubkey: DELEGATEE2_PUBKEY.to_string(),
            delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
            delegation_string: proxy1_delegation_string,
        }
    );

    // Proxy1 provides fragment for task1
    let proxy1_fragment2: String = String::from("A5fWxYyjkfJu/k2oq5A6w+pLgRtWRIKu2uEHe/i0AGSiAsK7jq0a7KjTeiBCBRTC64ATDb/QfYQ9CBoiF5FDdbJwh3Yb5RoZgkclP0cqNtftZnRCdVUuycy2UpQ7f4x5tFkCkymfOr+pOYAe56kPnK9cTGjuwGdgcrV3i5A1ocF5xYsD0mFd9APeYHeRjAIgPzM3na8xJuYgSdY9FA6upZOYqxkDGWHWjB6Uvjby3zTbN+A8vuQmHRx0NST4ICR5HfKCCPQCE6/D/Dep4lyf4v9E03VgMisZKFWW7+YP5qAIWgNDSPoCb8OqfDTrYzJSKGZg+ti4l/Cjo5PaqmlZlj1MCR/Rb906nywBtIQCU9iVAHGUnE8h9QV+kYWik4s2Vcq2W6r/Y6MXnYLK9JsYpIwny6zDwHzOwfTk4Wn9mLGENf4q2s/5ZeM/1TJIu6wECI+L5zmGMWl40iloHhHcxeCVKwKnM0s=");
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id,
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy1_fragment2,
    )
    .is_ok());

    // Check if proxy got reward
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + 2 * DEFAULT_TASK_REWARD_AMOUNT
    );

    // All tasks completed for proxy1
    assert!(
        get_proxy_tasks(deps.as_mut().storage, &proxy1, &DEFAULT_BLOCK_HEIGHT)
            .unwrap()
            .is_empty()
    );
    // But not for proxy2
    assert!(
        !get_proxy_tasks(deps.as_mut().storage, &proxy2, &DEFAULT_BLOCK_HEIGHT)
            .unwrap()
            .is_empty()
    );

    // Check available fragments
    assert_eq!(
        get_all_fragments(
            deps.as_mut().storage,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        vec![proxy1_fragment1]
    );
    assert_eq!(
        get_all_fragments(
            deps.as_mut().storage,
            &data_id,
            &DELEGATEE2_PUBKEY.to_string(),
        ),
        vec![proxy1_fragment2]
    );

    let state = store_get_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &0u64,
        ),
        ReencryptionRequestState::Ready
    );
    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        DelegationState::Active
    );

    // Re-encryption was requested in past
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator,
            DEFAULT_BLOCK_HEIGHT,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &request_reward,
        ),
        "Reencryption already requested",
    ));

    // Proxy 2 leaves - all its delegations gets deleted
    assert!(unregister_proxy(deps.as_mut(), &proxy2, DEFAULT_BLOCK_HEIGHT).is_ok());

    // Check proxy stake amount
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(proxy.stake_amount.u128(), 0);

    // Proxy 2 gets back
    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());

    // Check proxy stake amount
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT
    );

    let state = store_get_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id,
            &DELEGATEE1_PUBKEY.to_string(),
            &0u64,
        ),
        ReencryptionRequestState::Abandoned
    );
    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        DelegationState::NonExisting
    );

    // ProxyDelegation can be re-created again
    assert!(add_delegation(
        deps.as_mut(),
        &delegator,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    // Check if all stake gets returned to proxy after un-registered
    let unregister_response =
        unregister_proxy(deps.as_mut(), &proxy1, DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + DEFAULT_TASK_REWARD_AMOUNT * 2,
                stake_denom.as_str(),
            )],
        })
    );
}

#[test]
fn test_proxy_unregister_with_requests() {
    let mut deps = mock_dependencies();

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
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");
    let proxy4_pubkey: String = String::from("proxy_pubkey4");
    let proxy5_pubkey: String = String::from("proxy_pubkey5");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");
    let data_id3 = String::from("DATA3");

    let delegation_string = String::from("DELESTRING");

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward_2_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_TASK_REWARD_AMOUNT * 2),
    }];
    let request_reward_3_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_TASK_REWARD_AMOUNT * 3),
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
        DEFAULT_BLOCK_HEIGHT,
        &Some(2),
        &None,
        &Some(proxies),
        &stake_denom,
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy3,
        DEFAULT_BLOCK_HEIGHT,
        &proxy3_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy4,
        DEFAULT_BLOCK_HEIGHT,
        &proxy4_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy5,
        DEFAULT_BLOCK_HEIGHT,
        &proxy5_pubkey,
        &proxy_stake,
    )
    .is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATOR1_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id2,
        &DELEGATOR1_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());
    assert!(add_data(
        deps.as_mut(),
        &delegator2,
        DEFAULT_BLOCK_HEIGHT,
        &data_id3,
        &DELEGATOR2_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());

    // Add delegations manually

    let delegation1 = ProxyDelegation {
        delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
        delegatee_pubkey: DELEGATEE1_PUBKEY.to_string(),
        delegation_string: delegation_string.clone(),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee1
    store_set_delegation(deps.as_mut().storage, &0, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy1,
        &0,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy1, &0);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee1
    store_set_delegation(deps.as_mut().storage, &1, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy2,
        &1,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy2, &1);

    // delegator1 with delegator1_pubkey and proxy3 for delegatee1
    store_set_delegation(deps.as_mut().storage, &2, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy3,
        &2,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy3, &2);

    let delegation2 = ProxyDelegation {
        delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
        delegatee_pubkey: DELEGATEE2_PUBKEY.to_string(),
        delegation_string: delegation_string.clone(),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee2
    store_set_delegation(deps.as_mut().storage, &3, &delegation2);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy1,
        &3,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy1, &3);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee2
    store_set_delegation(deps.as_mut().storage, &4, &delegation2);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy2,
        &4,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy2, &4);

    let delegation3 = ProxyDelegation {
        delegator_pubkey: DELEGATOR2_PUBKEY.to_string(),
        delegatee_pubkey: DELEGATEE1_PUBKEY.to_string(),
        delegation_string: delegation_string,
    };

    // delegator2 with delegator2_pubkey and proxy4 for delegatee1
    store_set_delegation(deps.as_mut().storage, &5, &delegation3);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy4,
        &5,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy4, &5);

    // delegator2 with delegator2_pubkey and proxy5 for delegatee1
    store_set_delegation(deps.as_mut().storage, &6, &delegation3);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy5,
        &6,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy5, &6);

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
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward_3_proxies,
    )
    .is_ok());

    // Check proxy2 stake amount after creating reencryption request
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - DEFAULT_PER_TASK_SLASH_STAKE_AMOUNT
    );

    // Re-encryption requests with DATA1 to delegatee2
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE2_PUBKEY.to_string(),
        &request_reward_2_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA2 to delegatee2
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id2,
        &DELEGATEE2_PUBKEY.to_string(),
        &request_reward_2_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA3 to delegatee1
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator2,
        DEFAULT_BLOCK_HEIGHT,
        &data_id3,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward_2_proxies,
    )
    .is_ok());

    // Check proxy2 stake amount after creating reencryption requests
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 3 * DEFAULT_PER_TASK_SLASH_STAKE_AMOUNT
    );

    /*
    // Provide wrong fragment
    assert!(is_err(
        provide_reencrypted_fragment(
            deps.as_mut(),
            &proxy2,
            DEFAULT_BLOCK_HEIGHT,
            &data_id1,
            &DELEGATEE2_PUBKEY.to_string(),
            &FRAGMENT_P2_DR1_DE1.to_string(),
        ),
        "Invalid KeyFrag signature",
    ));
     */

    // Complete tasks
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE2_PUBKEY.to_string(),
        &FRAGMENT_P2_DR1_DE2.to_string(),
    )
    .is_ok());

    // Check proxy2 stake amount after finishing one reencryption task
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 2 * DEFAULT_PER_TASK_SLASH_STAKE_AMOUNT
            + DEFAULT_TASK_REWARD_AMOUNT
    );

    // Unregister proxy2
    let unregister_response =
        unregister_proxy(deps.as_mut(), &proxy2, DEFAULT_BLOCK_HEIGHT).unwrap();

    // Check if stake from unfinished request get returned to delegator
    assert_eq!(
        unregister_response.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: delegator1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_TASK_REWARD_AMOUNT * 2,
                stake_denom.as_str(),
            )],
        })
    );

    // Check if stake gets returned to proxy and if proxy got slashed
    assert_eq!(
        unregister_response.messages[1],
        SubMsg::new(BankMsg::Send {
            to_address: proxy2.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + DEFAULT_TASK_REWARD_AMOUNT
                    - 2 * DEFAULT_PER_TASK_SLASH_STAKE_AMOUNT,
                stake_denom.as_str(),
            )],
        })
    );

    // Already unregistered
    assert!(is_err(
        unregister_proxy(deps.as_mut(), &proxy2, DEFAULT_BLOCK_HEIGHT),
        "Proxy already unregistered",
    ));

    // Check state of delegations

    // ProxyDelegation 1 - Stays
    // delgator1, delegatee1, proxy1 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy1,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &0).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1,
        &0,
    ));

    // delgator1, delegatee1, proxy2 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy2,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &1).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy2,
        &1,
    ));

    // delgator1, delegatee1, proxy3 - stays
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy3,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &2).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy3,
        &2,
    ));

    // ProxyDelegation 2 - Number of proxies below threshold, removed entire delegation
    // delgator1, delegatee2, proxy1 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy1,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &3).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1,
        &3,
    ));

    // delgator1, delegatee2, proxy2 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy2,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &4).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1,
        &4,
    ));

    // ProxyDelegation 3 - Unaffected
    // delgator2, delegatee1, proxy4 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR2_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy4,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &5).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy4,
        &5,
    ));

    // delgator2, delegatee1, proxy5 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR2_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy5,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &6).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy5,
        &6,
    ));

    // Check state of re-encryption requests

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee1, proxy1 - unaffected
    assert!(store_get_proxy_task(deps.as_mut().storage, &0).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy1,
        )
        .unwrap(),
        0
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy1,
        &0,
    ));

    // DATA1, delegatee1, proxy2 - incomplete and removed
    assert!(
        store_get_proxy_task(deps.as_mut().storage, &1)
            .unwrap()
            .abandoned
    );
    assert!(!store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy1,
        &1,
    ));

    // DATA1, delegatee1, proxy3 - unaffected
    assert!(store_get_proxy_task(deps.as_mut().storage, &2).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy3,
        )
        .unwrap(),
        2
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy3,
        &2,
    ));

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee2, proxy1 - Not removed because it can still be completed
    assert!(store_get_proxy_task(deps.as_mut().storage, &3).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE2_PUBKEY.to_string(),
            &proxy1,
        )
        .unwrap(),
        3
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy1,
        &3,
    ));

    // DATA1, delegatee2, proxy2 - complete - unaffected
    // Request can still be obtained by delegatee
    assert!(store_get_proxy_task(deps.as_mut().storage, &4).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE2_PUBKEY.to_string(),
            &proxy2,
        )
        .unwrap(),
        4
    );
    // Request is completed - won't appear in proxy tasks
    assert!(!store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy2,
        &4,
    ));

    // Re-encryption requests with DATA2 to delegatee2
    // DATA2, delegatee2, proxy1 - incomplete, not removed
    assert!(store_get_proxy_task(deps.as_mut().storage, &5).is_some());
    assert!(store_get_delegatee_proxy_task(
        deps.as_mut().storage,
        &data_id2,
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy1,
    )
    .is_some());
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy1,
        &5,
    ));

    // DATA1, delegatee2, proxy2 - complete - removed because more than threshold fragments cannot be provided
    // Request can still be obtained by delegatee
    assert!(
        store_get_proxy_task(deps.as_mut().storage, &6)
            .unwrap()
            .abandoned
    );
    // Request is completed - won't appear in proxy tasks
    assert!(!store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy2,
        &6,
    ));

    // Re-encryption requests with DATA3 to delegatee1
    // DATA3, delegatee1, proxy4 - unaffected
    assert!(store_get_proxy_task(deps.as_mut().storage, &7).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id3,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy4,
        )
        .unwrap(),
        7
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy4,
        &7,
    ));

    // DATA3, delegatee1, proxy5 - unaffected
    assert!(store_get_proxy_task(deps.as_mut().storage, &8).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id3,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy5,
        )
        .unwrap(),
        8
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy5,
        &8,
    ));

    // ProxyDelegation 1 was removed with all re-encryption requests
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &request_reward_3_proxies,
        ),
        "Reencryption already requested",
    ));
}

#[test]
fn test_proxy_deactivate_and_remove_with_requests() {
    let mut deps = mock_dependencies();

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
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");
    let proxy4_pubkey: String = String::from("proxy_pubkey4");
    let proxy5_pubkey: String = String::from("proxy_pubkey5");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");
    let data_id3 = String::from("DATA3");

    let delegation_string = String::from("DELESTRING");

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    let request_reward_2_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_TASK_REWARD_AMOUNT * 2),
    }];
    let request_reward_3_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_TASK_REWARD_AMOUNT * 3),
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
        DEFAULT_BLOCK_HEIGHT,
        &Some(2),
        &None,
        &Some(proxies),
        &stake_denom,
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy3,
        DEFAULT_BLOCK_HEIGHT,
        &proxy3_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy4,
        DEFAULT_BLOCK_HEIGHT,
        &proxy4_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy5,
        DEFAULT_BLOCK_HEIGHT,
        &proxy5_pubkey,
        &proxy_stake,
    )
    .is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATOR1_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id2,
        &DELEGATOR1_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());
    assert!(add_data(
        deps.as_mut(),
        &delegator2,
        DEFAULT_BLOCK_HEIGHT,
        &data_id3,
        &DELEGATOR2_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());

    // Add delegations manually

    let delegation1 = ProxyDelegation {
        delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
        delegatee_pubkey: DELEGATEE1_PUBKEY.to_string(),
        delegation_string: delegation_string.clone(),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee1
    store_set_delegation(deps.as_mut().storage, &0, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy1,
        &0,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy1, &0);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee1
    store_set_delegation(deps.as_mut().storage, &1, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy2,
        &1,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy2, &1);

    // delegator1 with delegator1_pubkey and proxy3 for delegatee1
    store_set_delegation(deps.as_mut().storage, &2, &delegation1);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation1.delegator_pubkey,
        &delegation1.delegatee_pubkey,
        &proxy3,
        &2,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy3, &2);

    let delegation2 = ProxyDelegation {
        delegator_pubkey: DELEGATOR1_PUBKEY.to_string(),
        delegatee_pubkey: DELEGATEE2_PUBKEY.to_string(),
        delegation_string: delegation_string.clone(),
    };

    // delegator1 with delegator1_pubkey and proxy1 for delegatee2
    store_set_delegation(deps.as_mut().storage, &3, &delegation2);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy1,
        &3,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy1, &3);

    // delegator1 with delegator1_pubkey and proxy2 for delegatee2
    store_set_delegation(deps.as_mut().storage, &4, &delegation2);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation2.delegator_pubkey,
        &delegation2.delegatee_pubkey,
        &proxy2,
        &4,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy2, &4);

    let delegation3 = ProxyDelegation {
        delegator_pubkey: DELEGATOR2_PUBKEY.to_string(),
        delegatee_pubkey: DELEGATEE1_PUBKEY.to_string(),
        delegation_string: delegation_string,
    };

    // delegator2 with delegator2_pubkey and proxy4 for delegatee1
    store_set_delegation(deps.as_mut().storage, &5, &delegation3);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy4,
        &5,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy4, &5);

    // delegator2 with delegator2_pubkey and proxy5 for delegatee1
    store_set_delegation(deps.as_mut().storage, &6, &delegation3);
    store_set_delegation_id(
        deps.as_mut().storage,
        &delegation3.delegator_pubkey,
        &delegation3.delegatee_pubkey,
        &proxy5,
        &6,
    );
    store_add_per_proxy_delegation(deps.as_mut().storage, &proxy5, &6);

    // Request re-encryptions

    // Re-encryption requests with DATA1 to delegatee1
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward_3_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA1 to delegatee2
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE2_PUBKEY.to_string(),
        &request_reward_2_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA2 to delegatee2
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id2,
        &DELEGATEE2_PUBKEY.to_string(),
        &request_reward_2_proxies,
    )
    .is_ok());
    // Re-encryption requests with DATA3 to delegatee1
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator2,
        DEFAULT_BLOCK_HEIGHT,
        &data_id3,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward_2_proxies,
    )
    .is_ok());

    // Complete requests
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy2, DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE2_PUBKEY.to_string(),
        &String::from("AlCR5oC5v9i8G6IohBPRXg5qfYDP3awqEk2RIusDZo3wAwJEdUq0N3TG1iJ0lqfwRnNos1w3ysr3Vd0WPfFoBc991Pmeib1ZyAX52bfvtjcB3VdzunAnXrH0x259LtNX94oCkymfOr+pOYAe56kPnK9cTGjuwGdgcrV3i5A1ocF5xYsCK1m3Gcr1OeLeHMh26lX7rSDQKP7PoKJC4N/Mgeaqn0wDmP4BflDmAFm7AHvNbq6j5wlYLbZ0SDrpQ/L0axS4huYCYaHbevxWAYoQl2o1m+b5KtVg3c//Iaw1L4RRut+m1GMDhhuHPM1wslLQnN799sLX6itYkBWwTYnCDEc/9NBeCv4/P0LPZSNH6OW8Ta4x08yMA3WPSuBBj3rPcZt5Nydl+Mf4oj47fbmN6AhI9js2P7JBUCi/54jiNUkB4tDgqD2nNzC5ngPVPsnJEcSPx74aW1ppKowvRT7DwM9CNsEAbG0="),
    )
        .is_ok());

    // Check proxy state
    assert!(store_get_is_proxy_active(deps.as_mut().storage, &proxy2,));
    assert!(
        !get_proxy_tasks(deps.as_mut().storage, &proxy2, &DEFAULT_BLOCK_HEIGHT)
            .unwrap()
            .is_empty()
    );
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(proxy.state, ProxyState::Registered);
    assert_eq!(proxy.proxy_pubkey.unwrap(), proxy2_pubkey);

    // Sender is not a proxy
    assert!(is_err(
        deactivate_proxy(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT),
        "Sender is not a proxy",
    ));

    // Deactivate proxy2
    assert!(deactivate_proxy(deps.as_mut(), &proxy2, DEFAULT_BLOCK_HEIGHT).is_ok());
    // Already deactivated
    assert!(is_err(
        deactivate_proxy(deps.as_mut(), &proxy2, DEFAULT_BLOCK_HEIGHT),
        "Proxy already deactivated",
    ));

    // Check proxy state
    assert!(!store_get_is_proxy_active(deps.as_mut().storage, &proxy2,));
    assert!(
        !get_proxy_tasks(deps.as_mut().storage, &proxy2, &DEFAULT_BLOCK_HEIGHT)
            .unwrap()
            .is_empty()
    );
    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(proxy.state, ProxyState::Leaving);
    assert_eq!(proxy.proxy_pubkey.unwrap(), proxy2_pubkey);

    // Check state of delegations

    // ProxyDelegation 1 - Proxy 2 left, but delegation still exists
    // delgator1, delegatee1, proxy1 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy1,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &0).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1,
        &0,
    ));

    // delgator1, delegatee1, proxy2 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy2,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &1).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy2,
        &1,
    ));

    // delgator1, delegatee1, proxy3 - still exist
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy3,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &2).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy3,
        &2,
    ));

    // ProxyDelegation 2 - Number of proxies below threshold, removed entire delegation
    // delgator1, delegatee2, proxy1 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy1,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &3).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1,
        &3,
    ));

    // delgator1, delegatee2, proxy2 - Removed
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy2,
    )
    .is_none());
    assert!(store_get_delegation(deps.as_mut().storage, &4).is_none());
    assert!(!store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy1,
        &4,
    ));

    // ProxyDelegation 3 - Unaffected
    // delgator2, delegatee1, proxy4 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR2_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy4,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &5).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy4,
        &5,
    ));

    // delgator2, delegatee1, proxy5 - Unaffected
    assert!(store_get_proxy_delegation_id(
        deps.as_mut().storage,
        &DELEGATOR2_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy5,
    )
    .is_some());
    assert!(store_get_delegation(deps.as_mut().storage, &6).is_some());
    assert!(store_is_proxy_delegation(
        deps.as_mut().storage,
        &proxy5,
        &6,
    ));

    // Remove proxy by admin
    assert!(remove_proxy(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT, &proxy2).is_ok());
    // Already removed
    assert!(is_err(
        remove_proxy(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT, &proxy2),
        "Sender is not a proxy",
    ));

    // Check proxy state
    assert!(!store_get_is_proxy_active(deps.as_mut().storage, &proxy2,));
    assert!(
        get_proxy_tasks(deps.as_mut().storage, &proxy2, &DEFAULT_BLOCK_HEIGHT)
            .unwrap()
            .is_empty()
    );
    assert!(store_get_proxy_entry(deps.as_mut().storage, &proxy2).is_none());

    // Check state of re-encryption requests

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee1, proxy1 - unaffected
    assert!(store_get_proxy_task(deps.as_mut().storage, &0).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy1,
        )
        .unwrap(),
        0
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy1,
        &0,
    ));

    // DATA1, delegatee1, proxy2 - incomplete and removed
    assert!(
        store_get_proxy_task(deps.as_mut().storage, &1)
            .unwrap()
            .abandoned
    );
    assert!(!store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy1,
        &1,
    ));

    // DATA1, delegatee1, proxy3 - unaffected
    assert!(
        !store_get_proxy_task(deps.as_mut().storage, &2)
            .unwrap()
            .abandoned
    );
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy3,
        )
        .unwrap(),
        2
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy3,
        &2,
    ));

    // Re-encryption requests with DATA1 to delegatee1
    // DATA1, delegatee2, proxy1 - Not removed because it can still be completed
    assert!(store_get_proxy_task(deps.as_mut().storage, &3).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE2_PUBKEY.to_string(),
            &proxy1,
        )
        .unwrap(),
        3
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy1,
        &3,
    ));

    // DATA1, delegatee2, proxy2 - complete - unaffected
    // Request can still be obtained by delegatee
    assert!(store_get_proxy_task(deps.as_mut().storage, &4).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id1,
            &DELEGATEE2_PUBKEY.to_string(),
            &proxy2,
        )
        .unwrap(),
        4
    );
    // Request is completed - won't appear in proxy tasks
    assert!(!store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy2,
        &4,
    ));

    // Re-encryption requests with DATA2 to delegatee2
    // DATA2, delegatee2, proxy1 - incomplete and not removed
    assert!(store_get_proxy_task(deps.as_mut().storage, &5).is_some());
    assert!(store_get_delegatee_proxy_task(
        deps.as_mut().storage,
        &data_id2,
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy1,
    )
    .is_some());
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy1,
        &5,
    ));

    // DATA1, delegatee2, proxy2 - complete - removed because more than threshold fragments cannot be provided
    // Request can still be obtained by delegatee
    assert!(
        store_get_proxy_task(deps.as_mut().storage, &6)
            .unwrap()
            .abandoned
    );
    // Request is completed - won't appear in proxy tasks
    assert!(!store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy2,
        &6,
    ));

    // Re-encryption requests with DATA3 to delegatee1
    // DATA3, delegatee1, proxy4 - unaffected
    assert!(store_get_proxy_task(deps.as_mut().storage, &7).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id3,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy4,
        )
        .unwrap(),
        7
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy4,
        &7,
    ));

    // DATA3, delegatee1, proxy5 - unaffected
    assert!(store_get_proxy_task(deps.as_mut().storage, &8).is_some());
    assert_eq!(
        store_get_delegatee_proxy_task(
            deps.as_mut().storage,
            &data_id3,
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy5,
        )
        .unwrap(),
        8
    );
    assert!(store_is_proxy_task_in_queue(
        deps.as_mut().storage,
        &proxy5,
        &8,
    ));
}

#[test]
fn test_proxy_stake_withdrawal() {
    let mut deps = mock_dependencies();
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());
    let proxy2 = Addr::unchecked("proxy2".to_string());

    let proxy1_pubkey: String = String::from("proxy1_pubkey");
    let proxy2_pubkey: String = String::from("proxy2_pubkey");

    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone()];

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let proxy_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + 50),
    }];

    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &Some(proxies),
        &stake_denom,
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());

    let proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(
        proxy.stake_amount.u128(),
        DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + 50
    );

    // Proxy1 is trying to withdraw more than is available gives you maximum available stake
    let withdraw_res = withdraw_stake(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &Some(Uint128::new(51)),
    )
    .unwrap();
    assert_eq!(
        withdraw_res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(50, stake_denom.as_str())],
        })
    );

    // All stake of proxy1 withdrawn
    assert!(is_err(
        withdraw_stake(deps.as_mut(), &proxy1, DEFAULT_BLOCK_HEIGHT, &None),
        "Not enough stake to withdraw",
    ));

    // Proxy2 is trying to withdraw maximum available stake
    let withdraw_res = withdraw_stake(deps.as_mut(), &proxy2, DEFAULT_BLOCK_HEIGHT, &None).unwrap();
    assert_eq!(
        withdraw_res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy2.to_string(),
            amount: vec![Coin::new(50, stake_denom.as_str())],
        })
    );

    // All stake of proxy2 withdrawn
    assert!(is_err(
        withdraw_stake(deps.as_mut(), &proxy2, DEFAULT_BLOCK_HEIGHT, &None),
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
        withdraw_stake(
            deps.as_mut(),
            &proxy1,
            DEFAULT_BLOCK_HEIGHT,
            &Some(Uint128::new(1)),
        ),
        "Not enough stake to withdraw",
    ));

    // Remaining stake can be withdrawn only by unregistering
    let unregister_res = unregister_proxy(deps.as_mut(), &proxy1, DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(
        unregister_res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT,
                stake_denom.as_str(),
            )],
        })
    );

    // Proxy unregistered
    assert!(is_err(
        withdraw_stake(
            deps.as_mut(),
            &proxy1,
            DEFAULT_BLOCK_HEIGHT,
            &Some(Uint128::new(1)),
        ),
        "Not enough stake to withdraw",
    ));
}

#[test]
fn test_proxy_add_stake() {
    let mut deps = mock_dependencies();
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy1".to_string());

    let proxy1_pubkey: String = String::from("proxy1_pubkey");

    let proxies: Vec<Addr> = vec![proxy1.clone()];

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
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
        DEFAULT_BLOCK_HEIGHT,
        &None,
        &None,
        &Some(proxies),
        &stake_denom,
        &None,
        &None,
        &None,
        &None,
        &None,
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());

    assert!(is_err(
        add_stake(
            deps.as_mut(),
            &proxy1,
            DEFAULT_BLOCK_HEIGHT,
            &proxy_wrong_coins_additional_stake,
        ),
        "Expected 1 Coin with denom atestfet",
    ));
    assert!(add_stake(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy_additional_stake,
    )
    .is_ok());

    // Check if withdrawn stake amount
    let unregister_res = unregister_proxy(deps.as_mut(), &proxy1, DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(
        unregister_res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT + 50,
                stake_denom.as_str(),
            )],
        })
    );
}

#[test]
fn test_proxy_insufficient_funds_task_skip() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());
    let proxy3 = Addr::unchecked("proxy_3".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");
    let data_id3 = String::from("DATA3");

    let capsule = String::from("capsule");

    let delegation_string = String::from("DELESTRING");

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let minimum_proxy_stake_amount: u128 = 100;
    let per_proxy_task_reward_amount: u128 = 40;
    let per_task_slash_stake_amount: u128 = 98;

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
        amount: Uint128::new(per_proxy_task_reward_amount * 1),
    }];
    let request_reward_2_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(per_proxy_task_reward_amount * 2),
    }];
    let request_reward_3_proxies = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(per_proxy_task_reward_amount * 3),
    }];

    /*************** Initialise *************/
    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone(), proxy3.clone()];
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT,
        &Some(2),
        &None,
        &Some(proxies),
        &stake_denom,
        &Some(Uint128::new(minimum_proxy_stake_amount)),
        &Some(Uint128::new(per_proxy_task_reward_amount)),
        &Some(Uint128::new(per_task_slash_stake_amount)),
        &None,
        &None,
    )
    .is_ok());

    /*************** Register proxies *************/
    // Proxies register -> submits pubkeys
    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy1_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy2_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy3,
        DEFAULT_BLOCK_HEIGHT,
        &proxy3_pubkey,
        &proxy3_stake,
    )
    .is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATOR1_PUBKEY.to_string(),
        &capsule,
    )
    .is_ok());
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id2,
        &DELEGATOR1_PUBKEY.to_string(),
        &capsule,
    )
    .is_ok());
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id3,
        &DELEGATOR1_PUBKEY.to_string(),
        &capsule,
    )
    .is_ok());

    let proxy_delegations: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_addr: proxy1.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy2.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy3.clone(),
            delegation_string: delegation_string,
        },
    ];

    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    // Create first reencryption request -- All proxies has enough stake
    assert_eq!(
        get_n_available_proxies_from_delegation(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
            &per_task_slash_stake_amount,
        ),
        3
    );
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward_3_proxies,
    )
    .is_ok());

    assert_eq!(
        store_get_proxy_entry(deps.as_mut().storage, &proxy2)
            .unwrap()
            .stake_amount
            .u128(),
        102
    );

    // Create second reencryption request -- 1 proxies skipped because of insufficient funds
    assert_eq!(
        get_n_available_proxies_from_delegation(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
            &per_task_slash_stake_amount,
        ),
        2
    );
    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        DelegationState::Active
    );

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id2,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward_2_proxies,
    )
    .is_ok());

    // Creating third reencryption request fails -- 2 proxies skipped because of insufficient funds
    assert_eq!(
        get_n_available_proxies_from_delegation(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
            &per_task_slash_stake_amount,
        ),
        1
    );

    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        DelegationState::ProxiesAreBusy
    );
    assert!(is_err(
        request_reencryption(
            deps.as_mut(),
            &delegator1,
            DEFAULT_BLOCK_HEIGHT,
            &data_id2,
            &DELEGATEE1_PUBKEY.to_string(),
            &request_reward_1_proxy,
        ),
        "Proxies are too busy, try again later. Available 1 proxies out of 3, minimum is 2",
    ));

    // Requests:
    // req1 - (0)proxy1, (1)proxy2, (2)proxy3
    // req2 - (3)proxy2, (4)proxy3

    // Check proxy2 unregister
    // Proxy3 gets slashed for 2 unfinished tasks - 1 portion is returned to delegator
    let unregister_response =
        unregister_proxy(deps.as_mut(), &proxy2, DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: delegator1.to_string(),
            amount: vec![Coin::new(
                per_proxy_task_reward_amount * 2,
                stake_denom.as_str(),
            )],
        })
    );
    assert_eq!(
        unregister_response.messages[1],
        SubMsg::new(BankMsg::Send {
            to_address: proxy2.to_string(),
            amount: vec![Coin::new(
                2 * minimum_proxy_stake_amount - 2 * per_task_slash_stake_amount,
                stake_denom.as_str(),
            )],
        })
    );

    // Requests:
    // req1 - (0)proxy1, (2)proxy3
    // req2 - (4)proxy3 - Unfinishable

    // Check if only tasks for request2 are deleted
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy1).len(),
        1
    );
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy2).len(),
        0
    );
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy3).len(),
        2
    );

    // Check proxy3 unregister
    // Proxy3 gets slashed for 1 unfinished task, delegator gets full refund for request that cannot be completed
    let unregister_response =
        unregister_proxy(deps.as_mut(), &proxy3, DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: delegator1.to_string(),
            amount: vec![Coin::new(
                per_proxy_task_reward_amount * 3,
                stake_denom.as_str(),
            )],
        })
    );
    assert_eq!(
        unregister_response.messages[1],
        SubMsg::new(BankMsg::Send {
            to_address: proxy3.to_string(),
            amount: vec![Coin::new(
                3 * minimum_proxy_stake_amount - 2 * per_task_slash_stake_amount,
                stake_denom.as_str(),
            )],
        })
    );

    // Requests:
    // req1 - deleted
    // req2 - deleted

    // Check proxy tasks
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy1).len(),
        1
    );
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy2).len(),
        0
    );
    assert_eq!(
        store_get_all_proxy_tasks_in_queue(deps.as_mut().storage, &proxy3).len(),
        0
    );

    // Check proxy1 unregister
    // Proxy1 gets slashed for not finishing taskss
    let unregister_response =
        unregister_proxy(deps.as_mut(), &proxy1, DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(
        unregister_response.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                minimum_proxy_stake_amount - per_task_slash_stake_amount,
                stake_denom.as_str(),
            )],
        })
    );
}

#[test]
fn test_get_n_minimum_proxies_for_refund() {
    let mut state = State {
        admin: Addr::unchecked("admin"),
        threshold: 123,
        next_proxy_task_id: 0,
        next_delegation_id: 0,
        proxy_whitelisting: false,
        terminated: false,
    };
    let mut staking_config = StakingConfig {
        stake_denom: "denom".to_string(),
        minimum_proxy_stake_amount: Uint128::new(0),
        per_proxy_task_reward_amount: Uint128::new(0),
        per_task_slash_stake_amount: Uint128::new(0),
    };

    // zero division case
    staking_config.per_proxy_task_reward_amount = Uint128::new(100);
    staking_config.per_task_slash_stake_amount = Uint128::new(0);
    state.threshold = 123;
    assert_eq!(
        get_n_minimum_proxies_for_refund(&state, &staking_config),
        123
    );

    staking_config.per_proxy_task_reward_amount = Uint128::new(100);
    staking_config.per_task_slash_stake_amount = Uint128::new(100);
    state.threshold = 3;
    assert_eq!(get_n_minimum_proxies_for_refund(&state, &staking_config), 4);

    staking_config.per_proxy_task_reward_amount = Uint128::new(100);
    staking_config.per_task_slash_stake_amount = Uint128::new(100);
    state.threshold = 123;
    assert_eq!(
        get_n_minimum_proxies_for_refund(&state, &staking_config),
        244
    );

    staking_config.per_proxy_task_reward_amount = Uint128::new(100);
    staking_config.per_task_slash_stake_amount = Uint128::new(50);
    state.threshold = 123;
    assert_eq!(
        get_n_minimum_proxies_for_refund(&state, &staking_config),
        366
    );

    staking_config.per_proxy_task_reward_amount = Uint128::new(50);
    staking_config.per_task_slash_stake_amount = Uint128::new(100);
    state.threshold = 123;
    assert_eq!(
        get_n_minimum_proxies_for_refund(&state, &staking_config),
        183
    );

    staking_config.per_proxy_task_reward_amount = Uint128::new(1);
    staking_config.per_task_slash_stake_amount = Uint128::new(121);
    state.threshold = 123;
    assert_eq!(
        get_n_minimum_proxies_for_refund(&state, &staking_config),
        124
    );

    staking_config.per_proxy_task_reward_amount = Uint128::new(1);
    staking_config.per_task_slash_stake_amount = Uint128::new(122);
    state.threshold = 123;
    assert_eq!(
        get_n_minimum_proxies_for_refund(&state, &staking_config),
        123
    );

    staking_config.per_proxy_task_reward_amount = Uint128::new(1);
    staking_config.per_task_slash_stake_amount = Uint128::new(1000);
    state.threshold = 10;
    assert_eq!(
        get_n_minimum_proxies_for_refund(&state, &staking_config),
        10
    );

    staking_config.per_proxy_task_reward_amount = Uint128::new(1000);
    staking_config.per_task_slash_stake_amount = Uint128::new(1);
    state.threshold = 10;
    assert_eq!(
        get_n_minimum_proxies_for_refund(&state, &staking_config),
        9009
    );

    // Large numbers check
    staking_config.per_proxy_task_reward_amount = Uint128::new(100000000000000000000);
    staking_config.per_task_slash_stake_amount = Uint128::new(1000000000000);
    state.threshold = 10;
    assert_eq!(
        get_n_minimum_proxies_for_refund(&state, &staking_config),
        900000009
    );

    staking_config.per_proxy_task_reward_amount = Uint128::new(100);
    staking_config.per_task_slash_stake_amount = Uint128::new(100);
    state.threshold = 1;
    assert_eq!(get_n_minimum_proxies_for_refund(&state, &staking_config), 1);

    staking_config.per_proxy_task_reward_amount = Uint128::new(100);
    staking_config.per_task_slash_stake_amount = Uint128::new(100);
    state.threshold = 2;
    assert_eq!(get_n_minimum_proxies_for_refund(&state, &staking_config), 2);
}

/*
#[test]
fn test_verify_fragments() {
    assert!(verify_fragment(
        &FRAGMENT_P1_DR1_DE1.to_string(),
        &CAPSULE.to_string(),
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
    )
    .is_ok());

    assert!(verify_fragment(
        &FRAGMENT_P1_DR1_DE2.to_string(),
        &CAPSULE.to_string(),
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
    )
    .is_err());
}
 */

#[test]
fn test_timeouts() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());
    let proxy3 = Addr::unchecked("proxy_3".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");

    let data_id1 = String::from("DATA1");
    let data_id2 = String::from("DATA2");
    let delegation_string = String::from("DELESTRING");

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let minimum_proxy_stake_amount: u128 = 100;
    let per_proxy_task_reward_amount: u128 = 40;
    let per_task_slash_stake_amount: u128 = 98;

    // Timetous
    let timeout_height: u64 = 100;

    let request_reward = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(DEFAULT_TASK_REWARD_AMOUNT * 3),
    }];

    // Scenario

    /*************** Initialise *************/
    let proxies: Vec<Addr> = vec![proxy1.clone(), proxy2.clone(), proxy3.clone()];
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        0,
        &Some(2),
        &None,
        &Some(proxies),
        &stake_denom,
        &Some(Uint128::new(minimum_proxy_stake_amount)),
        &Some(Uint128::new(per_proxy_task_reward_amount)),
        &Some(Uint128::new(per_task_slash_stake_amount)),
        &Some(timeout_height),
        &None,
    )
    .is_ok());

    let proxy_register_stake = vec![Coin {
        denom: stake_denom.clone(),
        amount: Uint128::new(DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT),
    }];
    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        0,
        &proxy1_pubkey,
        &proxy_register_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        0,
        &proxy2_pubkey,
        &proxy_register_stake,
    )
    .is_ok());
    assert!(register_proxy(
        deps.as_mut(),
        &proxy3,
        0,
        &proxy3_pubkey,
        &proxy_register_stake,
    )
    .is_ok());

    /*************** Add data and delegations by delegator *************/
    // Add data by delegator
    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        0,
        &data_id1,
        &DELEGATOR1_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());

    assert!(add_data(
        deps.as_mut(),
        &delegator2,
        0,
        &data_id2,
        &DELEGATOR2_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());

    // Add delegations
    let delegation1: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_addr: proxy1.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy2.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy3.clone(),
            delegation_string: delegation_string.clone(),
        },
    ];

    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &delegation1,
    )
    .is_ok());

    let delegation2: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_addr: proxy1.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy3.clone(),
            delegation_string: delegation_string.clone(),
        },
    ];

    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE2_PUBKEY.to_string(),
        &delegation2,
    )
    .is_ok());

    let delegation3: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_addr: proxy1.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy2.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy3.clone(),
            delegation_string: delegation_string,
        },
    ];

    assert!(add_delegation(
        deps.as_mut(),
        &delegator2,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR2_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &delegation3,
    )
    .is_ok());

    /*************** Add and check re-encryption requests *************/

    // Requests:
    // req1 - (0)proxy1, (1)proxy2, (2)proxy3  - timeout at 300
    // req2 - (3)proxy1,            (4)proxy3  - timeout at 350
    // req3 - (4)proxy1, (5)proxy2, (6)proxy3  - timeout at 400

    // Request 1 will timeout at 200+100
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        200,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward,
    )
    .is_ok());

    // Complete request1 before timeout
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy1,
        220,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &FRAGMENT_P1_DR1_DE1.to_string(),
    )
    .is_ok());
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy2,
        220,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &FRAGMENT_P2_DR1_DE1.to_string(),
    )
    .is_ok());

    let state = store_get_state(deps.as_mut().storage).unwrap();
    // Check if request is completed
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id1,
            DELEGATEE1_PUBKEY,
            &220u64,
        ),
        ReencryptionRequestState::Granted
    );
    // Check if request1 state stays granted after timeout
    let state = store_get_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id1,
            DELEGATEE1_PUBKEY,
            &400u64,
        ),
        ReencryptionRequestState::Granted
    );

    // Check request to be checked pointer
    assert_eq!(
        store_get_timeouts_config(deps.as_mut().storage)
            .unwrap()
            .next_task_id_to_be_checked,
        1
    );

    // Request 2 will timeout at 250+100
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        250,
        &data_id1,
        &DELEGATEE2_PUBKEY.to_string(),
        &request_reward,
    )
    .is_ok());

    // Check request 2 timeout height
    let state = store_get_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id1,
            DELEGATEE2_PUBKEY,
            &350u64,
        ),
        ReencryptionRequestState::TimedOut
    );

    // Check task to be checked pointer
    assert_eq!(
        store_get_timeouts_config(deps.as_mut().storage)
            .unwrap()
            .next_task_id_to_be_checked,
        2
    );

    // Height = 250
    // Current states of requests/tasks:
    // req1 - (0)proxy1, (1)proxy2, (2)proxy3  - timeout at 300 - Completed
    // req2 - (3)proxy1,            (4)proxy3  - timeout at 350 - Ready
    // req3 - (4)proxy1, (5)proxy2, (6)proxy3  - timeout at 400 - Will be created at height 300

    // Request 3 will timeout at 300+100
    assert!(request_reencryption(
        deps.as_mut(),
        &delegator2,
        300,
        &data_id2,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward,
    )
    .is_ok());

    // Check if task to be checked pointer moved
    assert_eq!(
        store_get_timeouts_config(deps.as_mut().storage)
            .unwrap()
            .next_task_id_to_be_checked,
        3
    );

    // Height = 300
    // Current states of requests/tasks:
    // req1 - (0)proxy1, (1)proxy2, (2)proxy3  - timeout at 300 - Completed
    // req2 - (3)proxy1,            (4)proxy3  - timeout at 350 - Ready
    // req3 - (4)proxy1, (5)proxy2, (6)proxy3  - timeout at 400 - Ready

    // Unregister proxy after timeout
    // Request gets first timed out and then abandoned (will be displayed as TimedOut)
    let res = unregister_proxy(deps.as_mut(), &proxy3, 350).unwrap();
    assert_eq!(res.messages.len(), 2);
    // Timeout BankMsgs
    // Funds from timed out request 2 are sent back to delegator1
    assert_eq!(
        res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: delegator1.to_string(),
            amount: vec![Coin::new(
                2 * per_proxy_task_reward_amount,
                stake_denom.as_str(),
            )],
        })
    );
    // Unregister BankMsgs
    // Proxy 3 got slashed for 3 unfinished tasks
    assert_eq!(
        res.messages[1],
        SubMsg::new(BankMsg::Send {
            to_address: proxy3.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 3 * per_task_slash_stake_amount,
                stake_denom.as_str(),
            )],
        })
    );

    let state = store_get_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id1,
            &DELEGATEE2_PUBKEY.to_string(),
            &350u64,
        ),
        ReencryptionRequestState::TimedOut
    );

    // Height = 350
    // Current states of requests/tasks:
    // Proxy 3 unregistered
    // req1 - (0)  proxy1,  (1)proxy2,   (2)  proxy3  - timeout at 300 - Completed
    // req2 - (x3x)proxy1,               (x4x)proxy3  - timeout at 350 - TimedOut (Abandoned)
    // req3 - (4)  proxy1,  (5)proxy2,   (x6x)proxy3  - timeout at 400 - Ready

    // Cannot provide fragment for timed-out request2
    assert!(is_err(
        provide_reencrypted_fragment(
            deps.as_mut(),
            &proxy1,
            350,
            &data_id1,
            &DELEGATEE2_PUBKEY.to_string(),
            &FRAGMENT_P1_DR1_DE2.to_string(),
        ),
        "Request timed out.",
    ));

    // Tasks can be obtained before timeout
    assert!(!get_proxy_tasks(deps.as_mut().storage, &proxy1, &350)
        .unwrap()
        .is_empty());
    assert!(!get_proxy_tasks(deps.as_mut().storage, &proxy2, &350)
        .unwrap()
        .is_empty());
    assert!(get_proxy_tasks(deps.as_mut().storage, &proxy3, &350)
        .unwrap()
        .is_empty());

    // No tasks to complete after timeout
    // These tasks still exist but are skipped in get_proxy_tasks
    // These tasks will be removed when ExecuteMsg triggers check_and_resolve_all_timedout_tasks
    assert!(get_proxy_tasks(deps.as_mut().storage, &proxy1, &500)
        .unwrap()
        .is_empty());
    assert!(get_proxy_tasks(deps.as_mut().storage, &proxy2, &500)
        .unwrap()
        .is_empty());
    assert!(get_proxy_tasks(deps.as_mut().storage, &proxy3, &500)
        .unwrap()
        .is_empty());

    // Any ExecuteMSG call at height=500 timeouts the rest of requests
    let stake_to_add = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(1),
    }];
    let res = add_stake(deps.as_mut(), &proxy1, 500, &stake_to_add).unwrap();
    assert_eq!(res.messages.len(), 1);
    // Delegator 2 gets a refund.
    assert_eq!(
        res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: delegator2.to_string(),
            amount: vec![Coin::new(
                3 * per_proxy_task_reward_amount,
                stake_denom.as_str(),
            )],
        })
    );

    // Height = 500
    // Current states of requests:
    // Proxy 3 unregistered
    // req1 - (0)  proxy1,  (1)proxy2,       (2)  proxy3  - timeout at 300 - Completed
    // req2 - (x3x)proxy1,                   (x4x)proxy3  - timeout at 350 - TimedOut (Abandoned)
    // req3 - (x4x)  proxy1,  (x5x)proxy2,   (x6x)proxy3  - timeout at 400 - TimedOut

    // Check if all tasks from queue were deleted due to timeout
    assert!(get_proxy_tasks(deps.as_mut().storage, &proxy1, &0)
        .unwrap()
        .is_empty());
    assert!(get_proxy_tasks(deps.as_mut().storage, &proxy2, &0)
        .unwrap()
        .is_empty());
    assert!(get_proxy_tasks(deps.as_mut().storage, &proxy3, &0)
        .unwrap()
        .is_empty());

    // Unregister proxy 1
    let res = unregister_proxy(deps.as_mut(), &proxy1, 500).unwrap();
    assert_eq!(res.messages.len(), 1);
    // Proxy 1 got slashed for 2 tasks, rewarded for 1 task and additional stake 1 atestfet was added by previous call
    assert_eq!(
        res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy1.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - 2 * per_task_slash_stake_amount
                    + per_proxy_task_reward_amount
                    + 1,
                stake_denom.as_str(),
            )],
        })
    );

    // Unregister proxy 2
    let res = unregister_proxy(deps.as_mut(), &proxy2, 500).unwrap();
    assert_eq!(res.messages.len(), 1);
    // Proxy 1 got slashed for 1 task and rewarded for 1 task
    assert_eq!(
        res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: proxy2.to_string(),
            amount: vec![Coin::new(
                DEFAULT_MINIMUM_PROXY_STAKE_AMOUNT - per_task_slash_stake_amount
                    + per_proxy_task_reward_amount,
                stake_denom.as_str(),
            )],
        })
    );
}

#[test]
fn test_terminate_contract() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());
    let proxy3 = Addr::unchecked("proxy_3".to_string());
    let recipient = Addr::unchecked("recipient".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());
    let delegator2 = Addr::unchecked("delegator2".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");

    let data_id1 = String::from("DATA1");
    let delegation_string = String::from("DELESTRING");

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let minimum_proxy_stake_amount: u128 = 200;
    let per_proxy_task_reward_amount: u128 = 40;
    let per_task_slash_stake_amount: u128 = 98;

    let request_reward = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(per_proxy_task_reward_amount * 3),
    }];

    let proxy_stake = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(minimum_proxy_stake_amount),
    }];

    // Timetous
    let timeout_height: u64 = 100;

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        0,
        &Some(1),
        &None,
        &None,
        &stake_denom,
        &Some(Uint128::new(minimum_proxy_stake_amount)),
        &Some(Uint128::new(per_proxy_task_reward_amount)),
        &Some(Uint128::new(per_task_slash_stake_amount)),
        &Some(timeout_height),
        &Some(false),
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());

    // Prepare scenario
    // Proxy 1,2 gets 2 re-encryption tasks each

    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATOR1_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());

    let proxy_delegations: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_addr: proxy1.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy2.clone(),
            delegation_string: delegation_string,
        },
    ];

    // Add delegations
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward,
    )
    .is_ok());

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE2_PUBKEY.to_string(),
        &request_reward,
    )
    .is_ok());

    // Both proxies are available
    assert_eq!(get_proxies_availability(deps.as_mut().storage).len(), 2);

    // Try to withdraw contract
    assert!(is_err(
        withdraw_contract(deps.as_mut(), &delegator1, DEFAULT_BLOCK_HEIGHT, &recipient),
        "Only admin can execute this method.",
    ));

    assert!(is_err(
        withdraw_contract(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT, &recipient),
        "Contract not terminated",
    ));

    // Terminate contract
    assert!(is_err(
        terminate_contract(deps.as_mut(), &delegator1, DEFAULT_BLOCK_HEIGHT),
        "Only admin can execute this method.",
    ));
    assert!(terminate_contract(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT).is_ok());
    assert!(is_err(
        terminate_contract(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT),
        "Contract was terminated.",
    ));

    // No proxies are available
    assert!(get_proxies_availability(deps.as_mut().storage).is_empty());

    let proxy_entry: Proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy1).unwrap();
    assert_eq!(proxy_entry.state, ProxyState::Leaving);

    let proxy_entry: Proxy = store_get_proxy_entry(deps.as_mut().storage, &proxy2).unwrap();
    assert_eq!(proxy_entry.state, ProxyState::Leaving);

    // Check if contract state is terminated
    let state: State = store_get_state(deps.as_mut().storage).unwrap();
    assert!(state.terminated);

    assert!(is_err(
        add_delegation(
            deps.as_mut(),
            &delegator2,
            DEFAULT_BLOCK_HEIGHT,
            &DELEGATOR2_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string(),
            &proxy_delegations,
        ),
        "Contract was terminated.",
    ));

    assert!(is_err(
        register_proxy(
            deps.as_mut(),
            &proxy3,
            DEFAULT_BLOCK_HEIGHT,
            &proxy3_pubkey,
            &proxy_stake,
        ),
        "Contract was terminated.",
    ));

    // Proxies still can finish their jobs
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &FRAGMENT_P1_DR1_DE1.to_string(),
    )
    .is_ok());

    // There are pending requests at DEFAULT_BLOCK_HEIGHT
    assert!(is_err(
        withdraw_contract(deps.as_mut(), &creator, DEFAULT_BLOCK_HEIGHT, &recipient),
        "There are requests to be resolved",
    ));

    // All tasks time out at DEFAULT_BLOCK_HEIGHT+timeout_height
    assert!(is_err(
        withdraw_contract(
            deps.as_mut(),
            &creator,
            DEFAULT_BLOCK_HEIGHT + timeout_height,
            &recipient,
        ),
        "Nothing to withdraw",
    ));

    // Add balance to contract
    let contract_balance: Vec<Coin> = vec![
        Coin {
            denom: "something".to_string(),
            amount: Uint128::new(123),
        },
        Coin {
            denom: stake_denom,
            // amount is equal to remaining proxy stake amount
            amount: Uint128::new(2 * minimum_proxy_stake_amount - 254),
        },
    ];
    deps.querier
        .update_balance(mock_env().contract.address, contract_balance);

    // Withdrawing is now possible
    let res = withdraw_contract(
        deps.as_mut(),
        &creator,
        DEFAULT_BLOCK_HEIGHT + timeout_height,
        &recipient,
    )
    .unwrap();

    // Return remaining stake to recipient
    assert_eq!(
        res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: [
                Coin {
                    denom: "something".to_string(),
                    amount: Uint128::new(123),
                },
                // proxy stake got subtracted
            ]
            .to_vec(),
        })
    );
}

#[test]
fn test_skip_task() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());
    let proxy3 = Addr::unchecked("proxy_3".to_string());
    let proxy4 = Addr::unchecked("proxy_4".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");
    let proxy4_pubkey: String = String::from("proxy_pubkey4");

    let data_id1 = String::from("DATA1");
    let delegation_string = String::from("DELESTRING");

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let minimum_proxy_stake_amount: u128 = 200;
    let per_proxy_task_reward_amount: u128 = 40;
    let per_task_slash_stake_amount: u128 = 98;

    let request_reward = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(per_proxy_task_reward_amount * 3),
    }];

    let proxy_stake = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(minimum_proxy_stake_amount),
    }];

    // Timetous
    let timeout_height: u64 = 100;

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        0,
        &Some(2),
        &None,
        &None,
        &stake_denom,
        &Some(Uint128::new(minimum_proxy_stake_amount)),
        &Some(Uint128::new(per_proxy_task_reward_amount)),
        &Some(Uint128::new(per_task_slash_stake_amount)),
        &Some(timeout_height),
        &Some(false),
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy3,
        DEFAULT_BLOCK_HEIGHT,
        &proxy3_pubkey,
        &proxy_stake,
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy4,
        DEFAULT_BLOCK_HEIGHT,
        &proxy4_pubkey,
        &proxy_stake,
    )
    .is_ok());

    // Prepare scenario
    // Proxy 1,2,3 gets2 re-encryption tasks each

    assert!(add_data(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATOR1_PUBKEY.to_string(),
        &CAPSULE.to_string(),
    )
    .is_ok());

    let proxy_delegations: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_addr: proxy1.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy2.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy3.clone(),
            delegation_string: delegation_string,
        },
    ];

    // Add delegations
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE2_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &request_reward,
    )
    .is_ok());

    assert!(request_reencryption(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE2_PUBKEY.to_string(),
        &request_reward,
    )
    .is_ok());

    // Not a proxy
    assert!(is_err(
        skip_reencryption_task(
            deps.as_mut(),
            &creator,
            DEFAULT_BLOCK_HEIGHT,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        "Sender is not a proxy",
    ));

    // Incorrect task
    assert!(is_err(
        skip_reencryption_task(
            deps.as_mut(),
            &proxy4,
            DEFAULT_BLOCK_HEIGHT,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        "Task doesn't exist",
    ));

    let p1_tasks = get_proxy_tasks(deps.as_mut().storage, &proxy1, &DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(p1_tasks.len(), 2);
    let p2_tasks = get_proxy_tasks(deps.as_mut().storage, &proxy2, &DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(p2_tasks.len(), 2);
    let p3_tasks = get_proxy_tasks(deps.as_mut().storage, &proxy3, &DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(p3_tasks.len(), 2);

    // Remove task for data1 by proxy1 for delegatee1
    assert!(skip_reencryption_task(
        deps.as_mut(),
        &proxy1,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
    )
    .is_ok());

    // Check request state
    let state = store_get_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &DEFAULT_BLOCK_HEIGHT,
        ),
        ReencryptionRequestState::Ready
    );

    let p1_tasks = get_proxy_tasks(deps.as_mut().storage, &proxy1, &DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(p1_tasks.len(), 1);
    let p2_tasks = get_proxy_tasks(deps.as_mut().storage, &proxy2, &DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(p2_tasks.len(), 2);
    let p3_tasks = get_proxy_tasks(deps.as_mut().storage, &proxy3, &DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(p3_tasks.len(), 2);

    // Remove task for data1 by proxy3 for delegatee1
    assert!(skip_reencryption_task(
        deps.as_mut(),
        &proxy3,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
    )
    .is_ok());

    // Task is Abandoned
    let state = store_get_state(deps.as_mut().storage).unwrap();
    assert_eq!(
        get_reencryption_request_state(
            deps.as_mut().storage,
            &state,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
            &DEFAULT_BLOCK_HEIGHT,
        ),
        ReencryptionRequestState::Abandoned
    );

    let p1_tasks = get_proxy_tasks(deps.as_mut().storage, &proxy1, &DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(p1_tasks.len(), 1);
    let p2_tasks = get_proxy_tasks(deps.as_mut().storage, &proxy2, &DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(p2_tasks.len(), 2);
    let p3_tasks = get_proxy_tasks(deps.as_mut().storage, &proxy3, &DEFAULT_BLOCK_HEIGHT).unwrap();
    assert_eq!(p3_tasks.len(), 1);

    // Proxy 3 finish the task
    assert!(provide_reencrypted_fragment(
        deps.as_mut(),
        &proxy2,
        DEFAULT_BLOCK_HEIGHT,
        &data_id1,
        &DELEGATEE1_PUBKEY.to_string(),
        &FRAGMENT_P2_DR1_DE1.to_string(),
    )
    .is_ok());

    // Can't skip finished task
    assert!(is_err(
        skip_reencryption_task(
            deps.as_mut(),
            &proxy2,
            DEFAULT_BLOCK_HEIGHT,
            &data_id1,
            &DELEGATEE1_PUBKEY.to_string(),
        ),
        "Task was already completed.",
    ));
}

#[test]
fn test_remove_proxies_from_delegation() {
    let mut deps = mock_dependencies();

    // Addresses
    let creator = Addr::unchecked("creator".to_string());
    let proxy1 = Addr::unchecked("proxy_1".to_string());
    let proxy2 = Addr::unchecked("proxy_2".to_string());
    let proxy3 = Addr::unchecked("proxy_3".to_string());

    let delegator1 = Addr::unchecked("delegator1".to_string());

    // Pubkeys
    let proxy1_pubkey: String = String::from("proxy_pubkey1");
    let proxy2_pubkey: String = String::from("proxy_pubkey2");
    let proxy3_pubkey: String = String::from("proxy_pubkey3");

    let delegation_string = String::from("DELESTRING");

    // Staking
    let stake_denom = DEFAULT_STAKE_DENOM.to_string();
    let minimum_proxy_stake_amount: u128 = 200;
    let per_proxy_task_reward_amount: u128 = 40;
    let per_task_slash_stake_amount: u128 = 98;

    let proxy_stake = vec![Coin {
        denom: DEFAULT_STAKE_DENOM.to_string(),
        amount: Uint128::new(minimum_proxy_stake_amount),
    }];

    // Timetous
    let timeout_height: u64 = 100;

    /*************** Initialise *************/
    assert!(init_contract(
        deps.as_mut(),
        &creator,
        0,
        &Some(2),
        &None,
        &None,
        &stake_denom,
        &Some(Uint128::new(minimum_proxy_stake_amount)),
        &Some(Uint128::new(per_proxy_task_reward_amount)),
        &Some(Uint128::new(per_task_slash_stake_amount)),
        &Some(timeout_height),
        &Some(false),
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy1.clone(),
        DEFAULT_BLOCK_HEIGHT,
        &proxy1_pubkey,
        &proxy_stake,
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy2.clone(),
        DEFAULT_BLOCK_HEIGHT,
        &proxy2_pubkey,
        &proxy_stake,
    )
    .is_ok());

    assert!(register_proxy(
        deps.as_mut(),
        &proxy3,
        DEFAULT_BLOCK_HEIGHT,
        &proxy3_pubkey,
        &proxy_stake,
    )
    .is_ok());

    let proxy_delegations: Vec<ProxyDelegationString> = vec![
        ProxyDelegationString {
            proxy_addr: proxy1.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy2.clone(),
            delegation_string: delegation_string.clone(),
        },
        ProxyDelegationString {
            proxy_addr: proxy3.clone(),
            delegation_string: delegation_string,
        },
    ];

    // Add delegation
    assert!(add_delegation(
        deps.as_mut(),
        &delegator1,
        DEFAULT_BLOCK_HEIGHT,
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
        &proxy_delegations,
    )
    .is_ok());

    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string()
        ),
        DelegationState::Active
    );

    // Delegation still exists when proxy 1 leaves
    assert!(unregister_proxy(deps.as_mut(), &proxy1, DEFAULT_BLOCK_HEIGHT).is_ok());
    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string()
        ),
        DelegationState::Active
    );

    // Delegation gets deleted when number of proxies is less than minimum
    assert!(unregister_proxy(deps.as_mut(), &proxy2, DEFAULT_BLOCK_HEIGHT).is_ok());

    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string()
        ),
        DelegationState::NonExisting
    );

    assert!(unregister_proxy(deps.as_mut(), &proxy3, DEFAULT_BLOCK_HEIGHT).is_ok());

    assert_eq!(
        get_delegation_state(
            deps.as_mut().storage,
            &DELEGATOR1_PUBKEY.to_string(),
            &DELEGATEE1_PUBKEY.to_string()
        ),
        DelegationState::NonExisting
    );
}
