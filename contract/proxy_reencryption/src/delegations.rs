use cosmwasm_std::{from_slice, to_vec, Order, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

// To get all proxies from 1 delegation
// Map delegator_pubkey: String -> delegatee_pubkey: String -> proxy_pubkey: String -> delegation_id: u64
static DELEGATIONS_ID_STORE_KEY: &[u8] = b"DelegationIDStore";

// To get all delegations for proxy
// proxy_pubkey: String -> delegation_id: u64 -> is_delegation: bool
static PROXY_DELEGATIONS_STORE_KEY: &[u8] = b"ProxyDelegationsStore";

// Map delegation_id: u64 -> delegation: Delegation
static DELEGATIONS_STORE_KEY: &[u8] = b"DelegationsStore";

// Map  Map delegator_pubkey: String -> delegatee_pubkey: String -> is_used: bool
static IS_DELEGATION_USED: &[u8] = b"IsDelegationUsed";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Delegation {
    pub delegator_pubkey: String,
    pub delegatee_pubkey: String,
    pub delegation_string: Option<String>,
}

// DELEGATIONS_ID_STORE_KEY
pub fn set_delegation_id(
    storage: &mut dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
    delegation_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATIONS_ID_STORE_KEY,
            delegator_pubkey.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.set(proxy_pubkey.as_bytes(), &delegation_id.to_le_bytes());
}

pub fn remove_delegation_id(
    storage: &mut dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATIONS_ID_STORE_KEY,
            delegator_pubkey.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.remove(proxy_pubkey.as_bytes());
}

pub fn get_delegation_id(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) -> Option<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATIONS_ID_STORE_KEY,
            delegator_pubkey.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store
        .get(proxy_pubkey.as_bytes())
        .map(|data| u64::from_le_bytes(data.try_into().unwrap()))
}

pub fn get_all_proxies_from_delegation(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
) -> Vec<String> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATIONS_ID_STORE_KEY,
            delegator_pubkey.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    let mut deserialized_keys: Vec<String> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to string.as_bytes()
        deserialized_keys.push(std::str::from_utf8(&pair.0).unwrap().to_string());
    }

    deserialized_keys
}

// DELEGATIONS_STORE_KEY
pub fn set_delegation(storage: &mut dyn Storage, delegation_id: &u64, delegation: &Delegation) {
    let mut store = PrefixedStorage::new(storage, DELEGATIONS_STORE_KEY);

    store.set(&delegation_id.to_le_bytes(), &to_vec(delegation).unwrap());
}

pub fn get_delegation(storage: &dyn Storage, delegation_id: &u64) -> Option<Delegation> {
    let store = ReadonlyPrefixedStorage::new(storage, DELEGATIONS_STORE_KEY);

    store
        .get(&delegation_id.to_le_bytes())
        .map(|data| from_slice(&data).unwrap())
}

pub fn remove_delegation(storage: &mut dyn Storage, delegation_id: &u64) {
    let mut store = PrefixedStorage::new(storage, DELEGATIONS_STORE_KEY);

    store.remove(&delegation_id.to_le_bytes());
}

// PROXY_DELEGATIONS_STORE
pub fn add_proxy_delegation(storage: &mut dyn Storage, proxy_pubkey: &str, delegation_id: &u64) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PROXY_DELEGATIONS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    // Any value in store means true - &[1]
    store.set(&delegation_id.to_le_bytes(), &[1]);
}

pub fn remove_proxy_delegation(storage: &mut dyn Storage, proxy_pubkey: &str, delegation_id: &u64) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PROXY_DELEGATIONS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store.remove(&delegation_id.to_le_bytes());
}

pub fn is_proxy_delegation(storage: &dyn Storage, proxy_pubkey: &str, delegation_id: &u64) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PROXY_DELEGATIONS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store.get(&delegation_id.to_le_bytes()).is_some()
}

pub fn get_all_proxy_delegations(storage: &dyn Storage, proxy_pubkey: &str) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PROXY_DELEGATIONS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    let mut deserialized_keys: Vec<u64> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to to_vec
        deserialized_keys.push(u64::from_le_bytes(pair.0.try_into().unwrap()));
    }

    deserialized_keys
}

// IS_DELEGATION_USED
pub fn set_is_delegation_used(
    storage: &mut dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    is_delegation_used: bool,
) {
    let mut store =
        PrefixedStorage::multilevel(storage, &[IS_DELEGATION_USED, delegator_pubkey.as_bytes()]);

    // Any value in store means true - &[1]
    match is_delegation_used {
        true => store.set(delegatee_pubkey.as_bytes(), &[1]),
        false => store.remove(delegatee_pubkey.as_bytes()),
    }
}

pub fn get_is_delegation_used(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[IS_DELEGATION_USED, delegator_pubkey.as_bytes()],
    );

    store.get(delegatee_pubkey.as_bytes()).is_some()
}
