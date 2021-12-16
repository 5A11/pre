use crate::common::add_bank_msg;
use crate::state::get_state;
use cosmwasm_std::{from_slice, to_vec, Addr, Order, Response, StdResult, Storage, Uint128};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

// Map reencryption_request_id: u64 -> request: ReencryptionRequest
static REENCRYPTION_REQUESTS_STORE_KEY: &[u8] = b"ReencryptionRequests";

// Delegatee side to lookup fragments
// Map data_id: String -> delegatee_pubkey: String -> proxy_pubkey: String -> reencryption_request_id: u64
static DELEGATEE_REQUESTS_STORE_KEY: &[u8] = b"DelegateeRequests";

// Proxy side to lookup active tasks
// Map proxy_pubkey: String -> reencryption_request_id: u64 -> is_request: bool
static PROXY_REQUESTS_STORE_KEY: &[u8] = b"ProxyRequests";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ReencryptionRequest {
    pub proxy_pubkey: String,
    pub data_id: String,
    pub delegatee_pubkey: String,
    pub fragment: Option<String>,
    pub delegation_string: String,

    pub reward_amount: Uint128,
    // Stake will be returned to this address
    pub delegator_addr: Addr,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReencryptionRequestState {
    Inaccessible,
    Ready,
    Granted,
}

// REENCRYPTION_REQUESTS_STORE_KEY
pub fn set_reencryption_request(
    storage: &mut dyn Storage,
    reencryption_request_id: &u64,
    reencryption_request: &ReencryptionRequest,
) {
    let mut store = PrefixedStorage::new(storage, REENCRYPTION_REQUESTS_STORE_KEY);

    store.set(
        &reencryption_request_id.to_le_bytes(),
        &to_vec(reencryption_request).unwrap(),
    );
}

pub fn get_reencryption_request(
    storage: &dyn Storage,
    reencryption_request_id: &u64,
) -> Option<ReencryptionRequest> {
    let store = ReadonlyPrefixedStorage::new(storage, REENCRYPTION_REQUESTS_STORE_KEY);

    store
        .get(&reencryption_request_id.to_le_bytes())
        .map(|data| from_slice(&data).unwrap())
}

pub fn remove_reencryption_request(storage: &mut dyn Storage, reencryption_request_id: &u64) {
    let mut store = PrefixedStorage::new(storage, REENCRYPTION_REQUESTS_STORE_KEY);

    store.remove(&reencryption_request_id.to_le_bytes());
}

// DELEGATEE_REQUESTS_STORE
pub fn add_delegatee_reencryption_request(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
    reencryption_request_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_REQUESTS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.set(
        proxy_pubkey.as_bytes(),
        &reencryption_request_id.to_le_bytes(),
    );
}

pub fn get_delegatee_reencryption_request(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) -> Option<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_REQUESTS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store
        .get(proxy_pubkey.as_bytes())
        .map(|data| u64::from_le_bytes(data.try_into().unwrap()))
}

pub fn get_all_delegatee_reencryption_requests(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_REQUESTS_STORE_KEY,
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

pub fn remove_delegatee_reencryption_request(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_REQUESTS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.remove(proxy_pubkey.as_bytes());
}

// PROXY_REQUESTS_STORE_KEY
pub fn add_proxy_reencryption_request(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    reencryption_request_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PROXY_REQUESTS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    // Any value in store means true - &[1]
    store.set(&reencryption_request_id.to_le_bytes(), &[1]);
}

pub fn remove_proxy_reencryption_request(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    reencryption_request_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PROXY_REQUESTS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store.remove(&reencryption_request_id.to_le_bytes());
}

pub fn is_proxy_reencryption_request(
    storage: &dyn Storage,
    proxy_pubkey: &str,
    reencryption_request_id: &u64,
) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PROXY_REQUESTS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store.get(&reencryption_request_id.to_le_bytes()).is_some()
}

pub fn get_all_proxy_reencryption_requests(storage: &dyn Storage, proxy_pubkey: &str) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PROXY_REQUESTS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    let mut deserialized_keys: Vec<u64> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to to_vec
        deserialized_keys.push(u64::from_le_bytes(pair.0.try_into().unwrap()));
    }

    deserialized_keys
}

pub fn get_reencryption_request_state(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> ReencryptionRequestState {
    let state = get_state(storage).unwrap();

    // Check if threshold amount of fragments is provided:
    let delegatee_request_ids =
        get_all_delegatee_reencryption_requests(storage, data_id, delegatee_pubkey);

    if delegatee_request_ids.is_empty() {
        return ReencryptionRequestState::Inaccessible;
    }

    let mut n_completed_requests = 0;
    for i_request_id in &delegatee_request_ids {
        let i_request = get_reencryption_request(storage, i_request_id).unwrap();
        if i_request.fragment.is_some() {
            n_completed_requests += 1;
        }
    }

    if n_completed_requests >= state.threshold {
        ReencryptionRequestState::Granted
    } else {
        ReencryptionRequestState::Ready
    }
}

// Delete all unfinished current proxy re-encryption requests
pub fn remove_proxy_reencryption_requests(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    response: &mut Response,
) -> StdResult<()> {
    let state = get_state(storage)?;

    let mut delegator_retrieve_stake: HashMap<Addr, u128> = HashMap::new();

    for re_request_id in get_all_proxy_reencryption_requests(storage, proxy_pubkey) {
        let re_request = get_reencryption_request(storage, &re_request_id).unwrap();

        let all_related_requests_ids = get_all_delegatee_reencryption_requests(
            storage,
            &re_request.data_id,
            &re_request.delegatee_pubkey,
        );

        if all_related_requests_ids.len() < (state.threshold as usize + 1) {
            // Delete other proxies related requests because request cannot be completed without this proxy

            for i_re_request_id in all_related_requests_ids {
                resolve_proxy_reencryption_requests(
                    storage,
                    &i_re_request_id,
                    &mut delegator_retrieve_stake,
                );
            }
        } else {
            // Delete only current proxy unfinished request
            resolve_proxy_reencryption_requests(
                storage,
                &re_request_id,
                &mut delegator_retrieve_stake,
            );
        }
    }

    // Return stake from unfinished requests to delegators
    for (delegator_addr, stake_amount) in delegator_retrieve_stake {
        add_bank_msg(response, &delegator_addr, stake_amount, &state.stake_denom);
    }

    Ok(())
}

// High level methods

// Delete re-encryption request and remember remaining stake amount to be later returned to delegator
pub fn resolve_proxy_reencryption_requests(
    storage: &mut dyn Storage,
    re_request_id: &u64,
    delegator_retrieve_stake: &mut HashMap<Addr, u128>,
) {
    let re_request = get_reencryption_request(storage, re_request_id).unwrap();

    // Update delegator stake before deleting entry
    match delegator_retrieve_stake
        .get(&re_request.delegator_addr)
        .cloned()
    {
        None => {
            delegator_retrieve_stake.insert(
                re_request.delegator_addr.clone(),
                re_request.reward_amount.u128(),
            );
        }
        Some(stake_amount) => {
            delegator_retrieve_stake.insert(
                re_request.delegator_addr.clone(),
                stake_amount + re_request.reward_amount.u128(),
            );
        }
    }

    remove_delegatee_reencryption_request(
        storage,
        &re_request.data_id,
        &re_request.delegatee_pubkey,
        &re_request.proxy_pubkey,
    );
    remove_proxy_reencryption_request(storage, &re_request.proxy_pubkey, re_request_id);
    remove_reencryption_request(storage, re_request_id);
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
