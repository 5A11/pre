use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Binary};
use crate::state::{DataEntry, ProxyTask, HashID};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyDelegation {
    pub proxy_pubkey: Binary,
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
    RegisterProxy
    {
    proxy_pubkey: Binary
    },
    UnregisterProxy {},
    ProvideReencryptedFragment
    {
        data_id: HashID,
        delegatee_pubkey: Binary,
        fragment: HashID,
    },

    // Delegator actions
    AddData
    {
        data_id: HashID,
        delegator_pubkey: Binary,
    },
    AddDelegation
    {
        delegator_pubkey: Binary,
        delegatee_pubkey: Binary,
        proxy_delegations: Vec<ProxyDelegation>,
    },
    RequestReencryption
    {
        data_id: HashID,
        delegatee_pubkey: Binary,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetAvailableProxies {},
    GetDataID { data_id: HashID },
    GetFragments { data_id: HashID, delegatee_pubkey: Binary },
    GetThreshold {},
    GetNextProxyTask { proxy_pubkey: Binary },
    GetDoesDelegationExist { delegator_addr: Addr, delegator_pubkey: Binary, delegatee_pubkey: Binary },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetAvailableProxiesResponse {
    pub proxy_pubkeys: Vec<Binary>,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetDataIDResponse {
    pub data_entry: Option<DataEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetFragmentsResponse {
    pub fragments: Vec<HashID>,
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
