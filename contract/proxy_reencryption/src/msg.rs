use crate::delegations::DelegationState;
use crate::proxies::ProxyState;
use crate::reencryption_requests::ReencryptionRequestState;
use crate::state::DataEntry;
use cosmwasm_std::{Addr, Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyDelegationString {
    pub proxy_addr: Addr,
    pub delegation_string: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyTaskResponse {
    pub data_id: String,
    pub capsule: String,
    pub delegatee_pubkey: String,
    pub delegator_pubkey: String,
    pub delegation_string: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyAvailabilityResponse {
    pub proxy_addr: String,
    pub proxy_pubkey: String,
    pub stake_amount: Uint128,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyStatusResponse {
    pub proxy_addr: Addr,
    pub stake_amount: Uint128,
    pub withdrawable_stake_amount: Uint128,
    pub proxy_state: ProxyState,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub threshold: Option<u32>,
    pub admin: Option<Addr>,

    pub proxy_whitelisting: Option<bool>,
    pub proxies: Option<Vec<Addr>>,

    // Staking
    pub stake_denom: String,
    pub minimum_proxy_stake_amount: Option<Uint128>,
    pub per_proxy_task_reward_amount: Option<Uint128>,
    pub per_task_slash_stake_amount: Option<Uint128>,

    // Timeouts
    pub timeout_height: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Tag {
    pub key: String,
    pub value: String,
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
    TerminateContract {},
    WithdrawContract {
        recipient_addr: Addr,
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
    SkipReencryptionTask {
        data_id: String,
        delegatee_pubkey: String,
    },

    DeactivateProxy {},
    // Switch to leaving state
    WithdrawStake {
        stake_amount: Option<Uint128>,
    },
    AddStake {},

    // Delegator actions
    AddData {
        data_id: String,
        delegator_pubkey: String,
        capsule: String, // symmetric key encoded with data owner public key (only data owner can decode this)
        tags: Option<Vec<Tag>>,
    },
    // Remove data, reencryption request and fragments
    RemoveData {
        data_id: String,
    },
    AddDelegation {
        delegator_pubkey: String,
        delegatee_pubkey: String,
        proxy_delegations: Vec<ProxyDelegationString>,
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
    GetContractState {},
    GetStakingConfig {},

    GetProxyTasks {
        proxy_addr: Addr,
    },
    GetDelegationStatus {
        delegator_pubkey: String,
        delegatee_pubkey: String,
    },
    GetProxyStatus {
        proxy_addr: Addr,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetAvailableProxiesResponse {
    pub proxies: Vec<ProxyAvailabilityResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetDataIDResponse {
    pub data_entry: Option<DataEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetFragmentsResponse {
    pub reencryption_request_state: ReencryptionRequestState,
    pub capsule: String,
    pub fragments: Vec<String>,
    pub threshold: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetContractStateResponse {
    pub admin: Addr,
    pub threshold: u32,
    pub terminated: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetStakingConfigResponse {
    pub stake_denom: String,
    pub minimum_proxy_stake_amount: Uint128,
    pub per_proxy_task_reward_amount: Uint128,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetProxyTasksResponse {
    pub proxy_tasks: Vec<ProxyTaskResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetDelegationStatusResponse {
    pub delegation_state: DelegationState,
    pub total_request_reward_amount: Coin,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetProxyStatusResponse {
    pub proxy_status: Option<ProxyStatusResponse>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsgResponse {
    pub threshold: u32,
    pub admin: Addr,

    pub proxy_whitelisting: bool,
    pub proxies: Option<Vec<Addr>>,

    // Staking
    pub stake_denom: String,
    pub minimum_proxy_stake_amount: Uint128,
    pub per_proxy_task_reward_amount: Uint128,
    pub per_task_slash_stake_amount: Uint128,

    // Timeouts
    pub timeout_height: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProxyStakeResponse {
    pub proxy_addr: Addr,
    pub stake: Uint128,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsgJSONResponse {
    RequestReencryption { proxies: Vec<ProxyStakeResponse> },
    RemoveData { proxies: Vec<ProxyStakeResponse> },
}
