use crate::proxies::{store_get_proxy_address, store_get_proxy_entry};
use crate::state::{store_get_staking_config, StakingConfig};
use cosmwasm_std::{from_slice, to_vec, Order, StdResult, Storage, Uint128};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

// To get all proxies from 1 delegation
// Map delegator_pubkey: String -> delegatee_pubkey: String -> proxy_pubkey: String -> delegation_id: u64
static PROXY_DELEGATIONS_ID_STORE_KEY: &[u8] = b"ProxyDelegationIDStore";

// To get all delegations for proxy
// Map proxy_pubkey: String -> delegation_id: u64 -> is_delegation: bool
static PER_PROXY_DELEGATIONS_STORE_KEY: &[u8] = b"PerProxyDelegationsStore";

// Map delegation_id: u64 -> delegation: ProxyDelegation
static PROXY_DELEGATIONS_STORE_KEY: &[u8] = b"ProxyDelegationsStore";

// Map delegation_id: u64 -> delegation: DelegationInfo
static DELEGATION_INFO_STORE_KEY: &[u8] = b"ProxyDelegationsStore";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct DelegationInfo {
    // we can extend this struct for custom access control policy implementation
    pub minimum_proxies_for_reencryption: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyDelegation {
    pub delegator_pubkey: String,
    pub delegatee_pubkey: String,
    pub delegation_string: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DelegationState {
    NonExisting,
    Active,
    ProxiesAreBusy,
}

// PROXY_DELEGATIONS_ID_STORE_KEY
pub fn store_set_delegation_id(
    storage: &mut dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
    delegation_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            PROXY_DELEGATIONS_ID_STORE_KEY,
            delegator_pubkey.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.set(proxy_pubkey.as_bytes(), &delegation_id.to_le_bytes());
}

pub fn store_remove_proxy_delegation_id(
    storage: &mut dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            PROXY_DELEGATIONS_ID_STORE_KEY,
            delegator_pubkey.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.remove(proxy_pubkey.as_bytes());
}

pub fn store_get_proxy_delegation_id(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) -> Option<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            PROXY_DELEGATIONS_ID_STORE_KEY,
            delegator_pubkey.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store
        .get(proxy_pubkey.as_bytes())
        .map(|data| u64::from_le_bytes(data.try_into().unwrap()))
}

pub fn store_get_all_proxies_from_delegation(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
) -> Vec<String> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            PROXY_DELEGATIONS_ID_STORE_KEY,
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

pub fn get_delegation_id(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
) -> Option<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            PROXY_DELEGATIONS_ID_STORE_KEY,
            delegator_pubkey.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    let delegation_id = store
        .range(None, None, Order::Ascending)
        .next()
        .map(|pair| u64::from_le_bytes(pair.1.try_into().unwrap()));
    delegation_id
}

// PROXY_DELEGATIONS_STORE_KEY
pub fn store_set_delegation(
    storage: &mut dyn Storage,
    delegation_id: &u64,
    delegation: &ProxyDelegation,
) {
    let mut store = PrefixedStorage::new(storage, PROXY_DELEGATIONS_STORE_KEY);

    store.set(&delegation_id.to_le_bytes(), &to_vec(delegation).unwrap());
}

pub fn store_get_delegation(storage: &dyn Storage, delegation_id: &u64) -> Option<ProxyDelegation> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXY_DELEGATIONS_STORE_KEY);

    store
        .get(&delegation_id.to_le_bytes())
        .map(|data| from_slice(&data).unwrap())
}

pub fn store_remove_delegation(storage: &mut dyn Storage, delegation_id: &u64) {
    let mut store = PrefixedStorage::new(storage, PROXY_DELEGATIONS_STORE_KEY);

    store.remove(&delegation_id.to_le_bytes());
}

// PER_PROXY_DELEGATIONS_STORE
pub fn store_add_per_proxy_delegation(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    delegation_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PER_PROXY_DELEGATIONS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    // Any value in store means true - &[1]
    store.set(&delegation_id.to_le_bytes(), &[1]);
}

pub fn store_remove_per_proxy_delegation(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    delegation_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PER_PROXY_DELEGATIONS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store.remove(&delegation_id.to_le_bytes());
}

pub fn store_is_proxy_delegation(
    storage: &dyn Storage,
    proxy_pubkey: &str,
    delegation_id: &u64,
) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PER_PROXY_DELEGATIONS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store.get(&delegation_id.to_le_bytes()).is_some()
}

pub fn store_get_all_proxy_delegations(storage: &dyn Storage, proxy_pubkey: &str) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PER_PROXY_DELEGATIONS_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    let mut deserialized_keys: Vec<u64> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to to_vec
        deserialized_keys.push(u64::from_le_bytes(pair.0.try_into().unwrap()));
    }

    deserialized_keys
}

// DELEGATION_INFO_STORE_KEY
pub fn set_delegation_info(
    storage: &mut dyn Storage,
    delegation_id: &u64,
    delegation: &DelegationInfo,
) {
    let mut store = PrefixedStorage::new(storage, DELEGATION_INFO_STORE_KEY);

    store.set(&delegation_id.to_le_bytes(), &to_vec(delegation).unwrap());
}

pub fn get_delegation_info(storage: &dyn Storage, delegation_id: &u64) -> Option<DelegationInfo> {
    let store = ReadonlyPrefixedStorage::new(storage, DELEGATION_INFO_STORE_KEY);

    store
        .get(&delegation_id.to_le_bytes())
        .map(|data| from_slice(&data).unwrap())
}

// High level methods

pub fn get_delegation_state(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
) -> DelegationState {
    let staking_config = store_get_staking_config(storage).unwrap();

    if let Some(delegation_id) = get_delegation_id(storage, delegator_pubkey, delegatee_pubkey) {
        let n_available_proxies = get_n_available_proxies_from_delegation(
            storage,
            delegator_pubkey,
            delegatee_pubkey,
            &staking_config.per_task_slash_stake_amount.u128(),
        );
        if n_available_proxies < get_n_minimum_proxies_for_reencryption(storage, &delegation_id) {
            DelegationState::ProxiesAreBusy
        } else {
            DelegationState::Active
        }
    } else {
        DelegationState::NonExisting
    }
}

pub fn remove_proxy_from_delegations(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
) -> StdResult<()> {
    // Delete all proxy delegations -- Make proxy inactive / stop requests factory
    for delegation_id in store_get_all_proxy_delegations(storage, proxy_pubkey) {
        let delegation = store_get_delegation(storage, &delegation_id).unwrap();

        // Remove itself from delegation
        store_remove_delegation(storage, &delegation_id);
        store_remove_proxy_delegation_id(
            storage,
            &delegation.delegator_pubkey,
            &delegation.delegatee_pubkey,
            proxy_pubkey,
        );
        store_remove_per_proxy_delegation(storage, proxy_pubkey, &delegation_id);

        // Check if delegation has enough proxies
        let all_delegation_proxies = store_get_all_proxies_from_delegation(
            storage,
            &delegation.delegator_pubkey,
            &delegation.delegatee_pubkey,
        );

        let n_minimum_proxies = get_n_minimum_proxies_for_reencryption(storage, &delegation_id);

        // Delete entire delegation = delete each proxy delegation in delegation if there is less than minimum proxies
        if all_delegation_proxies.len() < n_minimum_proxies as usize {
            for i_proxy_pubkey in all_delegation_proxies {
                let i_delegation_id = store_get_proxy_delegation_id(
                    storage,
                    &delegation.delegator_pubkey,
                    &delegation.delegatee_pubkey,
                    &i_proxy_pubkey,
                )
                .unwrap();

                store_remove_delegation(storage, &i_delegation_id);
                store_remove_proxy_delegation_id(
                    storage,
                    &delegation.delegator_pubkey,
                    &delegation.delegatee_pubkey,
                    &i_proxy_pubkey,
                );
                store_remove_per_proxy_delegation(storage, &i_proxy_pubkey, &i_delegation_id);
            }
        }
    }
    Ok(())
}

pub fn get_n_available_proxies_from_delegation(
    storage: &dyn Storage,
    delegator_pubkey: &str,
    delegatee_pubkey: &str,
    proxy_slashed_amount: &u128,
) -> u32 {
    // Return number of proxies from delegation with enough stake to get re-encryption request

    // Get all proxy delegations
    let delegation_proxies =
        store_get_all_proxies_from_delegation(storage, delegator_pubkey, delegatee_pubkey);

    let mut n_available_proxies: u32 = 0;
    for proxy_pubkey in delegation_proxies {
        // Check if each proxy in delegation has enough stake
        let proxy_addr = store_get_proxy_address(storage, &proxy_pubkey).unwrap();
        let proxy = store_get_proxy_entry(storage, &proxy_addr).unwrap();
        if &proxy.stake_amount.u128() >= proxy_slashed_amount {
            n_available_proxies += 1;
        }
    }
    n_available_proxies
}

/*
pub fn get_n_minimum_proxies_for_refund(state: &State, staking_config: &StakingConfig) -> u32 {
    // n_minimum_proxies = (threshold-1) + ceil((reward_amount*(threshold-1))/slash_amount)

    // Prevent zero division
    if staking_config.per_task_slash_stake_amount.u128() == 0 {
        return state.threshold_percentage;
    }

    // Maximum number of proxies that can finish job when re-encryption can still fail
    let fail_threshold: u32 = state.threshold_percentage - 1;

    // Worst case scenario of refunding
    let maximum_amount_to_refund: u128 =
        staking_config.per_proxy_task_reward_amount.u128() * fail_threshold as u128;

    // Number of extra proxies needed to refund
    // n_extra_proxies = CEIL(maximum_amount_to_refund/per_task_slash_stake_amount)
    let mut n_extra_proxies =
        maximum_amount_to_refund / staking_config.per_task_slash_stake_amount.u128();

    // Ceiling division
    if maximum_amount_to_refund % staking_config.per_task_slash_stake_amount.u128() != 0 {
        n_extra_proxies += 1;
    }

    // Limit minimum to threshold
    std::cmp::max(fail_threshold + n_extra_proxies as u32, state.threshold_percentage)
}
*/

pub fn get_n_minimum_proxies_for_reencryption(storage: &dyn Storage, delegation_id: &u64) -> u32 {
    get_delegation_info(storage, delegation_id)
        .unwrap()
        .minimum_proxies_for_reencryption
}

pub fn get_maximum_security_percentage(
    staking_config: &StakingConfig,
    proxies_num_maybe: Option<u32>,
) -> StdResult<u8> {
    // maximum_security_factor = [slash_amount / (slash_amount + reward_amount)] + [1 / proxies_num]
    // maximum_security_percentage = maximum_security_factor * 100

    let u100 = Uint128::from(100u8);
    let slash_amount = &staking_config.per_task_slash_stake_amount;
    let reward_amount = &staking_config.per_proxy_task_reward_amount;
    let mut percentage = slash_amount
        .checked_mul(u100)?
        .checked_div(slash_amount.checked_add(*reward_amount)?)?;

    if let Some(proxies_num) = proxies_num_maybe {
        percentage = percentage.checked_add(u100.checked_div(Uint128::from(proxies_num))?)?;
    }

    percentage = std::cmp::min(percentage, u100);

    Ok(u8::try_from(percentage.u128()).unwrap())
}
