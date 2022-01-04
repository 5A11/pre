use crate::common::add_bank_msg;
use crate::proxies::{get_proxy_address, get_proxy_entry, set_proxy_entry};
use crate::state::{get_staking_config, get_state};
use cosmwasm_std::{from_slice, to_vec, Addr, Order, Response, StdResult, Storage, Uint128};
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

    // Incentives params
    pub reward_amount: Uint128,
    pub proxy_withdrawn_stake_amount: Uint128,
}

// Parent re-encryption request
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ParentReencryptionRequest {
    pub n_provided_fragments: u32,
    pub n_proxy_requests: u32,
    pub state: ReencryptionRequestState,

    // Incentives params
    pub slashed_stake_amount: Uint128,
    // Reward will be returned to this address when request is deleted
    pub delegator_addr: Addr,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReencryptionRequestState {
    Inaccessible,
    Ready,
    Granted,
}

// PROXY_REENCRYPTION_REQUESTS_STORE_KEY
pub fn set_proxy_reencryption_request(
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

pub fn get_proxy_reencryption_request(
    storage: &dyn Storage,
    proxy_reencryption_request_id: &u64,
) -> Option<ProxyReencryptionRequest> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXY_REENCRYPTION_REQUESTS_STORE_KEY);

    store
        .get(&proxy_reencryption_request_id.to_le_bytes())
        .map(|data| from_slice(&data).unwrap())
}

pub fn remove_proxy_reencryption_request(
    storage: &mut dyn Storage,
    proxy_reencryption_request_id: &u64,
) {
    let mut store = PrefixedStorage::new(storage, PROXY_REENCRYPTION_REQUESTS_STORE_KEY);

    store.remove(&proxy_reencryption_request_id.to_le_bytes());
}

// DELEGATEE_REQUESTS_STORE
pub fn add_delegatee_proxy_reencryption_request(
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

pub fn get_delegatee_proxy_reencryption_request(
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

pub fn get_all_delegatee_proxy_reencryption_requests(
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

pub fn remove_delegatee_proxy_reencryption_request(
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
pub fn add_proxy_reencryption_request_to_queue(
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

pub fn remove_proxy_reencryption_request_from_queue(
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

pub fn is_proxy_reencryption_request_in_queue(
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

pub fn get_all_proxy_reencryption_requests_in_queue(
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
pub fn set_parent_reencryption_request(
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

pub fn remove_parent_reencryption_request(
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

pub fn get_parent_reencryption_request(
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
) -> ReencryptionRequestState {
    let parent_request = match get_parent_reencryption_request(storage, data_id, delegatee_pubkey) {
        None => return ReencryptionRequestState::Inaccessible,
        Some(parent_request) => parent_request,
    };

    parent_request.state
}

// Delete all unfinished current proxy re-encryption requests
pub fn remove_proxy_reencryption_requests(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    response: &mut Response,
) -> StdResult<()> {
    let staking_config = get_staking_config(storage)?;
    let state = get_state(storage)?;

    let mut delegator_retrieve_funds_amount: HashMap<Addr, u128> = HashMap::new();

    for re_request_id in get_all_proxy_reencryption_requests_in_queue(storage, proxy_pubkey) {
        let mut re_request = get_proxy_reencryption_request(storage, &re_request_id).unwrap();

        let mut parent_request = get_parent_reencryption_request(
            storage,
            &re_request.data_id,
            &re_request.delegatee_pubkey,
        )
        .unwrap();

        // Slash current proxy -> Move withdrawn stake to parent_request pool
        let acquired_stake = re_request.proxy_withdrawn_stake_amount.u128();
        re_request.proxy_withdrawn_stake_amount = Uint128::new(0);
        set_proxy_reencryption_request(storage, &re_request_id, &re_request);

        // Add acquired stake to stake reserved for repaying delegator when request fails to be completed
        parent_request.slashed_stake_amount =
            Uint128::new(parent_request.slashed_stake_amount.u128() + acquired_stake);

        if parent_request.n_proxy_requests < state.threshold + 1 {
            // Delete other proxies related requests because request cannot be completed without this proxy

            // Get all neighbour re-encryption request
            for i_re_request_id in get_all_delegatee_proxy_reencryption_requests(
                storage,
                &re_request.data_id,
                &re_request.delegatee_pubkey,
            ) {
                resolve_proxy_reencryption_requests(
                    storage,
                    &i_re_request_id,
                    &mut delegator_retrieve_funds_amount,
                );
            }

            // Return stake retrieved from slashed proxies to delegator stake before deleting parent request
            update_retrieved_delegator_stake_map(
                &mut delegator_retrieve_funds_amount,
                &parent_request.delegator_addr,
                parent_request.slashed_stake_amount.u128(),
            );

            // Delete parent request
            remove_parent_reencryption_request(
                storage,
                &re_request.data_id,
                &re_request.delegatee_pubkey,
            );
        } else {
            // Delete only current proxy unfinished request

            // Update parent request
            parent_request.n_proxy_requests -= 1;
            set_parent_reencryption_request(
                storage,
                &re_request.data_id,
                &re_request.delegatee_pubkey,
                &parent_request,
            );

            resolve_proxy_reencryption_requests(
                storage,
                &re_request_id,
                &mut delegator_retrieve_funds_amount,
            );
        }
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

// Delete re-encryption request and remember remaining stake amount to be later returned to delegator
pub fn resolve_proxy_reencryption_requests(
    storage: &mut dyn Storage,
    re_request_id: &u64,
    delegator_retrieve_funds_amount: &mut HashMap<Addr, u128>,
) {
    let re_request = get_proxy_reencryption_request(storage, re_request_id).unwrap();
    let parent_request =
        get_parent_reencryption_request(storage, &re_request.data_id, &re_request.delegatee_pubkey)
            .unwrap();

    // Update delegator stake before deleting entry
    update_retrieved_delegator_stake_map(
        delegator_retrieve_funds_amount,
        &parent_request.delegator_addr,
        re_request.reward_amount.u128(),
    );

    // Return remaining stake to proxies if weren't slashed
    if re_request.proxy_withdrawn_stake_amount.u128() > 0 {
        let proxy_addr = get_proxy_address(storage, &re_request.proxy_pubkey).unwrap();
        let mut proxy_entry = get_proxy_entry(storage, &proxy_addr).unwrap();
        proxy_entry.stake_amount = Uint128(
            proxy_entry.stake_amount.u128() + re_request.proxy_withdrawn_stake_amount.u128(),
        );
        set_proxy_entry(storage, &proxy_addr, &proxy_entry);
    }

    remove_delegatee_proxy_reencryption_request(
        storage,
        &re_request.data_id,
        &re_request.delegatee_pubkey,
        &re_request.proxy_pubkey,
    );
    remove_proxy_reencryption_request_from_queue(storage, &re_request.proxy_pubkey, re_request_id);
    remove_proxy_reencryption_request(storage, re_request_id);
}

pub fn get_all_fragments(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> Vec<String> {
    let mut fragments: Vec<String> = Vec::new();
    for request_id in
        get_all_delegatee_proxy_reencryption_requests(storage, data_id, delegatee_pubkey)
    {
        let request = get_proxy_reencryption_request(storage, &request_id).unwrap();

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
