use cosmwasm_std::{Addr, Storage, StdResult, Binary, to_vec};
use cosmwasm_storage::{singleton, singleton_read, Singleton, Bucket, ReadonlyBucket, bucket_read, bucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::types::{set_bool_store, get_bool_store, get_all_keys, get_all_keys_multilevel, set_bool_store_multilevel, get_bool_store_multilevel, get_all_values_multilevel};

pub type DataId = String;

// Singletons
static STATE_KEY: &[u8] = b"State";

// Maps
// Map Addr proxy -> bool
static IS_PROXY_KEY: &[u8] = b"IsProxy";

// Map Addr proxy -> bool
static PROXIES_AVAILABITY_KEY: &[u8] = b"ProxyAvailable";

// Map String data_id -> DataEntry data_entry
static DATA_ENTRIES_KEY: &[u8] = b"DataEntries";

// Map String data_id -> Addr owner
static DATA_ENTRY_OWNERS_KEY: &[u8] = b"DataEntryOwners";

// Map Addr delegator_addr -> Binary delegatee_pubkey -> Addr proxy_addr -> Binary delegation_string
static DELEGATIONS_STORE_KEY: &[u8] = b"DelegationStore";

// Map String data_id -> Binary delegatee_pubkey -> Addr proxy_addr -> DataId reencrypted_cap_fragment
static FRAGMENTS_STORE_KEY: &[u8] = b"Fragments";

// Map Addr proxy -> ReencryptionRequest reencryption_request -> bool is_reencryption_request
static REENCRYPTION_REQUESTS_STORE_KEY: &[u8] = b"ReencryptionRequests";


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
    pub delegator_pubkey: Binary,
}


// Other structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ReencryptionRequest {
    pub data_id: DataId,
    pub delegatee_pubkey: Binary,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyTask {
    pub data_id: DataId,
    pub delegatee_pubkey: Binary,
    pub delegator_pubkey: Binary,
    pub delegation_string: Binary,
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
pub fn add_proxy_availability(store: &mut dyn Storage, proxy_addr: &Addr) -> StdResult<()> {
    let mut bucket: Bucket<bool> = bucket(store, PROXIES_AVAILABITY_KEY);
    return bucket.save(&to_vec(proxy_addr)?, &true);
}

pub fn remove_proxy_availability(store: &mut dyn Storage, proxy_addr: &Addr) -> StdResult<()> {
    let mut bucket: Bucket<Binary> = bucket(store, PROXIES_AVAILABITY_KEY);

    bucket.remove(&to_vec(proxy_addr)?);
    Ok(())
}

pub fn get_proxy_availability(store: &dyn Storage, proxy_addr: &Addr) -> StdResult<bool> {
    get_bool_store(store, PROXIES_AVAILABITY_KEY, &to_vec(proxy_addr)?)
}

pub fn get_available_proxies(store: &dyn Storage) -> StdResult<Vec<Addr>> {
    get_all_keys::<Addr, bool>(store, PROXIES_AVAILABITY_KEY)
}


// DATA_ENTRIES
pub fn set_data_entry(store: &mut dyn Storage, data_id: &DataId, data_entry: &DataEntry) -> StdResult<()> {
    let mut bucket: Bucket<DataEntry> = bucket(store, DATA_ENTRIES_KEY);

    return bucket.save(&to_vec(data_id)?, data_entry);
}

pub fn remove_data_entry(store: &mut dyn Storage, data_id: &DataId) -> StdResult<()> {
    let mut bucket: Bucket<DataEntry> = bucket(store, DATA_ENTRIES_KEY);

    bucket.remove(&to_vec(data_id)?);
    Ok(())
}

pub fn get_data_entry(store: &dyn Storage, data_id: &DataId) -> StdResult<Option<DataEntry>> {
    let bucket: ReadonlyBucket<DataEntry> = bucket_read(store, DATA_ENTRIES_KEY);

    bucket.may_load(&to_vec(data_id)?)
}


// DATA_ENTRY_OWNERS
pub fn set_data_entry_owner(store: &mut dyn Storage, data_id: &DataId, owner: &Addr) -> StdResult<()> {
    let mut bucket: Bucket<Addr> = bucket(store, DATA_ENTRY_OWNERS_KEY);

    return bucket.save(&to_vec(data_id)?, owner);
}

pub fn remove_data_entry_owner(store: &mut dyn Storage, data_id: &DataId) -> StdResult<()> {
    let mut bucket: Bucket<Addr> = bucket(store, DATA_ENTRY_OWNERS_KEY);

    bucket.remove(&to_vec(data_id)?);
    Ok(())
}

pub fn get_data_entry_owner(store: &dyn Storage, data_id: &DataId) -> StdResult<Option<Addr>> {
    let bucket: ReadonlyBucket<Addr> = bucket_read(store, DATA_ENTRY_OWNERS_KEY);

    bucket.may_load(&to_vec(data_id)?)
}


// DELEGATIONS_STORE
pub fn set_delegation_string(store: &mut dyn Storage, delegator_addr: &Addr, delegatee_pubkey: &Binary, proxy_addr: &Addr, delegation: &Binary) -> StdResult<()> {
    let mut bucket: Bucket<Binary> = Bucket::multilevel(store, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegatee_pubkey)?]);

    return bucket.save(&to_vec(proxy_addr)?, delegation);
}

pub fn remove_delegation_string(store: &mut dyn Storage, delegator_addr: &Addr, delegatee_pubkey: &Binary, proxy_addr: &Addr) -> StdResult<()> {
    let mut bucket: Bucket<Binary> = Bucket::multilevel(store, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegatee_pubkey)?]);

    bucket.remove(&to_vec(proxy_addr)?);
    Ok(())
}

pub fn get_delegation_string(store: &dyn Storage, delegator_addr: &Addr, delegatee_pubkey: &Binary, proxy_addr: &Addr) -> StdResult<Option<Binary>> {
    let bucket: ReadonlyBucket<Binary> = ReadonlyBucket::multilevel(store, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegatee_pubkey)?]);

    bucket.may_load(&to_vec(proxy_addr)?)
}

pub fn get_all_proxies_from_delegation(store: &dyn Storage, delegator_addr: &Addr, delegatee_pubkey: &Binary) -> StdResult<Vec<Addr>> {
    return get_all_keys_multilevel::<Addr, Binary>(store, &[DELEGATIONS_STORE_KEY, &to_vec(delegator_addr)?, &to_vec(delegatee_pubkey)?]);
}


// FRAGMENTS_STORE
pub fn set_fragment(store: &mut dyn Storage, data_id: &DataId, delegatee_pubkey: &Binary, proxy_addr: &Addr, reencrypted_cap_fragment: &DataId) -> StdResult<()> {
    let mut bucket: Bucket<DataId> = Bucket::multilevel(store, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);

    return bucket.save(&to_vec(proxy_addr)?, reencrypted_cap_fragment);
}

pub fn remove_fragment(store: &mut dyn Storage, data_id: &DataId, delegatee_pubkey: &Binary, proxy_addr: &Addr) -> StdResult<()> {
    let mut bucket: Bucket<DataId> = Bucket::multilevel(store, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);

    bucket.remove(&to_vec(proxy_addr)?);
    Ok(())
}

pub fn get_fragment(store: &dyn Storage, data_id: &DataId, delegatee_pubkey: &Binary, proxy_addr: &Addr) -> StdResult<Option<DataId>> {
    let bucket: ReadonlyBucket<DataId> = ReadonlyBucket::multilevel(store, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);

    bucket.may_load(&to_vec(proxy_addr)?)
}

pub fn get_all_fragments(store: &dyn Storage, data_id: &DataId, delegatee_pubkey: &Binary) -> StdResult<Vec<DataId>> {
    return get_all_values_multilevel::<DataId>(store, &[FRAGMENTS_STORE_KEY, &to_vec(data_id)?, &to_vec(delegatee_pubkey)?]);
}


// REENCRYPTION_REQUESTS_STORE
pub fn add_reencryption_request(store: &mut dyn Storage, proxy_addr: &Addr, reencryption_request: &ReencryptionRequest) -> StdResult<()> {
    return set_bool_store_multilevel(store, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_addr)?], &to_vec(reencryption_request)?, true);
}

pub fn remove_reencryption_request(store: &mut dyn Storage, proxy_addr: &Addr, reencryption_request: &ReencryptionRequest) -> StdResult<()> {
    return set_bool_store_multilevel(store, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_addr)?], &to_vec(reencryption_request)?, false);
}

pub fn is_reencryption_request(store: &dyn Storage, proxy_addr: &Addr, reencryption_request: &ReencryptionRequest) -> StdResult<bool> {
    return get_bool_store_multilevel(store, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_addr)?], &to_vec(reencryption_request)?);
}

pub fn get_all_reencryption_requests(store: &dyn Storage, proxy_addr: &Addr) -> StdResult<Vec<ReencryptionRequest>> {
    return get_all_keys_multilevel::<ReencryptionRequest, bool>(store, &[REENCRYPTION_REQUESTS_STORE_KEY, &to_vec(proxy_addr)?]);
}


