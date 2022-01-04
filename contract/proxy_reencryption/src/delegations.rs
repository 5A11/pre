use crate::proxies::{get_proxy_address, get_proxy_entry};
use cosmwasm_std::{from_slice, to_vec, Order, StdResult, Storage};
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Delegation {
    pub delegator_pubkey: String,
    pub delegatee_pubkey: String,
    pub delegation_string: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DelegationState {
    NonExisting,
    WaitingForDelegationStrings,
    Active,
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

pub fn is_delegation_empty(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATIONS_ID_STORE_KEY,
            delegator_pubkey.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    let is_empty: bool = store
        .range(None, None, Order::Ascending)
        .peekable()
        .peek()
        .is_none();
    is_empty
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

// High level methods

pub fn get_delegation_state(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
) -> DelegationState {
    let mut delegation_state: DelegationState = DelegationState::NonExisting;

    if !is_delegation_empty(storage, delegator_pubkey, delegatee_pubkey) {
        let proxy_pubkey =
            &get_all_proxies_from_delegation(storage, delegator_pubkey, delegatee_pubkey)[0];
        let delegation_id =
            get_delegation_id(storage, delegator_pubkey, delegatee_pubkey, proxy_pubkey).unwrap();
        let delegation = get_delegation(storage, &delegation_id).unwrap();

        if delegation.delegation_string.is_none() {
            // Delegation string provided
            delegation_state = DelegationState::WaitingForDelegationStrings;
        } else {
            // Delegation string not provided
            delegation_state = DelegationState::Active;
        }
    }
    delegation_state
}

pub fn remove_proxy_delegations(storage: &mut dyn Storage, proxy_pubkey: &str) -> StdResult<()> {
    // Delete all proxy delegations -- Make proxy inactive / stop requests factory
    for delegation_id in get_all_proxy_delegations(storage, proxy_pubkey) {
        let delegation = get_delegation(storage, &delegation_id).unwrap();

        let all_delegation_proxies = get_all_proxies_from_delegation(
            storage,
            &delegation.delegator_pubkey,
            &delegation.delegatee_pubkey,
        );
        // Delete entire delegation = delete each proxy delegation in delegation
        for i_proxy_pubkey in all_delegation_proxies {
            let i_delegation_id = get_delegation_id(
                storage,
                &delegation.delegator_pubkey,
                &delegation.delegatee_pubkey,
                &i_proxy_pubkey,
            )
            .unwrap();

            remove_delegation(storage, &i_delegation_id);
            remove_delegation_id(
                storage,
                &delegation.delegator_pubkey,
                &delegation.delegatee_pubkey,
                &i_proxy_pubkey,
            );
            remove_proxy_delegation(storage, &i_proxy_pubkey, &i_delegation_id);
        }
    }
    Ok(())
}

pub fn get_n_available_proxies_from_delegation(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_withdrawn_stake_amount: &u128,
) -> u32 {
    // Get number of proxies from delegation with enough stake to get re-encryption request
    let delegation_proxies =
        get_all_proxies_from_delegation(storage, delegator_pubkey, delegatee_pubkey);

    let mut n_active_proxies: u32 = 0;
    for proxy_pubkey in delegation_proxies {
        // Check if proxy has enough stake
        let proxy_addr = get_proxy_address(storage, &proxy_pubkey).unwrap();
        let proxy = get_proxy_entry(storage, &proxy_addr).unwrap();

        if &proxy.stake_amount.u128() >= proxy_withdrawn_stake_amount {
            // Proxy cannot be selected for insufficient amount
            n_active_proxies += 1;
        }
    }
    n_active_proxies
}
