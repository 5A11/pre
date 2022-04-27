use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type Uri = String; // data uri string
pub type MetaData = Vec<(String, String)>; // key-value list

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct StreamInfo {
    pub id: Uint128,
    pub owner: Addr,
    pub length: Uint128,
    pub latest_data_uri: Option<Uri>,
    pub metadata: Option<MetaData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct StreamDataSelection {
    pub id: Uint128,
    pub total_length: Uint128,
    pub next_height: Option<u64>,
    pub data_uris: Vec<Uri>,
}
