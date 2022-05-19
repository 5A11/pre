use cosmwasm_std::{from_slice, to_vec, Addr, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// To be able to tell that delegator allowed delegatee to request access
// Map delegator_address: Addr -> delegatee_pubkey: String -> permission: ReencryptionPermission
static REENCRYPTION_PERMISSION_STORE_KEY: &[u8] = b"ReencryptionPermissionStore";

// This will represent data entry permission filter
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ReencryptionPermission {
    ids: HashSet<String>,
}

impl ReencryptionPermission {
    fn new() -> ReencryptionPermission {
        ReencryptionPermission {
            ids: HashSet::new(),
        }
    }
}

pub fn store_get_permission(
    storage: &dyn Storage,
    delegator_addr: &Addr,
    delegatee_pubkey: &str,
) -> Option<ReencryptionPermission> {
    let store = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[REENCRYPTION_PERMISSION_STORE_KEY, delegator_addr.as_bytes()],
    );

    store
        .get(delegatee_pubkey.as_bytes())
        .map(|data| from_slice(&data).unwrap())
}

pub fn store_set_permission(
    storage: &mut dyn Storage,
    delegator_addr: &Addr,
    delegatee_pubkey: &str,
    permission: &ReencryptionPermission,
) {
    let mut store = PrefixedStorage::multilevel(
        storage,
        &[REENCRYPTION_PERMISSION_STORE_KEY, delegator_addr.as_bytes()],
    );

    store.set(delegatee_pubkey.as_bytes(), &to_vec(permission).unwrap());
}

pub fn set_permission(
    storage: &mut dyn Storage,
    delegator_addr: &Addr,
    delegatee_pubkey: &str,
    data_id: &str,
    permitted: bool,
) {
    let mut permissions: ReencryptionPermission =
        match store_get_permission(storage, delegator_addr, delegatee_pubkey) {
            None => {
                if permitted {
                    ReencryptionPermission::new()
                } else {
                    return;
                }
            }
            Some(permission) => permission,
        };

    if permitted {
        permissions.ids.insert(data_id.to_string());
    } else {
        permissions.ids.remove(&data_id.to_string());
    }

    store_set_permission(storage, delegator_addr, delegatee_pubkey, &permissions);
}

pub fn get_permission(
    storage: &dyn Storage,
    delegator_addr: &Addr,
    delegatee_pubkey: &str,
    data_id: &str,
) -> bool {
    // Placeholder for whitelisting access control

    let permissions: ReencryptionPermission =
        match store_get_permission(storage, delegator_addr, delegatee_pubkey) {
            None => {
                return false;
            }
            Some(permissions) => permissions,
        };

    permissions.ids.contains(data_id)
}
