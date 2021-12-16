use crate::state::State;
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
// Map proxy: Addr -> proxy: Proxy
static PROXIES_KEY: &[u8] = b"Proxies";

// Map proxy_pubkey: String -> proxy: Addr
static PROXY_ADDRESS_KEY: &[u8] = b"ProxyAddress";

// Map proxy_pubkey: String -> is_active: bool
static IS_PROXY_ACTIVE: &[u8] = b"IsProxyActive";

// Getters and setters

// PROXIES_KEY
pub fn set_proxy(storage: &mut dyn Storage, proxy_addr: &Addr, proxy: &Proxy) {
    let mut store = PrefixedStorage::new(storage, PROXIES_KEY);

    store.set(proxy_addr.as_bytes(), &to_vec(proxy).unwrap());
}

pub fn remove_proxy(storage: &mut dyn Storage, proxy_addr: &Addr) {
    let mut store = PrefixedStorage::new(storage, PROXIES_KEY);

    store.remove(proxy_addr.as_bytes());
}

pub fn get_proxy(storage: &dyn Storage, proxy_addr: &Addr) -> Option<Proxy> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXIES_KEY);

    store
        .get(proxy_addr.as_bytes())
        .map(|data| from_slice(&data).unwrap())
}

// ACTIVE_PROXIES_ADDRESSES_KEY

// PROXY_ADDRESS
pub fn set_proxy_address(storage: &mut dyn Storage, proxy_pubkey: &str, proxy_addr: &Addr) {
    let mut storage = PrefixedStorage::new(storage, PROXY_ADDRESS_KEY);

    storage.set(proxy_pubkey.as_bytes(), proxy_addr.as_bytes());
}

pub fn remove_proxy_address(storage: &mut dyn Storage, proxy_pubkey: &str) {
    let mut storage = PrefixedStorage::new(storage, PROXY_ADDRESS_KEY);

    storage.remove(proxy_pubkey.as_bytes());
}

pub fn get_proxy_address(storage: &dyn Storage, proxy_pubkey: &str) -> Option<Addr> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXY_ADDRESS_KEY);

    let res = store.get(proxy_pubkey.as_bytes());
    res.map(|res| Addr::unchecked(String::from_utf8(res).unwrap()))
}

// IS_PROXY_ACTIVE
pub fn set_is_proxy_active(storage: &mut dyn Storage, proxy_pubkey: &str, is_proxy_active: bool) {
    let mut store = PrefixedStorage::new(storage, IS_PROXY_ACTIVE);

    // Any value in store means true - &[1]
    match is_proxy_active {
        true => store.set(proxy_pubkey.as_bytes(), &[1]),
        false => store.remove(proxy_pubkey.as_bytes()),
    }
}

pub fn get_is_proxy_active(storage: &dyn Storage, proxy_pubkey: &str) -> bool {
    let store = ReadonlyPrefixedStorage::new(storage, IS_PROXY_ACTIVE);

    store.get(proxy_pubkey.as_bytes()).is_some()
}

pub fn get_all_active_proxy_pubkeys(storage: &dyn Storage) -> Vec<String> {
    let store = ReadonlyPrefixedStorage::new(storage, IS_PROXY_ACTIVE);

    let mut deserialized_keys: Vec<String> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to &string.as_bytes()
        deserialized_keys.push(std::str::from_utf8(&pair.0).unwrap().to_string());
    }

    deserialized_keys
}

// Other
pub fn maximum_withdrawable_stake_amount(state: &State, proxy: &Proxy) -> u128 {
    if proxy.stake_amount.u128() > state.minimum_proxy_stake_amount.u128() {
        proxy.stake_amount.u128() - state.minimum_proxy_stake_amount.u128()
    } else {
        0
    }
}
