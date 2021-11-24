use cosmwasm_std::{Addr, Storage, StdResult, to_vec, Order, from_slice};
use cosmwasm_storage::{singleton, singleton_read, Singleton, PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

pub type HashID = String;

// Singletons
static STATE_KEY: &[u8] = b"State";

// Maps

// Proxy register whitelist
// Map proxy: Addr -> is_registered: bool
static IS_PROXY_KEY: &[u8] = b"IsProxy";

// Map proxy_pubkey: String -> proxy: Addr
static PROXIES_AVAILABITY_KEY: &[u8] = b"ProxyAvailable";

// Map proxy: Addr -> proxy_pubkey: String
static PROXIES_PUBKEYS_KEY: &[u8] = b"ProxyPubkeys";

// Map data_id: String -> data_entry: DataEntry
static DATA_ENTRIES_KEY: &[u8] = b"DataEntries";

// Map delegator_addr: Addr -> delegator_pubkey: String -> delegatee_pubkey: String -> proxy_pubkey: String -> delegation_string: Option<String>
static DELEGATIONS_STORE_KEY: &[u8] = b"DelegationStore";

// Map reencryption_request_id: u64 -> request: ReencryptionRequest
static REENCRYPTION_REQUESTS_STORE_KEY: &[u8] = b"ReencryptionRequests";

// Delegatee side to lookup fragments
// Map data_id: HashID -> delegatee_pubkey: String -> proxy_pubkey: String -> reencryption_request_id: u64
static DELEGATEE_REQUESTS_STORE_KEY: &[u8] = b"DelegateeRequests";

// Proxy side to lookup active tasks
// Map proxy_pubkey: String -> reencryption_request_id: u64 -> is_request: bool
static PROXY_REQUESTS_STORE_KEY: &[u8] = b"ProxyRequests";


// Singleton structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct State {
    pub admin: Addr,
    // n_selected proxies will be between threshold and n_max_proxies
    pub threshold: u32,
    pub n_max_proxies: u32,

    // Total number of re-encryption requests
    pub next_request_id: u64,
}

// Store structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct DataEntry {
    pub delegator_pubkey: String,
    pub delegator_addr: Addr,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ReencryptionRequest {
    pub data_id: HashID,
    pub delegatee_pubkey: String,
    pub fragment: Option<HashID>,
}

// Getters and setters

// STATE
pub fn get_state(storage: &dyn Storage) -> StdResult<State> {
    singleton_read(storage, STATE_KEY).load()
}

pub fn set_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    let mut singl: Singleton<State> = singleton(storage, STATE_KEY);
    singl.save(&state)
}

// IS_PROXY
pub fn set_is_proxy(storage: &mut dyn Storage, proxy_addr: &Addr, is_proxy: bool) -> () {
    let mut store = PrefixedStorage::new(storage, IS_PROXY_KEY);

    // Any value in store means true - &[1]
    match is_proxy
    {
        true => store.set(proxy_addr.as_bytes(), &[1]),
        false => store.remove(proxy_addr.as_bytes())
    }
}

pub fn get_is_proxy(storage: &dyn Storage, proxy_addr: &Addr) -> bool {
    let store = ReadonlyPrefixedStorage::new(storage, IS_PROXY_KEY);

    match store.get(proxy_addr.as_bytes())
    {
        None => false,
        Some(_) => true
    }
}

pub fn get_all_proxies(storage: &dyn Storage) -> Vec<Addr> {
    let store = ReadonlyPrefixedStorage::new(storage, IS_PROXY_KEY);

    let mut deserialized_keys: Vec<Addr> = Vec::new();

    for pair in store.range(None, None, Order::Ascending)
    {
        // Deserialize keys with inverse operation to &proxy_addr.as_bytes()
        deserialized_keys.push(Addr::unchecked(std::str::from_utf8(&pair.0).unwrap()));
    }

    return deserialized_keys;
}

// PROXIES_AVAILABITY
pub fn set_proxy_availability(storage: &mut dyn Storage, pub_key: &String, proxy_addr: &Addr) -> () {
    let mut storage = PrefixedStorage::new(storage, PROXIES_AVAILABITY_KEY);

    storage.set( pub_key.as_bytes(), proxy_addr.as_bytes());
}

pub fn remove_proxy_availability(storage: &mut dyn Storage, pub_key: &String) -> () {
    let mut storage = PrefixedStorage::new(storage, PROXIES_AVAILABITY_KEY);

    storage.remove(pub_key.as_bytes());
}

pub fn get_proxy_availability(storage: &dyn Storage, pub_key: &String) -> Option<Addr> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXIES_AVAILABITY_KEY);

    let res = store.get(pub_key.as_bytes());
    match res
    {
        None => None,
        Some(res) => Some(Addr::unchecked(std::str::from_utf8(&res).unwrap()))
    }
}

// PROXIES_PUBKEYS
pub fn set_proxy_pubkey(storage: &mut dyn Storage, proxy_addr: &Addr, pub_key: &String) -> () {
    let mut storage = PrefixedStorage::new(storage, PROXIES_PUBKEYS_KEY);

    storage.set(proxy_addr.as_bytes(), pub_key.as_bytes());
}

pub fn remove_proxy_pubkey(storage: &mut dyn Storage, proxy_addr: &Addr) -> () {
    let mut storage = PrefixedStorage::new(storage, PROXIES_PUBKEYS_KEY);

    storage.remove(proxy_addr.as_bytes());
}

pub fn get_proxy_pubkey(storage: &dyn Storage, proxy_addr: &Addr) -> Option<String> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXIES_PUBKEYS_KEY);

    let res = store.get(proxy_addr.as_bytes());
    match res
    {
        None => None,
        Some(res) => Some(String::from_utf8(res).unwrap())
    }
}

pub fn get_all_available_proxy_pubkeys(storage: &dyn Storage) -> Vec<String> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXIES_PUBKEYS_KEY);

    let mut deserialized_keys: Vec<String> = Vec::new();

    for pair in store.range(None, None, Order::Ascending)
    {
        // Deserialize keys with inverse operation to &string.as_bytes()
        deserialized_keys.push(std::str::from_utf8(&pair.1).unwrap().to_string());
    }

    return deserialized_keys;
}

// DATA_ENTRIES
pub fn set_data_entry(storage: &mut dyn Storage, data_id: &HashID, data_entry: &DataEntry) -> () {
    let mut store = PrefixedStorage::new(storage, DATA_ENTRIES_KEY);
    store.set(data_id.as_bytes(), &to_vec(data_entry).unwrap());
}

pub fn remove_data_entry(storage: &mut dyn Storage, data_id: &HashID) -> () {
    let mut store = PrefixedStorage::new(storage, DATA_ENTRIES_KEY);

    store.remove(data_id.as_bytes());
}

pub fn get_data_entry(storage: &dyn Storage, data_id: &HashID) -> Option<DataEntry> {
    let store = ReadonlyPrefixedStorage::new(storage, DATA_ENTRIES_KEY);

    match store.get(data_id.as_bytes())
    {
        None => None,
        Some(data) => Some(from_slice(&data).unwrap())
    }
}

// DELEGATIONS_STORE
pub fn set_delegation_string(storage: &mut dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String, proxy_pubkey: &String, delegation_string: &Option<String>) -> () {
    let mut store = PrefixedStorage::multilevel(storage, &[DELEGATIONS_STORE_KEY, delegator_addr.as_bytes(), delegator_pubkey.as_bytes(), delegatee_pubkey.as_bytes()]);

    store.set(proxy_pubkey.as_bytes(), &to_vec(delegation_string).unwrap());
}

pub fn remove_delegation_string(storage: &mut dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String, proxy_pubkey: &String) -> () {
    let mut store = PrefixedStorage::multilevel(storage, &[DELEGATIONS_STORE_KEY, delegator_addr.as_bytes(), delegator_pubkey.as_bytes(), delegatee_pubkey.as_bytes()]);

    store.remove(proxy_pubkey.as_bytes());
}

pub fn get_delegation_string(storage: &dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String, proxy_pubkey: &String) -> Option<Option<String>> {
    let store = ReadonlyPrefixedStorage::multilevel(storage, &[DELEGATIONS_STORE_KEY, delegator_addr.as_bytes(), delegator_pubkey.as_bytes(), delegatee_pubkey.as_bytes()]);

    match store.get(proxy_pubkey.as_bytes())
    {
        None => None,
        Some(data) => Some(from_slice(&data).unwrap())
    }
}

pub fn get_all_proxies_from_delegation(storage: &dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String) -> Vec<String> {
    let store = ReadonlyPrefixedStorage::multilevel(storage, &[DELEGATIONS_STORE_KEY, delegator_addr.as_bytes(), delegator_pubkey.as_bytes(), delegatee_pubkey.as_bytes()]);

    let mut deserialized_keys: Vec<String> = Vec::new();

    for pair in store.range(None, None, Order::Ascending)
    {
        // Deserialize keys with inverse operation to string.as_bytes()
        deserialized_keys.push(std::str::from_utf8(&pair.0).unwrap().to_string());
    }

    return deserialized_keys;
}

pub fn is_delegation_empty(storage: &dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String) -> bool
{
    let store = ReadonlyPrefixedStorage::multilevel(storage, &[DELEGATIONS_STORE_KEY, delegator_addr.as_bytes(), delegator_pubkey.as_bytes(), delegatee_pubkey.as_bytes()]);

    for _ in store.range(None, None, Order::Ascending)
    {
        return false;
    }
    return true;
}

// REENCRYPTION_REQUESTS_STORE_KEY
pub fn set_reencryption_request(storage: &mut dyn Storage, reencryption_request_id: &u64, reencryption_request: &ReencryptionRequest) -> () {
    let mut store = PrefixedStorage::new(storage, &REENCRYPTION_REQUESTS_STORE_KEY);

    store.set(&reencryption_request_id.to_le_bytes(), &to_vec(reencryption_request).unwrap());
}

pub fn get_reencryption_request(storage: &dyn Storage, reencryption_request_id: &u64) -> Option<ReencryptionRequest> {
    let store = ReadonlyPrefixedStorage::new(storage, &REENCRYPTION_REQUESTS_STORE_KEY);

    match store.get(&reencryption_request_id.to_le_bytes())
    {
        None => None,
        Some(data) => Some(from_slice(&data).unwrap())
    }
}

// DELEGATEE_REQUESTS_STORE
pub fn add_delegatee_reencryption_request(storage: &mut dyn Storage, data_id: &HashID, delegatee_pubkey: &String, proxy_pubkey: &String, reencryption_request_id: &u64) -> () {
    let mut store = PrefixedStorage::multilevel(storage, &[DELEGATEE_REQUESTS_STORE_KEY, data_id.as_bytes(), delegatee_pubkey.as_bytes()]);

    store.set(&proxy_pubkey.as_bytes(), &reencryption_request_id.to_le_bytes());
}

pub fn get_delegatee_reencryption_request(storage: &mut dyn Storage, data_id: &HashID, delegatee_pubkey: &String, proxy_pubkey: &String) -> Option<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(storage, &[DELEGATEE_REQUESTS_STORE_KEY, data_id.as_bytes(), delegatee_pubkey.as_bytes()]);

    match store.get(proxy_pubkey.as_bytes())
    {
        None => None,
        Some(data) => Some(u64::from_le_bytes(data.try_into().unwrap()))
    }
}

pub fn get_all_delegatee_reencryption_requests(storage: &dyn Storage, data_id: &HashID, delegatee_pubkey: &String) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(storage, &[DELEGATEE_REQUESTS_STORE_KEY, data_id.as_bytes(), delegatee_pubkey.as_bytes()]);

    let mut deserialized_keys: Vec<u64> = Vec::new();

    for pair in store.range(None, None, Order::Ascending)
    {
        // Deserialize keys with inverse operation to to_vec
        deserialized_keys.push(u64::from_le_bytes(pair.1.try_into().unwrap()));
    }

    return deserialized_keys;
}

// PROXY_REQUESTS_STORE_KEY
pub fn add_proxy_reencryption_request(storage: &mut dyn Storage, proxy_pubkey: &String, reencryption_request_id: &u64) -> () {
    let mut store = PrefixedStorage::multilevel(storage, &[PROXY_REQUESTS_STORE_KEY, proxy_pubkey.as_bytes()]);

    // Any value in store means true - &[1]
    store.set(&reencryption_request_id.to_le_bytes(), &[1]);
}

pub fn remove_proxy_reencryption_request(storage: &mut dyn Storage, proxy_pubkey: &String, reencryption_request_id: &u64) -> () {
    let mut store = PrefixedStorage::multilevel(storage, &[PROXY_REQUESTS_STORE_KEY, proxy_pubkey.as_bytes()]);

    store.remove(&reencryption_request_id.to_le_bytes());
}

pub fn is_proxy_reencryption_request(storage: &dyn Storage, proxy_pubkey: &String, reencryption_request_id: &u64) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(storage, &[PROXY_REQUESTS_STORE_KEY, proxy_pubkey.as_bytes()]);

    match store.get(&reencryption_request_id.to_le_bytes())
    {
        None => false,
        Some(_) => true
    }
}

pub fn get_all_proxy_reencryption_requests(storage: &dyn Storage, proxy_pubkey: &String) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(storage, &[PROXY_REQUESTS_STORE_KEY, proxy_pubkey.as_bytes()]);

    let mut deserialized_keys: Vec<u64> = Vec::new();

    for pair in store.range(None, None, Order::Ascending)
    {
        // Deserialize keys with inverse operation to to_vec
        deserialized_keys.push(u64::from_le_bytes(pair.0.try_into().unwrap()));
    }

    return deserialized_keys;
}