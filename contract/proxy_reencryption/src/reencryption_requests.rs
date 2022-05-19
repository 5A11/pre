use crate::common::add_bank_msg;
use crate::state::{
    store_get_staking_config, store_get_state, store_get_timeouts_config,
    store_set_timeouts_config, StakingConfig, State, TimeoutsConfig,
};
use cosmwasm_std::{from_slice, to_vec, Addr, Order, Response, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

// Map proxy_task_id: u64 -> task: ProxyTask
static PROXY_TASKS_STORE_KEY: &[u8] = b"ProxyTasks";

// Delegatee side to lookup fragments
// Map data_id: String -> delegatee_pubkey: String -> proxy_pubkey: String -> proxy_task_id: u64
static DELEGATEE_PROXY_TASKS_STORE_KEY: &[u8] = b"DelegateeProxyTasks";

// Proxy side to lookup active tasks
// Map proxy_pubkey: String -> proxy_task_id: u64 -> is_task: bool
static PROXY_TASKS_QUEUE_STORE_KEY: &[u8] = b"ProxyTasksQueue";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyTask {
    // To find neighbouring tasks
    pub data_id: String,
    pub delegatee_pubkey: String,

    // Individual task parameters
    pub proxy_pubkey: String,
    pub fragment: Option<String>,
    pub delegation_string: String,

    // Timeouts
    pub timeout_height: u64,
    // Reward will be returned to this address when request cannot be completed
    pub refund_addr: Addr,

    // When task was finished and proxy got rewarded or it was abandoned/timed-out and delegator got refunded
    pub resolved: bool,
    // When proxy skips task or unregister - doesn't mean that delegator got refunded
    pub abandoned: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReencryptionRequestState {
    Inaccessible,
    Ready,
    Granted,
    Abandoned,
    TimedOut,
}

// PROXY_TASKS_STORE_KEY
pub fn store_set_proxy_task(
    storage: &mut dyn Storage,
    proxy_task_id: &u64,
    reencryption_task: &ProxyTask,
) {
    let mut store = PrefixedStorage::new(storage, PROXY_TASKS_STORE_KEY);

    store.set(
        &proxy_task_id.to_le_bytes(),
        &to_vec(reencryption_task).unwrap(),
    );
}

pub fn store_get_proxy_task(storage: &dyn Storage, proxy_task_id: &u64) -> Option<ProxyTask> {
    let store = ReadonlyPrefixedStorage::new(storage, PROXY_TASKS_STORE_KEY);

    store
        .get(&proxy_task_id.to_le_bytes())
        .map(|data| from_slice(&data).unwrap())
}

pub fn store_remove_proxy_task(storage: &mut dyn Storage, proxy_task_id: &u64) {
    let mut store = PrefixedStorage::new(storage, PROXY_TASKS_STORE_KEY);

    store.remove(&proxy_task_id.to_le_bytes());
}

// DELEGATEE_PROXY_TASKS_STORE_KEY
pub fn store_add_delegatee_proxy_task(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
    proxy_task_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_PROXY_TASKS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.set(proxy_pubkey.as_bytes(), &proxy_task_id.to_le_bytes());
}

pub fn store_get_delegatee_proxy_task(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) -> Option<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_PROXY_TASKS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store
        .get(proxy_pubkey.as_bytes())
        .map(|data| u64::from_le_bytes(data.try_into().unwrap()))
}

pub fn store_get_all_delegatee_proxy_tasks(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_PROXY_TASKS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    let mut deserialized_keys: Vec<u64> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to to_vec
        deserialized_keys.push(u64::from_le_bytes(pair.1.try_into().unwrap()));
    }

    deserialized_keys
}

pub fn store_is_list_of_delegatee_proxy_tasks_empty(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_PROXY_TASKS_STORE_KEY,
            data_id.as_bytes(),
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

pub fn store_remove_delegatee_proxy_task(
    storage: &mut dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
    proxy_pubkey: &str,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_PROXY_TASKS_STORE_KEY,
            data_id.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.remove(proxy_pubkey.as_bytes());
}

// PROXY_TASKS_QUEUE_STORE_KEY
pub fn store_add_proxy_task_to_queue(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    proxy_task_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PROXY_TASKS_QUEUE_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    // Any value in store means true - &[1]
    store.set(&proxy_task_id.to_le_bytes(), &[1]);
}

pub fn store_remove_proxy_task_from_queue(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    proxy_task_id: &u64,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[PROXY_TASKS_QUEUE_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store.remove(&proxy_task_id.to_le_bytes());
}

pub fn store_is_proxy_task_in_queue(
    storage: &dyn Storage,
    proxy_pubkey: &str,
    proxy_task_id: &u64,
) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PROXY_TASKS_QUEUE_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    store.get(&proxy_task_id.to_le_bytes()).is_some()
}

pub fn store_get_all_proxy_tasks_in_queue(storage: &dyn Storage, proxy_pubkey: &str) -> Vec<u64> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PROXY_TASKS_QUEUE_STORE_KEY, proxy_pubkey.as_bytes()],
    );

    let mut deserialized_keys: Vec<u64> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to to_vec
        deserialized_keys.push(u64::from_le_bytes(pair.0.try_into().unwrap()));
    }

    deserialized_keys
}

// High level methods

pub fn get_reencryption_request_state(
    storage: &dyn Storage,
    state: &State,
    data_id: &str,
    delegatee_pubkey: &str,
    block_height: &u64,
) -> ReencryptionRequestState {
    // Return state of re-encryption request by aggregating states of all individual tasks

    let proxy_tasks = store_get_all_delegatee_proxy_tasks(storage, data_id, delegatee_pubkey);

    if proxy_tasks.is_empty() {
        return ReencryptionRequestState::Inaccessible;
    }

    let mut n_provided_fragments: u32 = 0;
    let mut n_incompletable_tasks: u32 = 0;
    let mut timeout_height: u64 = 0;
    for &task_id in &proxy_tasks {
        let task: ProxyTask = store_get_proxy_task(storage, &task_id).unwrap();
        timeout_height = task.timeout_height;
        if task.fragment.is_some() {
            n_provided_fragments += 1;
        } else if task.abandoned {
            n_incompletable_tasks += 1;
        }
    }

    if n_provided_fragments >= state.threshold {
        return ReencryptionRequestState::Granted;
    }

    if block_height >= &timeout_height {
        return ReencryptionRequestState::TimedOut;
    }

    // Task cannot be completed any more
    if (proxy_tasks.len() - n_incompletable_tasks as usize) < state.threshold as usize {
        return ReencryptionRequestState::Abandoned;
    }

    ReencryptionRequestState::Ready
}

pub fn abandon_proxy_task(
    storage: &mut dyn Storage,
    re_task_id: &u64,
    state: &State,
    staking_config: &StakingConfig,
    delegator_retrieve_funds_amount: &mut HashMap<Addr, u128>,
) -> StdResult<()> {
    // Abandon individual proxy task and refunds delegator if request cannot be complete any more

    let mut re_task: ProxyTask = store_get_proxy_task(storage, re_task_id).unwrap();

    // Abandon task
    re_task.abandoned = true;
    store_set_proxy_task(storage, re_task_id, &re_task);

    // Remove task from proxy queue
    store_remove_proxy_task_from_queue(storage, &re_task.proxy_pubkey, re_task_id);

    if re_task.resolved {
        return Ok(());
    }

    // Block height is irrelevant here as we don't expect TimedOut state
    if get_reencryption_request_state(
        storage,
        state,
        &re_task.data_id,
        &re_task.delegatee_pubkey,
        &0,
    ) == ReencryptionRequestState::Abandoned
    {
        // Resolve all neighbour proxy tasks if request cannot be completed any more

        // Get all neighbouring tasks (including current one)
        let proxy_tasks = store_get_all_delegatee_proxy_tasks(
            storage,
            &re_task.data_id,
            &re_task.delegatee_pubkey,
        );
        for task_id in proxy_tasks {
            let mut task: ProxyTask = store_get_proxy_task(storage, &task_id).unwrap();

            // Skip already resolved tasks
            if task.resolved {
                continue;
            }

            task.resolved = true;
            store_set_proxy_task(storage, &task_id, &task);

            // Refund the delegator - even when is completed
            update_refunds_map(
                delegator_retrieve_funds_amount,
                &task.refund_addr,
                staking_config.per_proxy_task_reward_amount.u128(),
            );
        }
    }

    Ok(())
}

// Delete all unfinished current proxy re-encryption tasks
pub fn abandon_all_proxy_tasks(
    storage: &mut dyn Storage,
    proxy_pubkey: &str,
    response: &mut Response,
) -> StdResult<()> {
    let staking_config = store_get_staking_config(storage)?;
    let state = store_get_state(storage)?;

    let mut delegator_retrieve_funds_amount: HashMap<Addr, u128> = HashMap::new();

    for re_task_id in store_get_all_proxy_tasks_in_queue(storage, proxy_pubkey) {
        abandon_proxy_task(
            storage,
            &re_task_id,
            &state,
            &staking_config,
            &mut delegator_retrieve_funds_amount,
        )?;
    }

    // Return stake from unfinished tasks to delegators
    for (delegator_addr, stake_amount) in delegator_retrieve_funds_amount {
        add_bank_msg(
            response,
            &delegator_addr,
            stake_amount,
            &staking_config.stake_denom,
        );
    }

    Ok(())
}

pub fn timeout_proxy_task(
    storage: &mut dyn Storage,
    re_task_id: &u64,
    staking_config: &StakingConfig,
    delegator_retrieve_funds_amount: &mut HashMap<Addr, u128>,
) -> StdResult<()> {
    // Resolve proxy task, remove it from proxy que and refund the delegator

    let mut re_task: ProxyTask = store_get_proxy_task(storage, re_task_id).unwrap();

    if re_task.resolved {
        return Ok(());
    }

    // Refund the delegator - even when is completed
    update_refunds_map(
        delegator_retrieve_funds_amount,
        &re_task.refund_addr,
        staking_config.per_proxy_task_reward_amount.u128(),
    );

    // Abandon task
    re_task.abandoned = true;
    re_task.resolved = true;
    store_set_proxy_task(storage, re_task_id, &re_task);

    // Remove task from proxy queue
    store_remove_proxy_task_from_queue(storage, &re_task.proxy_pubkey, re_task_id);

    Ok(())
}

pub fn check_and_resolve_all_timedout_tasks(
    storage: &mut dyn Storage,
    response: &mut Response,
    block_height: u64,
) {
    // Check and resolve all tasks between next_task_id_to_be_checked..next_proxy_task_id
    // next_task_id_to_be_checked is moved to first task ID that is not timed-out

    let state: State = store_get_state(storage).unwrap();
    let staking_config: StakingConfig = store_get_staking_config(storage).unwrap();
    let mut timeouts_config: TimeoutsConfig = store_get_timeouts_config(storage).unwrap();

    // All existing tasks were already checked
    if timeouts_config.next_task_id_to_be_checked == state.next_proxy_task_id {
        return;
    }

    let mut delegator_retrieve_funds_amount: HashMap<Addr, u128> = HashMap::new();
    for i in timeouts_config.next_task_id_to_be_checked..state.next_proxy_task_id {
        match store_get_proxy_task(storage, &i) {
            // Skip if task was deleted
            None => {}
            Some(proxy_task) => {
                // We can move pointer when task is already completed
                if proxy_task.fragment.is_some() {
                    timeouts_config.next_task_id_to_be_checked = i + 1;
                    continue;
                }

                // If task is Active it will become new next_task_id_to_be_checked
                if block_height < proxy_task.timeout_height && !proxy_task.resolved {
                    timeouts_config.next_task_id_to_be_checked = i;
                    break;
                }

                // Resolve timed-out task
                timeout_proxy_task(
                    storage,
                    &i,
                    &staking_config,
                    &mut delegator_retrieve_funds_amount,
                )
                .unwrap();
            }
        }
        timeouts_config.next_task_id_to_be_checked = i + 1;
    }
    // If this finish without being terminated next_task_id_to_be_checked == next_proxy_task_id

    // Update next_task_id_to_be_checked
    store_set_timeouts_config(storage, &timeouts_config).unwrap();

    // Return stake from unfinished tasks to delegators
    let staking_config = store_get_staking_config(storage).unwrap();
    for (delegator_addr, stake_amount) in delegator_retrieve_funds_amount {
        add_bank_msg(
            response,
            &delegator_addr,
            stake_amount,
            &staking_config.stake_denom,
        );
    }
}

pub fn get_all_fragments(
    storage: &dyn Storage,
    data_id: &str,
    delegatee_pubkey: &str,
) -> Vec<String> {
    let mut fragments: Vec<String> = Vec::new();
    for task_id in store_get_all_delegatee_proxy_tasks(storage, data_id, delegatee_pubkey) {
        let task = store_get_proxy_task(storage, &task_id).unwrap();

        match task.fragment {
            None => continue,
            Some(fragment) => fragments.push(fragment.clone()),
        }
    }
    fragments
}

pub fn update_refunds_map(
    refund_amounts: &mut HashMap<Addr, u128>,
    refund_addr: &Addr,
    additional_stake_amount: u128,
) {
    match refund_amounts.get(refund_addr).cloned() {
        None => {
            refund_amounts.insert(refund_addr.clone(), additional_stake_amount);
        }
        Some(stake_amount) => {
            refund_amounts.insert(refund_addr.clone(), stake_amount + additional_stake_amount);
        }
    }
}
