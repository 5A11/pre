use cosmwasm_std::{from_slice, to_vec, Addr, StdResult, Storage, Uint128};
use cosmwasm_storage::{
    singleton, singleton_read, PrefixedStorage, ReadonlyPrefixedStorage, Singleton,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Singletons
static STATE_KEY: &[u8] = b"State";

static STAKING_CONFIG_KEY: &[u8] = b"StakingConfig";

// Maps

// Map data_id: String -> data_entry: DataEntry
static DATA_ENTRIES_KEY: &[u8] = b"DataEntries";

// Map delegator_pubkey: String -> delegator_addr: Addr
static DELEGATOR_ADDRESS_KEY: &[u8] = b"DelegatorAddr";

// Singleton structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct State {
    pub admin: Addr,
    // n_selected proxies will be between threshold and n_max_proxies
    pub threshold: u32,
    pub n_max_proxies: u32,

    // Total number of re-encryption requests
    pub next_proxy_request_id: u64,
    pub next_delegation_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct StakingConfig {
    pub stake_denom: String,
    pub minimum_proxy_stake_amount: Uint128,
    pub per_proxy_request_reward_amount: Uint128,
    pub per_request_slash_stake_amount: Uint128,
}

// Store structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct DataEntry {
    pub delegator_pubkey: String,
    pub capsule: String,
}

// Getters and setters

// STATE
pub fn store_get_state(storage: &dyn Storage) -> StdResult<State> {
    singleton_read(storage, STATE_KEY).load()
}

pub fn store_set_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    let mut singl: Singleton<State> = singleton(storage, STATE_KEY);
    singl.save(state)
}

// STAKING_CONFIG
pub fn store_get_staking_config(storage: &dyn Storage) -> StdResult<StakingConfig> {
    singleton_read(storage, STAKING_CONFIG_KEY).load()
}

pub fn store_set_staking_config(
    storage: &mut dyn Storage,
    staking_config: &StakingConfig,
) -> StdResult<()> {
    let mut singl: Singleton<StakingConfig> = singleton(storage, STAKING_CONFIG_KEY);
    singl.save(staking_config)
}

// DATA_ENTRIES
pub fn store_set_data_entry(storage: &mut dyn Storage, data_id: &str, data_entry: &DataEntry) {
    let mut store = PrefixedStorage::new(storage, DATA_ENTRIES_KEY);
    store.set(data_id.as_bytes(), &to_vec(data_entry).unwrap());
}

pub fn store_get_data_entry(storage: &dyn Storage, data_id: &str) -> Option<DataEntry> {
    let store = ReadonlyPrefixedStorage::new(storage, DATA_ENTRIES_KEY);

    store
        .get(data_id.as_bytes())
        .map(|data| from_slice(&data).unwrap())
}

// DELEGATOR_ADDRESS
pub fn store_set_delegator_address(
    storage: &mut dyn Storage,
    delegator_pubkey: &str,
    delegator_addr: &Addr,
) {
    let mut storage = PrefixedStorage::new(storage, DELEGATOR_ADDRESS_KEY);

    storage.set(delegator_pubkey.as_bytes(), delegator_addr.as_bytes());
}

pub fn store_get_delegator_address(storage: &dyn Storage, delegator_pubkey: &str) -> Option<Addr> {
    let store = ReadonlyPrefixedStorage::new(storage, DELEGATOR_ADDRESS_KEY);

    let res = store.get(delegator_pubkey.as_bytes());
    res.map(|res| Addr::unchecked(String::from_utf8(res).unwrap()))
}
