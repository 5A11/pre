use cosmwasm_std::{Addr, Order, StdError, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

// Map data_id: String -> label: String -> is_label: bool
static DATA_LABELS_STORE_KEY: &[u8] = b"DataLabelsStore";

// Map delegator_address: Addr -> delegatee_pubkey: String -> label: String -> is_label: bool
static DELEGATEE_LABELS_STORE_KEY: &[u8] = b"DelegateeLabelsStore";

// DATA_LABELS_STORE
pub fn store_add_data_labels(storage: &mut dyn Storage, data_id: &str, labels: &[String]) {
    let mut store =
        PrefixedStorage::multilevel(storage, &[DATA_LABELS_STORE_KEY, data_id.as_bytes()]);

    for label in labels {
        store.set(label.as_bytes(), &[1])
    }
}

pub fn store_remove_data_labels(
    storage: &mut dyn Storage,
    data_id: &str,
    labels: &[String],
) -> StdResult<()> {
    let mut store =
        PrefixedStorage::multilevel(storage, &[DATA_LABELS_STORE_KEY, data_id.as_bytes()]);

    for label in labels {
        if store.get(label.as_bytes()).is_none() {
            return Err(StdError::generic_err(format!(
                "Non existing data label {}",
                &label
            )));
        }

        store.remove(label.as_bytes())
    }

    Ok(())
}

pub fn store_is_data_label(storage: &dyn Storage, data_id: &str, label: &str) -> bool {
    let store =
        ReadonlyPrefixedStorage::multilevel(storage, &[DATA_LABELS_STORE_KEY, data_id.as_bytes()]);

    store.get(label.as_bytes()).is_some()
}

pub fn store_get_all_data_labels(storage: &dyn Storage, data_id: &str) -> Vec<String> {
    let store =
        ReadonlyPrefixedStorage::multilevel(storage, &[DATA_LABELS_STORE_KEY, data_id.as_bytes()]);

    let mut deserialized_keys: Vec<String> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to &string.as_bytes()
        deserialized_keys.push(String::from_utf8(pair.0).unwrap());
    }

    deserialized_keys
}

// DELEGATEE_LABELS_STORE
pub fn store_add_delegatee_labels(
    storage: &mut dyn Storage,
    delegator_addr: &Addr,
    delegatee_pubkey: &str,
    labels: &[String],
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_LABELS_STORE_KEY,
            delegator_addr.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    for label in labels {
        store.set(label.as_bytes(), &[1])
    }
}

pub fn store_remove_delegatee_labels(
    storage: &mut dyn Storage,
    delegator_addr: &Addr,
    delegatee_pubkey: &str,
    labels: &[String],
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_LABELS_STORE_KEY,
            delegator_addr.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    for label in labels {
        if store.get(label.as_bytes()).is_none() {
            return Err(StdError::generic_err(format!(
                "Non existing delegatee label {}",
                &label
            )));
        }

        store.remove(label.as_bytes())
    }

    Ok(())
}

pub fn store_is_delegatee_label(
    storage: &dyn Storage,
    delegator_addr: &Addr,
    delegatee_pubkey: &str,
    label: &str,
) -> bool {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_LABELS_STORE_KEY,
            delegator_addr.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    store.get(label.as_bytes()).is_some()
}

pub fn store_get_all_delegatee_labels(
    storage: &dyn Storage,
    delegator_addr: &Addr,
    delegatee_pubkey: &str,
) -> Vec<String> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[
            DELEGATEE_LABELS_STORE_KEY,
            delegator_addr.as_bytes(),
            delegatee_pubkey.as_bytes(),
        ],
    );

    let mut deserialized_keys: Vec<String> = Vec::new();

    for pair in store.range(None, None, Order::Ascending) {
        // Deserialize keys with inverse operation to &string.as_bytes()
        deserialized_keys.push(String::from_utf8(pair.0).unwrap());
    }

    deserialized_keys
}

// Public functions

pub fn get_permission(
    storage: &dyn Storage,
    delegator_addr: &Addr,
    delegatee_pubkey: &str,
    data_id: &str,
) -> bool {
    let delegatee_labels =
        store_get_all_delegatee_labels(storage, delegator_addr, delegatee_pubkey);

    for delegatee_label in delegatee_labels {
        if store_is_data_label(storage, data_id, &delegatee_label) {
            return true;
        }
    }

    false
}
