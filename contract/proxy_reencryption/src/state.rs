use cosmwasm_std::{Addr, Storage, StdResult, to_vec, StdError, Order, from_slice};
use cosmwasm_storage::{singleton, singleton_read, Singleton, Bucket, ReadonlyBucket, bucket_read, bucket, PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

use crate::types::{set_bool_store, get_bool_store, get_all_keys, get_all_keys_multilevel, set_bool_store_multilevel, get_bool_store_multilevel, get_all_values_multilevel};

pub type HashID = String;

// Singletons
static STATE_KEY: &[u8] = b"State";

// Maps

// Proxy register whitelist
// Map Addr proxy -> bool
static IS_PROXY_KEY: &[u8] = b"IsProxy";

// Map Addr proxy -> String proxy_pubkey
static PROXIES_AVAILABITY_KEY: &[u8] = b"ProxyAvailable";

// Counts number of proxies with the same pubkey
// Used for selecting proxy pubkeys for delegations
// Map String proxy_pubkey -> u32 n_addresses
static PROXIES_PUBKEYS_KEY: &[u8] = b"ProxyPubkey";

// Map String data_id -> DataEntry data_entry
static DATA_ENTRIES_KEY: &[u8] = b"DataEntries";

// Map Addr delegator_addr -> String delegator_pubkey -> String delegatee_pubkey -> String proxy_pubkey -> Option<String> delegation_string
static DELEGATIONS_STORE_KEY: &[u8] = b"DelegationStore";

// Map String proxy_pubkey -> ReencryptionRequest reencryption_request -> bool is_reencryption_request
static REENCRYPTION_REQUESTS_STORE_KEY: &[u8] = b"ReencryptionRequests";

// Map String data_id -> String delegatee_pubkey -> String proxy_pubkey -> HashID reencrypted_cap_fragment
static FRAGMENTS_STORE_KEY: &[u8] = b"Fragments";


// Singleton structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct State {
    pub admin: Addr,
    // n_selected proxies will be between threshold and n_max_proxies
    pub threshold: u32,
    pub n_max_proxies: u32,
}

// Store structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct DataEntry {
    pub delegator_pubkey: String,
    pub delegator_addr: Addr,
}


// Other structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ReencryptionRequest {
    pub data_id: HashID,
    pub delegatee_pubkey: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyTask {
    pub data_id: HashID,
    pub delegatee_pubkey: String,
    pub delegator_pubkey: String,
    pub delegation_string: String,
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

    match is_proxy
    {
        true => store.set(proxy_addr.as_bytes(), &[0]),
        false => store.remove(proxy_addr.as_bytes())
    }

}

pub fn get_is_proxy(storage: &dyn Storage, proxy_addr: &Addr) -> bool{
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
        deserialized_keys.push( Addr::unchecked(std::str::from_utf8(&pair.0).unwrap()));
    }

    return deserialized_keys;
}

// PROXIES_AVAILABITY
pub fn set_proxy_availability(storage: &mut dyn Storage, proxy_addr: &Addr, pub_key: &String) -> () {
    let mut storage = PrefixedStorage::new(storage, PROXIES_AVAILABITY_KEY);

    storage.set(proxy_addr.as_bytes(), pub_key.as_bytes());
}

pub fn remove_proxy_availability(storage: &mut dyn Storage, proxy_addr: &Addr) -> () {
    let mut storage = PrefixedStorage::new(storage, PROXIES_AVAILABITY_KEY);

    storage.remove(proxy_addr.as_bytes());
}

pub fn get_proxy_availability(storage: &dyn Storage, proxy_addr: &Addr) -> Option<String> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXIES_AVAILABITY_KEY);

    let res = store.get(proxy_addr.as_bytes());
    match res
    {
        None => None,
        Some(res) => Some(String::from_utf8(res).unwrap())
    }

}


// PROXIES_PUBKEYS_KEY


pub fn get_all_available_proxy_pubkeys(storage: &dyn Storage) -> Vec<String> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXIES_PUBKEYS_KEY);

    let mut deserialized_keys: Vec<String> = Vec::new();

    for pair in store.range(None, None, Order::Ascending)
    {
        // Deserialize keys with inverse operation to &string.as_bytes()
        deserialized_keys.push( std::str::from_utf8(&pair.0).unwrap().to_string());
    }

    return deserialized_keys;
}


pub fn increase_available_proxy_pubkeys(storage: &mut dyn Storage, proxy_pubkey: &String) -> () {
    let mut store = PrefixedStorage::new(storage, PROXIES_PUBKEYS_KEY);


    match store.get(proxy_pubkey.as_bytes())
    {
        None => { store.set(proxy_pubkey.as_bytes(), &1_u32.to_le_bytes()) }
        Some(n) => { store.set(proxy_pubkey.as_bytes(), &(u32::from_le_bytes(n.try_into().unwrap()) + 1).to_le_bytes() ) }
    }
}

pub fn decrease_available_proxy_pubkeys(storage: &mut dyn Storage, proxy_pubkey: &String) -> () {
    let mut store = PrefixedStorage::new(storage, PROXIES_PUBKEYS_KEY);

    match store.get(proxy_pubkey.as_bytes())
    {
        None => { panic!("Number of pubkeys is already 0") }
        Some(res) =>
            {
                let n:u32 = u32::from_le_bytes(res.try_into().unwrap());
                if n == 1
                {
                    store.remove(proxy_pubkey.as_bytes())
                } else {
                    store.set(proxy_pubkey.as_bytes(), &(n - 1).to_le_bytes())
                }
            }
    }
}

pub fn get_n_available_proxy_pubkeys(storage: &dyn Storage, proxy_pubkey: &String) -> u32 {
    let store = ReadonlyPrefixedStorage::new(storage, PROXIES_PUBKEYS_KEY);

    match store.get(proxy_pubkey.as_bytes())
    {
        None => 0,
        Some(n) => u32::from_le_bytes(n.try_into().unwrap()),
    }
}


// DATA_ENTRIES
pub fn set_data_entry(storage: &mut dyn Storage, data_id: &HashID, data_entry: &DataEntry) -> StdResult<()> {
    let mut bucket: Bucket<DataEntry> = bucket(storage, DATA_ENTRIES_KEY);

    return bucket.save(&to_vec(data_id)?, data_entry);
}

pub fn remove_data_entry(storage: &mut dyn Storage, data_id: &HashID) -> StdResult<()> {
    let mut bucket: Bucket<DataEntry> = bucket(storage, DATA_ENTRIES_KEY);

    bucket.remove(&to_vec(data_id)?);
    Ok(())
}

pub fn get_data_entry(storage: &dyn Storage, data_id: &HashID) -> StdResult<Option<DataEntry>> {
    let bucket: ReadonlyBucket<DataEntry> = bucket_read(storage, DATA_ENTRIES_KEY);

    bucket.may_load(&to_vec(data_id)?)
}

// DELEGATIONS_STORE
pub fn set_delegation_string(storage: &mut dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String, proxy_pubkey: &String, delegation_string: &Option<String>) -> StdResult<()> {
    let mut bucket: Bucket<Option<String>> = Bucket::multilevel(storage, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);

    return bucket.save(&to_vec(proxy_pubkey)?, delegation_string);
}

pub fn remove_delegation_string(storage: &mut dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String, proxy_pubkey: &String) -> StdResult<()> {
    let mut bucket: Bucket<Option<String>> = Bucket::multilevel(storage, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);

    bucket.remove(&to_vec(proxy_pubkey)?);
    Ok(())
}

pub fn get_delegation_string(storage: &dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String, proxy_pubkey: &String) -> StdResult<Option<Option<String>>> {
    let bucket: ReadonlyBucket<Option<String>> = ReadonlyBucket::multilevel(storage, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);

    bucket.may_load(&to_vec(proxy_pubkey)?)
}

pub fn get_all_proxies_from_delegation(storage: &dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String) -> StdResult<Vec<String>> {
    return get_all_keys_multilevel::<String, Option<String>>(storage, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);
}

pub fn is_delegation_empty(storage: &dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String) -> StdResult<bool>
{
    let bucket: ReadonlyBucket<Option<String>> = ReadonlyBucket::multilevel(storage, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);

    for _ in bucket.range(None, None, Order::Ascending)
    {
        return Ok(false);
    }
    return Ok(true);
}


// FRAGMENTS_STORE
pub fn set_fragment(storage: &mut dyn Storage, data_id: &HashID, delegatee_pubkey: &String, proxy_pubkey: &String, reencrypted_cap_fragment: &HashID) -> StdResult<()> {
    let mut bucket: Bucket<HashID> = Bucket::multilevel(storage, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);

    return bucket.save(&to_vec(proxy_pubkey)?, reencrypted_cap_fragment);
}

pub fn remove_fragment(storage: &mut dyn Storage, data_id: &HashID, delegatee_pubkey: &String, proxy_pubkey: &String) -> StdResult<()> {
    let mut bucket: Bucket<HashID> = Bucket::multilevel(storage, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);

    bucket.remove(&to_vec(proxy_pubkey)?);
    Ok(())
}

pub fn get_fragment(storage: &dyn Storage, data_id: &HashID, delegatee_pubkey: &String, proxy_pubkey: &String) -> StdResult<Option<HashID>> {
    let bucket: ReadonlyBucket<HashID> = ReadonlyBucket::multilevel(storage, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);

    bucket.may_load(&to_vec(proxy_pubkey)?)
}

pub fn get_all_fragments(storage: &dyn Storage, data_id: &HashID, delegatee_pubkey: &String) -> StdResult<Vec<HashID>> {
    return get_all_values_multilevel::<HashID>(storage, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);
}


// REENCRYPTION_REQUESTS_STORE
pub fn add_reencryption_request(storage: &mut dyn Storage, proxy_pubkey: &String, reencryption_request: &ReencryptionRequest) -> StdResult<()> {
    return set_bool_store_multilevel(storage, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_pubkey)?], &to_vec(reencryption_request)?, true);
}

pub fn remove_reencryption_request(storage: &mut dyn Storage, proxy_pubkey: &String, reencryption_request: &ReencryptionRequest) -> StdResult<()> {
    return set_bool_store_multilevel(storage, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_pubkey)?], &to_vec(reencryption_request)?, false);
}

pub fn is_reencryption_request(storage: &dyn Storage, proxy_pubkey: &String, reencryption_request: &ReencryptionRequest) -> StdResult<bool> {
    return get_bool_store_multilevel(storage, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_pubkey)?], &to_vec(reencryption_request)?);
}

pub fn get_all_reencryption_requests(storage: &dyn Storage, proxy_pubkey: &String) -> StdResult<Vec<ReencryptionRequest>> {
    return get_all_keys_multilevel::<ReencryptionRequest, bool>(storage, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_pubkey)?]);
}


