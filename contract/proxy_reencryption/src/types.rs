use cosmwasm_std::{
    StdResult,
};

use cosmwasm_std::{Storage, Order, from_slice};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use serde::{de, Serialize};

pub fn set_bool_store_multilevel(store: &mut dyn Storage, namespaces: &[&[u8]], data: &[u8], value: bool) -> StdResult<()> {
    let mut store: Bucket<bool> = Bucket::multilevel(store, namespaces);

    match value
    {
        true => store.save(data, &value)?,
        false => store.remove(data)
    }
    Ok(())
}
pub fn set_bool_store(store: &mut dyn Storage, namespace: &[u8], data: &[u8], value: bool) -> StdResult<()>
{
    return set_bool_store_multilevel(store,&[namespace], &data, value);
}

pub fn get_bool_store_multilevel(store: &dyn Storage, namespaces: &[&[u8]], data: &[u8]) -> StdResult<bool> {
    let store: ReadonlyBucket<bool> = ReadonlyBucket::multilevel(store, namespaces);

    // Return false if not present in store
    Ok(store.may_load(data)?.unwrap_or(false))
}
pub fn get_bool_store(store: &dyn Storage, namespace: &[u8], data: &[u8]) -> StdResult<bool>
{
    return get_bool_store_multilevel(store, &[namespace], &data);
}

pub fn get_all_keys_multilevel<K: de::DeserializeOwned, V: de::DeserializeOwned + Serialize>(store: &dyn Storage, namespaces: &[&[u8]]) -> StdResult<Vec<K>>
{
    let bucket: ReadonlyBucket<V> = ReadonlyBucket::multilevel(store, namespaces);

    let mut deserialized_keys: Vec<K> = Vec::new();

    for pair in bucket.range(None, None, Order::Ascending)
    {
        // Deserialize keys with inverse operation to &to_vec(K)?
        deserialized_keys.push( from_slice(&pair?.0)?);
    }

    return Ok(deserialized_keys);
}

pub fn get_all_keys<K: de::DeserializeOwned, V: de::DeserializeOwned + Serialize>(store: &dyn Storage, namespace: &[u8]) -> StdResult<Vec<K>>
{
    get_all_keys_multilevel::<K,V>(store,&[namespace])
}




pub fn get_all_values_multilevel<V: de::DeserializeOwned + Serialize>(store: &dyn Storage, namespaces: &[&[u8]]) -> StdResult<Vec<V>>
{
    let bucket: ReadonlyBucket<V> = ReadonlyBucket::multilevel(store, namespaces);

    let mut deserialized_values: Vec<V> = Vec::new();

    for pair in bucket.range(None, None, Order::Ascending)
    {
        // Deserialize keys with inverse operation to &to_vec(K)?
        deserialized_values.push( pair?.1);
    }

    return Ok(deserialized_values);
}

pub fn get_all_values<V: de::DeserializeOwned + Serialize>(store: &dyn Storage, namespace: &[u8]) -> StdResult<Vec<V>>
{
    get_all_values_multilevel::<V>(store,&[namespace])
}

