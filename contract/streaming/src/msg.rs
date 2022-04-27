use crate::types::{MetaData, StreamDataSelection, StreamInfo, Uri};
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateStream { metadata: Option<MetaData> },
    UpdateStream { id: Uint128, data_uri: Uri },
    CloseStream { id: Uint128 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetStreamInfo {
        id: Uint128,
    },
    GetStreamData {
        id: Uint128,
        height: Option<u64>,
        count: Option<Uint128>,
    },
}

pub type GetStreamInfoResponse = StreamInfo;

pub type GetStreamDataResponse = StreamDataSelection;
