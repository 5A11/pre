use crate::msg::{InstantiateMsg, QueryMsg, VerifyFragmentResponse};

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

// FRAGMENT_VERIFICATION
use umbral_pre::{Capsule, CapsuleFrag, DeserializableFromArray, PublicKey};

macro_rules! generic_err {
    ($val:expr) => {
        Err(StdError::generic_err($val))
    };
}

pub const FRAGMENT_VERIFICATION_ERROR: &str = "Fragment verification failed: ";

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerifyFragment {
            fragment,
            capsule,
            delegator_pubkey,
            delegatee_pubkey,
        } => Ok(to_binary(&VerifyFragmentResponse {
            valid: verify_fragment(&fragment, &capsule, &delegator_pubkey, &delegatee_pubkey)
                .is_ok(),
        })?),
    }
}

// FRAGMENT_VERIFICATION
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
