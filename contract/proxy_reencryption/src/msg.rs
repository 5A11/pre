use crate::delegations::DelegationState;
use crate::reencryption_requests::ReencryptionRequestState;
use crate::state::DataEntry;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyDelegation {
    pub proxy_pubkey: String,
    pub delegation_string: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyTask {
    pub data_id: String,
    pub delegatee_pubkey: String,
    pub delegator_pubkey: String,
    pub delegation_string: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
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
    AddProxy {
        proxy_addr: Addr,
    },
    RemoveProxy {
        proxy_addr: Addr,
    },

    // Proxy actions
    RegisterProxy {
        proxy_pubkey: String,
    },
    UnregisterProxy {},
    ProvideReencryptedFragment {
        data_id: String,
        delegatee_pubkey: String,
        fragment: String,
    },

    DeactivateProxy {}, // Switch to leaving state

    // Delegator actions
    AddData {
        data_id: String,
        delegator_pubkey: String,
    },
    RequestProxiesForDelegation {
        delegator_pubkey: String,
        delegatee_pubkey: String,
    },
    AddDelegation {
        delegator_pubkey: String,
        delegatee_pubkey: String,
        proxy_delegations: Vec<ProxyDelegation>,
    },
    RequestReencryption {
        data_id: String,
        delegatee_pubkey: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetAvailableProxies {},
    GetDataID {
        data_id: String,
    },
    GetFragments {
        data_id: String,
        delegatee_pubkey: String,
    },
    GetThreshold {},
    GetNextProxyTask {
        proxy_pubkey: String,
    },
    GetDelegationState {
        delegator_pubkey: String,
        delegatee_pubkey: String,
    },
    GetSelectedProxiesForDelegation {
        delegator_pubkey: String,
        delegatee_pubkey: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetAvailableProxiesResponse {
    pub proxy_pubkeys: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetDataIDResponse {
    pub data_entry: Option<DataEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetFragmentsResponse {
    pub reencryption_request_state: ReencryptionRequestState,
    pub fragments: Vec<String>,
    pub threshold: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetThresholdResponse {
    pub threshold: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetNextProxyTaskResponse {
    pub proxy_task: Option<ProxyTask>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetDelegationStateRepsonse {
    pub delegation_state: DelegationState,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetSelectedProxiesForDelegationResponse {
    pub proxy_pubkeys: Vec<String>,
}
