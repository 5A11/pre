use cosmwasm_std::{from_slice, to_vec, Addr, Order, StdResult, Storage};
use cosmwasm_storage::{
    singleton, singleton_read, PrefixedStorage, ReadonlyPrefixedStorage, Singleton,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Singletons
static STATE_KEY: &[u8] = b"State";

// Maps

// Proxy register whitelist
// Map proxy: Addr -> is_registered: bool
static IS_PROXY_KEY: &[u8] = b"IsProxy";

// Map proxy_pubkey: String -> proxy: Addr
static ACTIVE_PROXIES_ADDRESSES_KEY: &[u8] = b"ProxyAddresses";

// Map proxy: Addr -> proxy_pubkey: String
static ACTIVE_PROXIES_PUBKEYS_KEY: &[u8] = b"ProxyPubkeys";

// Map data_id: String -> data_entry: DataEntry
static DATA_ENTRIES_KEY: &[u8] = b"DataEntries";

// Singleton structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct State {
    pub admin: Addr,
    // n_selected proxies will be between threshold and n_max_proxies
    pub threshold: u32,
    pub n_max_proxies: u32,

    // Total number of re-encryption requests
    pub next_request_id: u64,
    pub next_delegation_id: u64,
}

// Store structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct DataEntry {
    pub delegator_pubkey: String,
    pub delegator_addr: Addr,
}

// Getters and setters

// STATE
pub fn get_state(storage: &dyn Storage) -> StdResult<State> {
    singleton_read(storage, STATE_KEY).load()
}

pub fn set_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    let mut singl: Singleton<State> = singleton(storage, STATE_KEY);
    singl.save(state)
}

// IS_PROXY
pub fn set_is_proxy(storage: &mut dyn Storage, proxy_addr: &Addr, is_proxy: bool) {
    let mut store = PrefixedStorage::new(storage, IS_PROXY_KEY);

    // Any value in store means true - &[1]
    match is_proxy {
        true => store.set(proxy_addr.as_bytes(), &[1]),
        false => store.remove(proxy_addr.as_bytes()),
    }
}

pub fn get_is_proxy(storage: &dyn Storage, proxy_addr: &Addr) -> bool {
    let store = ReadonlyPrefixedStorage::new(storage, IS_PROXY_KEY);

    store.get(proxy_addr.as_bytes()).is_some()
}

pub fn get_all_proxies(storage: &dyn Storage) -> Vec<Addr> {
    let store = ReadonlyPrefixedStorage::new(storage, IS_PROXY_KEY);

    let mut deserialized_keys: Vec<Addr> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to &proxy_addr.as_bytes()
        deserialized_keys.push(Addr::unchecked(std::str::from_utf8(&pair.0).unwrap()));
    }

    deserialized_keys
}

// PROXIES_AVAILABITY
pub fn set_proxy_address(storage: &mut dyn Storage, pub_key: &str, proxy_addr: &Addr) {
    let mut storage = PrefixedStorage::new(storage, ACTIVE_PROXIES_ADDRESSES_KEY);

    storage.set(pub_key.as_bytes(), proxy_addr.as_bytes());
}

pub fn remove_proxy_address(storage: &mut dyn Storage, pub_key: &str) {
    let mut storage = PrefixedStorage::new(storage, ACTIVE_PROXIES_ADDRESSES_KEY);

    storage.remove(pub_key.as_bytes());
}

pub fn get_proxy_address(storage: &dyn Storage, pub_key: &str) -> Option<Addr> {
    let store = ReadonlyPrefixedStorage::new(storage, ACTIVE_PROXIES_ADDRESSES_KEY);

    let res = store.get(pub_key.as_bytes());
    res.map(|res| Addr::unchecked(std::str::from_utf8(&res).unwrap()))
}

// PROXIES_PUBKEYS
pub fn set_proxy_pubkey(storage: &mut dyn Storage, proxy_addr: &Addr, pub_key: &str) {
    let mut storage = PrefixedStorage::new(storage, ACTIVE_PROXIES_PUBKEYS_KEY);

    storage.set(proxy_addr.as_bytes(), pub_key.as_bytes());
}

pub fn remove_proxy_pubkey(storage: &mut dyn Storage, proxy_addr: &Addr) {
    let mut storage = PrefixedStorage::new(storage, ACTIVE_PROXIES_PUBKEYS_KEY);

    storage.remove(proxy_addr.as_bytes());
}

pub fn get_proxy_pubkey(storage: &dyn Storage, proxy_addr: &Addr) -> Option<String> {
    let store = ReadonlyPrefixedStorage::new(storage, ACTIVE_PROXIES_PUBKEYS_KEY);

    let res = store.get(proxy_addr.as_bytes());
    res.map(|res| String::from_utf8(res).unwrap())
}

pub fn get_all_available_proxy_pubkeys(storage: &dyn Storage) -> Vec<String> {
    let store = ReadonlyPrefixedStorage::new(storage, ACTIVE_PROXIES_PUBKEYS_KEY);

    let mut deserialized_keys: Vec<String> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to &string.as_bytes()
        deserialized_keys.push(std::str::from_utf8(&pair.1).unwrap().to_string());
    }

    deserialized_keys
}

// DATA_ENTRIES
pub fn set_data_entry(storage: &mut dyn Storage, data_id: &str, data_entry: &DataEntry) {
    let mut store = PrefixedStorage::new(storage, DATA_ENTRIES_KEY);
    store.set(data_id.as_bytes(), &to_vec(data_entry).unwrap());
}

pub fn remove_data_entry(storage: &mut dyn Storage, data_id: &str) {
    let mut store = PrefixedStorage::new(storage, DATA_ENTRIES_KEY);

    store.remove(data_id.as_bytes());
}

pub fn get_data_entry(storage: &dyn Storage, data_id: &str) -> Option<DataEntry> {
    let store = ReadonlyPrefixedStorage::new(storage, DATA_ENTRIES_KEY);

    store
        .get(data_id.as_bytes())
        .map(|data| from_slice(&data).unwrap())
}
