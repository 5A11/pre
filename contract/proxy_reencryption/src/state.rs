use cosmwasm_std::{Addr, Storage, StdResult, to_vec, StdError, Order};
use cosmwasm_storage::{singleton, singleton_read, Singleton, Bucket, ReadonlyBucket, bucket_read, bucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
pub fn set_is_proxy(store: &mut dyn Storage, proxy_addr: &Addr, is_proxy: bool) -> StdResult<()> {
    return set_bool_store(store, IS_PROXY_KEY, &to_vec(proxy_addr)?, is_proxy);
}

pub fn get_is_proxy(store: &dyn Storage, proxy_addr: &Addr) -> StdResult<bool> {
    return get_bool_store(store, IS_PROXY_KEY, &to_vec(proxy_addr)?);
}

pub fn get_all_proxies(store: &dyn Storage) -> StdResult<Vec<Addr>> {
    return get_all_keys::<Addr, bool>(store, IS_PROXY_KEY);
}

// PROXIES_AVAILABITY
pub fn set_proxy_availability(store: &mut dyn Storage, proxy_addr: &Addr, pub_key: &String) -> StdResult<()> {
    let mut bucket: Bucket<String> = bucket(store, PROXIES_AVAILABITY_KEY);

    return bucket.save(&to_vec(proxy_addr)?, pub_key);
}

pub fn remove_proxy_availability(store: &mut dyn Storage, proxy_addr: &Addr) -> StdResult<()> {
    let mut bucket: Bucket<String> = bucket(store, PROXIES_AVAILABITY_KEY);

    bucket.remove(&to_vec(proxy_addr)?);
    Ok(())
}

pub fn get_proxy_availability(store: &dyn Storage, proxy_addr: &Addr) -> StdResult<Option<String>> {
    let bucket: ReadonlyBucket<String> = bucket_read(store, PROXIES_AVAILABITY_KEY);

    bucket.may_load(&to_vec(proxy_addr)?)
}


// PROXIES_PUBKEYS_KEY


pub fn get_all_available_proxy_pubkeys(store: &dyn Storage) -> StdResult<Vec<String>> {
    get_all_keys::<String, u32>(store, PROXIES_PUBKEYS_KEY)
}


pub fn increase_available_proxy_pubkeys(store: &mut dyn Storage, proxy_pubkey: &String) -> StdResult<()> {
    let mut bucket: Bucket<u32> = bucket(store, PROXIES_PUBKEYS_KEY);

    let n_keys = bucket.may_load(&to_vec(proxy_pubkey)?)?;

    match n_keys
    {
        None => { bucket.save(&to_vec(proxy_pubkey)?, &1) }
        Some(n) => { bucket.save(&to_vec(proxy_pubkey)?, &(n + 1)) }
    }
}

pub fn decrease_available_proxy_pubkeys(store: &mut dyn Storage, proxy_pubkey: &String) -> StdResult<()> {
    let mut bucket: Bucket<u32> = bucket(store, PROXIES_PUBKEYS_KEY);

    let n_keys = bucket.may_load(&to_vec(proxy_pubkey)?)?;

    match n_keys
    {
        None => { Err(StdError::generic_err("Number of pubkeys is already 0")) }
        Some(n) =>
            {
                if n == 1
                {
                    Ok(bucket.remove(&to_vec(proxy_pubkey)?))
                } else {
                    bucket.save(&to_vec(proxy_pubkey)?, &(n - 1))
                }
            }
    }
}

pub fn get_n_available_proxy_pubkeys(store: &dyn Storage, proxy_pubkey: &String) -> StdResult<u32> {
    let bucket: ReadonlyBucket<u32> = bucket_read(store, PROXIES_PUBKEYS_KEY);

    match bucket.may_load(&to_vec(proxy_pubkey)?)?
    {
        None => Ok(0),
        Some(n) => Ok(n),
    }
}


// DATA_ENTRIES
pub fn set_data_entry(store: &mut dyn Storage, data_id: &HashID, data_entry: &DataEntry) -> StdResult<()> {
    let mut bucket: Bucket<DataEntry> = bucket(store, DATA_ENTRIES_KEY);

    return bucket.save(&to_vec(data_id)?, data_entry);
}

pub fn remove_data_entry(store: &mut dyn Storage, data_id: &HashID) -> StdResult<()> {
    let mut bucket: Bucket<DataEntry> = bucket(store, DATA_ENTRIES_KEY);

    bucket.remove(&to_vec(data_id)?);
    Ok(())
}

pub fn get_data_entry(store: &dyn Storage, data_id: &HashID) -> StdResult<Option<DataEntry>> {
    let bucket: ReadonlyBucket<DataEntry> = bucket_read(store, DATA_ENTRIES_KEY);

    bucket.may_load(&to_vec(data_id)?)
}

// DELEGATIONS_STORE
pub fn set_delegation_string(store: &mut dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String, proxy_pubkey: &String, delegation_string: &Option<String>) -> StdResult<()> {
    let mut bucket: Bucket<Option<String>> = Bucket::multilevel(store, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);

    return bucket.save(&to_vec(proxy_pubkey)?, delegation_string);
}

pub fn remove_delegation_string(store: &mut dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String, proxy_pubkey: &String) -> StdResult<()> {
    let mut bucket: Bucket<Option<String>> = Bucket::multilevel(store, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);

    bucket.remove(&to_vec(proxy_pubkey)?);
    Ok(())
}

pub fn get_delegation_string(store: &dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String, proxy_pubkey: &String) -> StdResult<Option<Option<String>>> {
    let bucket: ReadonlyBucket<Option<String>> = ReadonlyBucket::multilevel(store, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);

    bucket.may_load(&to_vec(proxy_pubkey)?)
}

pub fn get_all_proxies_from_delegation(store: &dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String) -> StdResult<Vec<String>> {
    return get_all_keys_multilevel::<String, Option<String>>(store, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);
}

pub fn is_delegation_empty(store: &dyn Storage, delegator_addr: &Addr, delegator_pubkey: &String, delegatee_pubkey: &String) -> StdResult<bool>
{
    let bucket: ReadonlyBucket<Option<String>> = ReadonlyBucket::multilevel(store, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegator_pubkey)?, &to_vec(delegatee_pubkey)?]);

    for _ in bucket.range(None, None, Order::Ascending)
    {
        return Ok(false);
    }
    return Ok(true);
}


// FRAGMENTS_STORE
pub fn set_fragment(store: &mut dyn Storage, data_id: &HashID, delegatee_pubkey: &String, proxy_pubkey: &String, reencrypted_cap_fragment: &HashID) -> StdResult<()> {
    let mut bucket: Bucket<HashID> = Bucket::multilevel(store, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);

    return bucket.save(&to_vec(proxy_pubkey)?, reencrypted_cap_fragment);
}

pub fn remove_fragment(store: &mut dyn Storage, data_id: &HashID, delegatee_pubkey: &String, proxy_pubkey: &String) -> StdResult<()> {
    let mut bucket: Bucket<HashID> = Bucket::multilevel(store, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);

    bucket.remove(&to_vec(proxy_pubkey)?);
    Ok(())
}

pub fn get_fragment(store: &dyn Storage, data_id: &HashID, delegatee_pubkey: &String, proxy_pubkey: &String) -> StdResult<Option<HashID>> {
    let bucket: ReadonlyBucket<HashID> = ReadonlyBucket::multilevel(store, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);

    bucket.may_load(&to_vec(proxy_pubkey)?)
}

pub fn get_all_fragments(store: &dyn Storage, data_id: &HashID, delegatee_pubkey: &String) -> StdResult<Vec<HashID>> {
    return get_all_values_multilevel::<HashID>(store, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);
}


// REENCRYPTION_REQUESTS_STORE
pub fn add_reencryption_request(store: &mut dyn Storage, proxy_pubkey: &String, reencryption_request: &ReencryptionRequest) -> StdResult<()> {
    return set_bool_store_multilevel(store, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_pubkey)?], &to_vec(reencryption_request)?, true);
}

pub fn remove_reencryption_request(store: &mut dyn Storage, proxy_pubkey: &String, reencryption_request: &ReencryptionRequest) -> StdResult<()> {
    return set_bool_store_multilevel(store, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_pubkey)?], &to_vec(reencryption_request)?, false);
}

pub fn is_reencryption_request(store: &dyn Storage, proxy_pubkey: &String, reencryption_request: &ReencryptionRequest) -> StdResult<bool> {
    return get_bool_store_multilevel(store, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_pubkey)?], &to_vec(reencryption_request)?);
}

pub fn get_all_reencryption_requests(store: &dyn Storage, proxy_pubkey: &String) -> StdResult<Vec<ReencryptionRequest>> {
    return get_all_keys_multilevel::<ReencryptionRequest, bool>(store, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_pubkey)?]);
}


