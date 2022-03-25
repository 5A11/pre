use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::contract::{instantiate, query, verify_fragment};

// FRAGMENT_VERIFICATION
// use crate::contract::verify_fragment
use crate::msg::{InstantiateMsg, QueryMsg};

// Test constants
const DELEGATOR1_PUBKEY: &str = "ApEPhAeq+TAL5aKiRkIpdoJ2pD+6qSt1RqHxGthT+XRY";
const DELEGATEE1_PUBKEY: &str = "A5CYTfwD0EocpW4gCKtnP1lIFkMveO55v5+nbJaLqmLX";
const CAPSULE: &str = "Ax83HFfEW1e+DW3KlikFLELPOVqYnlS39baHHC+/vsB4AmV+m1r9eZ6nCV9KXv7dSH+bSdWFbsqWFTxfF5qsjwObLtgsZUVSt8iv8UtkP0bLJs2sguElu4Syek6Seh3ZTj4=";
const FRAGMENT_P1_DR1_DE1: &str = "Agn8MTBWSKzz277FLeNKvhOwa3juw7HBciLmyA/3kZ2hAtQv0l/B+Ej2vQLxZDx+MHDr5uevth9PzntoIz6gbPI1xJk3dVwZohs3YgdaXJsBXpAambF1FpOGrola7KcwjtQDOL6tYr3e6dlMgsW9GnONyZUWk15ixjxdrAIZfp8qWAMCbOd9fCO820cnEqBeQHpit75l8gxb6Al3s28p4uMFeq4Dzsh5SbQgRk7KjI9LEq2a9YzQ2ts3O5KEx3SuZoCOE0UDns625ayBRPD5BHdYwGaCGo/w6oJ5PvRp7rEpMSvxpOACu5HXcj2KNZnzAc2QGNrHmrAIxxS4pUbp7ffoPjSK/eGOs3Yh2IaeLQMzj2FNpUCYii6D3KJMT5sWqdKQV+5Aw6ebgujLY0o4Gs2aJ3toE3GuNuSfwFKzySmpq5CfSGaJJftZDYt72g7t8cRKVFXT6D8ugCXfMVL6GRE7adJEkYU=";
const FRAGMENT_P1_DR1_DE2: &str = "A5fWxYyjkfJu/k2oq5A6w+pLgRtWRIKu2uEHe/i0AGSiAsK7jq0a7KjTeiBCBRTC64ATDb/QfYQ9CBoiF5FDdbJwh3Yb5RoZgkclP0cqNtftZnRCdVUuycy2UpQ7f4x5tFkCkymfOr+pOYAe56kPnK9cTGjuwGdgcrV3i5A1ocF5xYsD0mFd9APeYHeRjAIgPzM3na8xJuYgSdY9FA6upZOYqxkDGWHWjB6Uvjby3zTbN+A8vuQmHRx0NST4ICR5HfKCCPQCE6/D/Dep4lyf4v9E03VgMisZKFWW7+YP5qAIWgNDSPoCb8OqfDTrYzJSKGZg+ti4l/Cjo5PaqmlZlj1MCR/Rb906nywBtIQCU9iVAHGUnE8h9QV+kYWik4s2Vcq2W6r/Y6MXnYLK9JsYpIwny6zDwHzOwfTk4Wn9mLGENf4q2s/5ZeM/1TJIu6wECI+L5zmGMWl40iloHhHcxeCVKwKnM0s=";

fn mock_env_height(signer: &Addr, height: u64, coins: &Vec<Coin>) -> (Env, MessageInfo) {
    let mut env = mock_env();
    env.block.height = height;
    let info = mock_info(signer.as_str(), &coins);

    return (env, info);
}

/*
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
*/

fn init_contract(deps: DepsMut, creator: &Addr, init_msg: InstantiateMsg) -> StdResult<Response> {
    let env = mock_env_height(&creator, 0, &vec![]);
    return instantiate(deps, env.0, env.1, init_msg);
}

fn query_verify_fragment(
    deps: Deps,
    fragment: &String,
    capsule: &String,
    delegator_pubkey: &String,
    delegatee_pubkey: &String,
) -> StdResult<Binary> {
    let env = mock_env();

    let msg = QueryMsg::VerifyFragment {
        fragment: fragment.clone(),
        capsule: capsule.clone(),
        delegator_pubkey: delegator_pubkey.clone(),
        delegatee_pubkey: delegatee_pubkey.clone(),
    };

    return query(deps, env, msg);
}

#[test]
fn test_new_contract_default_values() {
    let mut deps = mock_dependencies();
    let creator = Addr::unchecked("creator".to_string());

    assert!(init_contract(deps.as_mut(), &creator, InstantiateMsg {}).is_ok());

    let res = query_verify_fragment(
        deps.as_ref(),
        &FRAGMENT_P1_DR1_DE1.to_string(),
        &CAPSULE.to_string(),
        &DELEGATOR1_PUBKEY.to_string(),
        &DELEGATEE1_PUBKEY.to_string(),
    );

    assert_eq!(res.unwrap().as_slice(), b"{\"valid\":true}");
}

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
