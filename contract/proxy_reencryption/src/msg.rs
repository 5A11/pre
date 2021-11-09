use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Binary};
use crate::state::{DataEntry, ProxyTask, DataId};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyDelegation {
    pub proxy_addr: Addr,
    pub delegation_string: Binary,
}


#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg
{
    pub threshold: Option<u32>,
    pub admin: Option<Addr>,
    // Maximum proxies you can select for delegation = Number of active proxies if None
    pub n_max_proxies: Option<u32>,
    pub proxies: Option<Vec<Addr>>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Admin actions
    AddProxy
    {
        proxy_addr: Addr,
    },
    RemoveProxy
    {
        proxy_addr: Addr,
    },

    // Proxy actions
    RegisterProxy {},
    UnregisterProxy {},
    ProvideReencryptedFragment
    {
        data_id: DataId,
        delegatee_pubkey: Binary,
        fragment: DataId,
    },

    // Delegator actions
    AddData
    {
        data_id: DataId,
        delegator_pubkey: Binary,
    },
    AddDelegation
    {
        delegatee_pubkey: Binary,
        proxy_delegations: Vec<ProxyDelegation>,
    },
    RequestReencryption
    {
        data_id: DataId,
        delegatee_pubkey: Binary,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetAvailableProxies {},
    GetDataID { data_id: DataId },
    GetFragments { data_id: DataId, delegatee_pubkey: Binary },
    GetThreshold {},
    GetNextProxyTask { proxy_addr: Addr },
    GetDoesDelegationExist { delegator_addr: Addr, delegatee_pubkey: Binary },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetAvailableProxiesResponse {
    pub proxies: Vec<Addr>,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetDataIDResponse {
    pub data_entry: Option<DataEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetFragmentsResponse {
    pub fragments: Vec<DataId>,
    pub threshold: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetThresholdResponse {
    pub threshold: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetNextProxyTaskResponse {
    pub proxy_task: Option<ProxyTask>
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetDoesDelegationExistRepsonse {
    pub delegation_exists: bool
}
