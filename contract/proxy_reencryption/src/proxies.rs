use crate::state::StakingConfig;
use cosmwasm_std::{from_slice, to_vec, Addr, Order, Storage, Uint128};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProxyState {
    Authorised,
    Registered,
    Leaving,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Proxy {
    pub state: ProxyState,
    pub proxy_pubkey: Option<String>,
    pub stake_amount: Uint128,
}

// Proxy register whitelist
// Map proxy_addr: Addr -> proxy: Proxy
static PROXIES_KEY: &[u8] = b"Proxies";

// Active proxies
// Map proxy_addr: String -> is_active: bool
static IS_PROXY_ACTIVE: &[u8] = b"IsProxyActive";

// Getters and setters

// PROXIES_KEY
pub fn store_set_proxy_entry(storage: &mut dyn Storage, proxy_addr: &Addr, proxy: &Proxy) {
    let mut store = PrefixedStorage::new(storage, PROXIES_KEY);

    store.set(proxy_addr.as_bytes(), &to_vec(proxy).unwrap());
}

pub fn store_remove_proxy_entry(storage: &mut dyn Storage, proxy_addr: &Addr) {
    let mut store = PrefixedStorage::new(storage, PROXIES_KEY);

    store.remove(proxy_addr.as_bytes());
}

pub fn store_get_proxy_entry(storage: &dyn Storage, proxy_addr: &Addr) -> Option<Proxy> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXIES_KEY);

    store
        .get(proxy_addr.as_bytes())
        .map(|data| from_slice(&data).unwrap())
}


pub fn store_get_all_proxies(storage: &dyn Storage) -> Vec<Addr> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXIES_KEY);

    let mut deserialized_keys: Vec<Addr> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys
        deserialized_keys.push(Addr::unchecked(String::from_utf8(pair.0).unwrap()));
    }

    deserialized_keys
}


// IS_PROXY_ACTIVE
pub fn store_set_is_proxy_active(
    storage: &mut dyn Storage,
    proxy_addr: &Addr,
    is_proxy_active: bool,
) {
    let mut store = PrefixedStorage::new(storage, IS_PROXY_ACTIVE);

    // Any value in store means true - &[1]
    match is_proxy_active {
        true => store.set(proxy_addr.as_bytes(), &[1]),
        false => store.remove(proxy_addr.as_bytes()),
    }
}

pub fn store_get_is_proxy_active(storage: &dyn Storage, proxy_addr: &Addr) -> bool {
    let store = ReadonlyPrefixedStorage::new(storage, IS_PROXY_ACTIVE);

    store.get(proxy_addr.as_bytes()).is_some()
}

pub fn store_get_all_active_proxy_addresses(storage: &dyn Storage) -> Vec<Addr> {
    let store = ReadonlyPrefixedStorage::new(storage, IS_PROXY_ACTIVE);

    let mut deserialized_keys: Vec<Addr> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to &string.as_bytes()
        deserialized_keys.push(Addr::from_bytes(&pair.0).unwrap());
    }

    deserialized_keys
}

// High level methods
pub fn get_maximum_withdrawable_stake_amount(
    staking_config: &StakingConfig,
    proxy: &Proxy,
) -> u128 {
    if proxy.stake_amount.u128() > staking_config.minimum_proxy_stake_amount.u128() {
        proxy.stake_amount.u128() - staking_config.minimum_proxy_stake_amount.u128()
    } else {
        0
    }
}
