use crate::types::{StreamDataSelection, StreamInfo, Uri};
use cosmwasm_std::{Env, Order, StdError, StdResult, Storage, TransactionInfo, Uint128};
use cw_storage_plus::{Bound, Item, Key, Map, PrimaryKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct State {
    pub stream_next_id: Uint128,
}

#[derive(Clone, Debug)]
struct StreamDataKey {
    stream: u128,
    height: u64,
    index: u32,
}

impl StreamDataKey {
    fn new(stream_id: Uint128, env: &Env) -> StreamDataKey {
        StreamDataKey {
            stream: stream_id.u128(),
            height: env.block.height,
            index: env
                .transaction
                .as_ref()
                .unwrap_or(&TransactionInfo { index: 0 }) // assumes maximum of 1 call outside of a tx
                .index, // assumes maximum of one insertion per tx
        }
    }
}

impl<'a> PrimaryKey<'a> for &StreamDataKey {
    type Prefix = u128;
    type SubPrefix = ();
    type Suffix = (u64, u32);
    type SuperSuffix = (u128, u64, u32);

    /// returns a slice of key steps, which can be optionally combined
    fn key(&self) -> Vec<Key> {
        vec![
            Key::Val128(self.stream.to_be_bytes()),
            Key::Val64(self.height.to_be_bytes()),
            Key::Val32(self.index.to_be_bytes()),
        ]
    }
}

// Singleton
const STATE: Item<State> = Item::new("streaming:contract-state");

// Maps
const STREAMS_INFO: Map<u128, StreamInfo> = Map::new("streaming:streams-info");
const STREAMS_DATA: Map<&StreamDataKey, Uri> = Map::new("streaming:streams-data");

// Shared state
pub fn new_stream_id(storage: &mut dyn Storage) -> StdResult<Uint128> {
    let mut state = STATE.load(storage)?;
    let id = state.stream_next_id;
    state.stream_next_id += Uint128::from(1u128);
    STATE.save(storage, &state)?;
    Ok(id)
}

pub fn save_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

// Streams info & data
pub fn add_stream(storage: &mut dyn Storage, stream: &StreamInfo) -> StdResult<()> {
    STREAMS_INFO.save(storage, stream.id.u128(), stream)
}

pub fn close_stream(storage: &mut dyn Storage, stream_id: Uint128) -> StdResult<()> {
    STREAMS_INFO.remove(storage, stream_id.u128());
    Ok(())
}

/// update_stream add a data element to a stream making it its latest
/// N.B. Can only add one element per transaction
pub fn update_stream(
    storage: &mut dyn Storage,
    env: &Env,
    stream_id: Uint128,
    data_uri: &Uri,
) -> StdResult<()> {
    let mut info = STREAMS_INFO
        .may_load(storage, stream_id.u128())?
        .ok_or_else(|| StdError::generic_err("Stream info lookup failed"))?;

    info.latest_data_uri = Some(data_uri.clone());
    info.length += Uint128::from(1u128);
    STREAMS_INFO.save(storage, stream_id.u128(), &info)?;

    STREAMS_DATA.save(storage, &StreamDataKey::new(stream_id, env), data_uri)
}

pub fn get_stream_info(storage: &dyn Storage, id: Uint128) -> StdResult<StreamInfo> {
    STREAMS_INFO
        .may_load(storage, id.u128())?
        .ok_or_else(|| StdError::generic_err("Stream info lookup failed"))
}

pub fn get_stream_data(
    storage: &dyn Storage,
    id: Uint128,
    maybe_height: Option<u64>,
    maybe_count: Option<Uint128>,
) -> StdResult<StreamDataSelection> {
    let total_size = get_stream_info(storage, id)?.length;
    let height = maybe_height.unwrap_or(u64::MAX);
    let count = maybe_count.unwrap_or(
        total_size + Uint128::from(1u128), // +1 to detect that there is no more data to return
    );
    let (mut oldest_height, mut oldest_index) = (0, 0); // stores last element composite key
    let mut data = STREAMS_DATA
        .prefix(id.u128())
        .range(
            storage,
            None,
            Some(Bound::exclusive((height, 0))), // all blocks with 'height' or newer will be excluded
            Order::Descending,                   // from newest to oldest
        )
        .take(count.u128() as usize)
        .map(|e| {
            let kv = e.unwrap();
            (oldest_height, oldest_index) = kv.0;
            kv.1
        })
        .collect::<Vec<Uri>>(); // contains at most a count elements of uris stored before given block 'height'

    // collect all elements added within oldest block height
    STREAMS_DATA
        .prefix(id.u128())
        .range(
            storage,
            Some(Bound::inclusive((oldest_height, 0))),
            Some(Bound::exclusive((oldest_height, oldest_index))),
            Order::Descending,
        )
        .for_each(|e| {
            data.push(e.unwrap().1);
        });

    // pagination and break condition
    let next_height = if data.len() < count.u128() as usize {
        // no more items older than oldest_height
        None
    } else {
        // next query look for items strictly older than oldest_height
        Some(oldest_height)
    };

    Ok(StreamDataSelection {
        id,
        total_length: total_size,
        next_height,
        data_uris: data,
    })
}
