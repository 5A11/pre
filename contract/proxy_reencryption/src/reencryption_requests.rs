use crate::common::add_bank_msg;
use crate::state::{
    store_get_staking_config, store_get_state, store_get_timeouts_config,
    store_set_timeouts_config, State, TimeoutsConfig,
};
use cosmwasm_std::{
    from_slice, to_vec, Addr, Order, Response, StdError, StdResult, Storage, Uint128,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

// Map proxy_reencryption_request_id: u64 -> request: ProxyReencryptionRequest
static PROXY_REENCRYPTION_REQUESTS_STORE_KEY: &[u8] = b"ProxyReencryptionRequests";

// Delegatee side to lookup fragments
// Map data_id: String -> delegatee_pubkey: String -> proxy_pubkey: String -> proxy_reencryption_request_id: u64
static DELEGATEE_PROXY_REQUESTS_STORE_KEY: &[u8] = b"DelegateeProxyRequests";

// Proxy side to lookup active tasks
// Map proxy_pubkey: String -> proxy_reencryption_request_id: u64 -> is_request: bool
static PROXY_REQUESTS_QUEUE_STORE_KEY: &[u8] = b"ProxyRequestsQueue";

// Parent requests
// Map data_id: String -> delegatee_pubkey: String  -> request: ParentReencryptionRequest
static PARENT_REENCRYPTION_REQUESTS_STORE_KEY: &[u8] = b"ParentReencryptionRequests";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyReencryptionRequest {
    // To find parent request
    pub data_id: String,
    pub delegatee_pubkey: String,

    // Individual request parameters
    pub proxy_pubkey: String,
    pub fragment: Option<String>,
    pub delegation_string: String,
}

// Parent re-encryption request
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ParentReencryptionRequest {
    pub n_provided_fragments: u32,
    pub n_proxy_requests: u32,
    pub state: ReencryptionRequestState,

    // Incentives
    pub delegator_provided_reward_amount: Uint128,
    pub funds_pool: Uint128,
    // Reward will be returned from funds pool to this address when request cannot be completed
    pub delegator_addr: Addr,

    // Timeouts
    pub timeout_height: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReencryptionRequestState {
    Inaccessible,
    Ready,
    Granted,
    Abandoned,
    TimedOut,
}

// PROXY_REENCRYPTION_REQUESTS_STORE_KEY
pub fn store_set_proxy_reencryption_request(
    storage: &mut dyn Storage,
    proxy_reencryption_request_id: &u64,
    reencryption_request: &ProxyReencryptionRequest,
) {
    let mut store = PrefixedStorage::new(storage, PROXY_REENCRYPTION_REQUESTS_STORE_KEY);

    store.set(
        &proxy_reencryption_request_id.to_le_bytes(),
        &to_vec(reencryption_request).unwrap(),
    );
}

pub fn store_get_proxy_reencryption_request(
    storage: &dyn Storage,
    proxy_reencryption_request_id: &u64,
) -> Option<ProxyReencryptionRequest> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXY_REENCRYPTION_REQUESTS_STORE_KEY);

    store
        .get(&proxy_reencryption_request_id.to_le_bytes())
        .map(|data| from_slice(&data).unwrap())
}

pub fn store_remove_proxy_reencryption_request(
    storage: &mut dyn Storage,
    proxy_reencryption_request_id: &u64,
) {
    let mut store = PrefixedStorage::new(storage, PROXY_REENCRYPTION_REQUESTS_STORE_KEY);

    store.remove(&proxy_reencryption_request_id.to_le_bytes());
}

// DELEGATEE_REQUESTS_STORE
pub fn store_add_delegatee_proxy_reencryption_request(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
    proxy_reencryption_request_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_PROXY_REQUESTS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.set(
        proxy_pubkey.as_bytes(),
        &proxy_reencryption_request_id.to_le_bytes(),
    );
}

pub fn store_get_delegatee_proxy_reencryption_request(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) -> Option<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_PROXY_REQUESTS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store
        .get(proxy_pubkey.as_bytes())
        .map(|data| u64::from_le_bytes(data.try_into().unwrap()))
}

pub fn store_get_all_delegatee_proxy_reencryption_requests(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_PROXY_REQUESTS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    let mut deserialized_keys: Vec<u64> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to to_vec
        deserialized_keys.push(u64::from_le_bytes(pair.1.try_into().unwrap()));
    }

    deserialized_keys
}

pub fn store_remove_delegatee_proxy_reencryption_request(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_PROXY_REQUESTS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.remove(proxy_pubkey.as_bytes());
}

// PROXY_REQUESTS_QUEUE_STORE_KEY
pub fn store_add_proxy_reencryption_request_to_queue(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    proxy_reencryption_request_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PROXY_REQUESTS_QUEUE_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    // Any value in store means true - &[1]
    store.set(&proxy_reencryption_request_id.to_le_bytes(), &[1]);
}

pub fn store_remove_proxy_reencryption_request_from_queue(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    proxy_reencryption_request_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PROXY_REQUESTS_QUEUE_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store.remove(&proxy_reencryption_request_id.to_le_bytes());
}

pub fn store_is_proxy_reencryption_request_in_queue(
    storage: &dyn Storage,
    proxy_pubkey: &str,
    proxy_reencryption_request_id: &u64,
) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PROXY_REQUESTS_QUEUE_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store
        .get(&proxy_reencryption_request_id.to_le_bytes())
        .is_some()
}

pub fn store_get_all_proxy_reencryption_requests_in_queue(
    storage: &dyn Storage,
    proxy_pubkey: &str,
) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PROXY_REQUESTS_QUEUE_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    let mut deserialized_keys: Vec<u64> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to to_vec
        deserialized_keys.push(u64::from_le_bytes(pair.0.try_into().unwrap()));
    }

    deserialized_keys
}

// PARENT_REENCRYPTION_REQUESTS_STORE_KEY
pub fn store_set_parent_reencryption_request(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    request: &ParentReencryptionRequest,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PARENT_REENCRYPTION_REQUESTS_STORE_KEY, data_id.as_bytes()],
    );

    store.set(delegatee_pubkey.as_bytes(), &to_vec(request).unwrap());
}

pub fn store_remove_parent_reencryption_request(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PARENT_REENCRYPTION_REQUESTS_STORE_KEY, data_id.as_bytes()],
    );

    store.remove(delegatee_pubkey.as_bytes());
}

pub fn store_get_parent_reencryption_request(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> Option<ParentReencryptionRequest> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PARENT_REENCRYPTION_REQUESTS_STORE_KEY, data_id.as_bytes()],
    );

    store
        .get(delegatee_pubkey.as_bytes())
        .map(|data| from_slice(&data).unwrap())
}

// High level methods

pub fn get_reencryption_request_state(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    block_height: &u64,
) -> ReencryptionRequestState {
    match store_get_parent_reencryption_request(storage, data_id, delegatee_pubkey) {
        None => ReencryptionRequestState::Inaccessible,
        Some(parent_request) => {
            if block_height >= &parent_request.timeout_height
                && parent_request.state != ReencryptionRequestState::Granted
            {
                ReencryptionRequestState::TimedOut
            } else {
                parent_request.state
            }
        }
    }
}

pub fn remove_proxy_reencryption_request(
    storage: &mut dyn Storage,
    re_request_id: &u64,
    state: &State,
    delegator_retrieve_funds_amount: &mut HashMap<Addr, u128>,
) -> StdResult<()> {
    let re_request = store_get_proxy_reencryption_request(storage, re_request_id).unwrap();

    let mut parent_request: ParentReencryptionRequest = store_get_parent_reencryption_request(
        storage,
        &re_request.data_id,
        &re_request.delegatee_pubkey,
    )
    .unwrap();

    // Request cannot be completed any more
    if parent_request.n_proxy_requests < state.threshold + 1
        && parent_request.state != ReencryptionRequestState::Abandoned
        && parent_request.state != ReencryptionRequestState::TimedOut
    {
        // Return stake retrieved from slashed proxies to delegator stake before deleting parent request
        update_retrieved_delegator_stake_map(
            delegator_retrieve_funds_amount,
            &parent_request.delegator_addr,
            parent_request.delegator_provided_reward_amount.u128(),
        );
        parent_request.delegator_provided_reward_amount = Uint128::new(0);
        parent_request.funds_pool = parent_request
            .funds_pool
            .checked_sub(parent_request.delegator_provided_reward_amount)?;

        // Update parent request state
        parent_request.state = ReencryptionRequestState::Abandoned;
    }

    // Update parent request
    parent_request.n_proxy_requests -= 1;
    store_set_parent_reencryption_request(
        storage,
        &re_request.data_id,
        &re_request.delegatee_pubkey,
        &parent_request,
    );

    // Delete re-encryption request
    store_remove_delegatee_proxy_reencryption_request(
        storage,
        &re_request.data_id,
        &re_request.delegatee_pubkey,
        &re_request.proxy_pubkey,
    );
    store_remove_proxy_reencryption_request_from_queue(
        storage,
        &re_request.proxy_pubkey,
        re_request_id,
    );
    store_remove_proxy_reencryption_request(storage, re_request_id);

    Ok(())
}

// Delete all unfinished current proxy re-encryption requests
pub fn remove_proxy_reencryption_requests(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    response: &mut Response,
) -> StdResult<()> {
    let staking_config = store_get_staking_config(storage)?;
    let state = store_get_state(storage)?;

    let mut delegator_retrieve_funds_amount: HashMap<Addr, u128> = HashMap::new();

    for re_request_id in store_get_all_proxy_reencryption_requests_in_queue(storage, proxy_pubkey) {
        remove_proxy_reencryption_request(
            storage,
            &re_request_id,
            &state,
            &mut delegator_retrieve_funds_amount,
        )?;
    }

    // Return stake from unfinished requests to delegators
    for (delegator_addr, stake_amount) in delegator_retrieve_funds_amount {
        add_bank_msg(
            response,
            &delegator_addr,
            stake_amount,
            &staking_config.stake_denom,
        );
    }

    Ok(())
}

pub fn check_and_resolve_all_requests_timeout(
    storage: &mut dyn Storage,
    response: &mut Response,
    block_height: u64,
) {
    // Check and resolve all requests between next_request_id_to_be_checked..next_proxy_request_id
    // next_request_id_to_be_checked is moved to first request ID that is not timed-out

    let state: State = store_get_state(storage).unwrap();
    let mut timeouts_config: TimeoutsConfig = store_get_timeouts_config(storage).unwrap();

    // All existing requests were already checked
    if timeouts_config.next_request_id_to_be_checked == state.next_proxy_request_id {
        return;
    }

    let mut delegator_retrieve_funds_amount: HashMap<Addr, u128> = HashMap::new();
    for i in timeouts_config.next_request_id_to_be_checked..state.next_proxy_request_id {
        match store_get_proxy_reencryption_request(storage, &i) {
            // Skip if request was deleted
            None => {
                continue;
            }
            Some(proxy_request) => {
                let request_state = get_reencryption_request_state(
                    storage,
                    &proxy_request.data_id,
                    &proxy_request.delegatee_pubkey,
                    &block_height,
                );

                // Resolve timed-out request
                if request_state == ReencryptionRequestState::TimedOut {
                    timeout_proxy_reencryption_requests(
                        storage,
                        &i,
                        &mut delegator_retrieve_funds_amount,
                        block_height,
                    )
                    .unwrap();
                }
                // Skip completed requests
                else if request_state != ReencryptionRequestState::Granted {
                    // If request is Active it will become new next_request_id_to_be_checked
                    timeouts_config.next_request_id_to_be_checked = i;
                    break;
                }
            }
        }
        timeouts_config.next_request_id_to_be_checked = i + 1;
    }
    // If this finish without being terminated next_request_id_to_be_checked == next_proxy_request_id

    // Update next_request_id_to_be_checked
    store_set_timeouts_config(storage, &timeouts_config).unwrap();

    // Return stake from unfinished requests to delegators
    let staking_config = store_get_staking_config(storage).unwrap();
    for (delegator_addr, stake_amount) in delegator_retrieve_funds_amount {
        add_bank_msg(
            response,
            &delegator_addr,
            stake_amount,
            &staking_config.stake_denom,
        );
    }
}

pub fn timeout_proxy_reencryption_requests(
    storage: &mut dyn Storage,
    re_request_id: &u64,
    delegator_retrieve_funds_amount: &mut HashMap<Addr, u128>,
    block_height: u64,
) -> StdResult<()> {
    // Delete selected proxy_re_encryption request with timeout and retrieve stake back to the delegator
    // And resolve ParentReencryptionRequest

    // Check if proxy-reencryption request exists
    let re_request: ProxyReencryptionRequest =
        match store_get_proxy_reencryption_request(storage, re_request_id) {
            None => Err(StdError::generic_err(format!(
                "Request {} doesn't exist",
                re_request_id
            ))),
            Some(request) => Ok(request),
        }?;

    // Get parent re-encryption request
    let mut parent_request: ParentReencryptionRequest = store_get_parent_reencryption_request(
        storage,
        &re_request.data_id,
        &re_request.delegatee_pubkey,
    )
    .unwrap();

    // Check if request timed out
    if block_height < parent_request.timeout_height {
        return Err(StdError::generic_err(format!(
            "Request {} isn't timed out",
            re_request_id
        )));
    }

    // Time-out request when state is not granted
    if parent_request.state != ReencryptionRequestState::Granted
        && parent_request.state != ReencryptionRequestState::TimedOut
    {
        // Resolve parent re-encryption request
        // Refund delegator
        update_retrieved_delegator_stake_map(
            delegator_retrieve_funds_amount,
            &parent_request.delegator_addr,
            parent_request.delegator_provided_reward_amount.u128(),
        );
        parent_request.funds_pool = parent_request
            .funds_pool
            .checked_sub(parent_request.delegator_provided_reward_amount)?;
        parent_request.delegator_provided_reward_amount = Uint128::new(0);
        parent_request.state = ReencryptionRequestState::TimedOut;
        store_set_parent_reencryption_request(
            storage,
            &re_request.data_id,
            &re_request.delegatee_pubkey,
            &parent_request,
        );
    }

    // Delete proxy re-encryption request
    store_remove_delegatee_proxy_reencryption_request(
        storage,
        &re_request.data_id,
        &re_request.delegatee_pubkey,
        &re_request.proxy_pubkey,
    );
    store_remove_proxy_reencryption_request_from_queue(
        storage,
        &re_request.proxy_pubkey,
        re_request_id,
    );
    store_remove_proxy_reencryption_request(storage, re_request_id);

    Ok(())
}

pub fn get_all_fragments(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> Vec<String> {
    let mut fragments: Vec<String> = Vec::new();
    for request_id in
        store_get_all_delegatee_proxy_reencryption_requests(storage, data_id, delegatee_pubkey)
    {
        let request = store_get_proxy_reencryption_request(storage, &request_id).unwrap();

        match request.fragment {
            None => continue,
            Some(fragment) => fragments.push(fragment.clone()),
        }
    }
    fragments
}

pub fn update_retrieved_delegator_stake_map(
    delegator_retrieve_funds_amount: &mut HashMap<Addr, u128>,
    delegator_addr: &Addr,
    additional_stake_amount: u128,
) {
    match delegator_retrieve_funds_amount.get(delegator_addr).cloned() {
        None => {
            delegator_retrieve_funds_amount.insert(delegator_addr.clone(), additional_stake_amount);
        }
        Some(stake_amount) => {
            delegator_retrieve_funds_amount.insert(
                delegator_addr.clone(),
                stake_amount + additional_stake_amount,
            );
        }
    }
}
