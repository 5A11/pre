use cosmwasm_std::{from_slice, to_vec, Order, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
