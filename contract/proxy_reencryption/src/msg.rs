use crate::delegations::DelegationState;
use crate::proxies::ProxyState;
use crate::reencryption_requests::ReencryptionRequestState;
use crate::state::DataEntry;
use cosmwasm_std::{Addr, Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyDelegationString {
    pub proxy_pubkey: String,
    pub delegation_string: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyTask {
    pub data_id: String,
    pub capsule: String,
    pub delegatee_pubkey: String,
    pub delegator_pubkey: String,
    pub delegation_string: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyAvailability {
    pub proxy_pubkey: String,
    pub stake_amount: Uint128,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ProxyStatus {
    pub proxy_address: Addr,
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
    pub per_proxy_request_reward_amount: Option<Uint128>,
    pub per_request_slash_stake_amount: Option<Uint128>,

    // Timeouts
    pub timeout_height: Option<u64>,
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
        capsule: String,
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
        proxy_pubkey: String,
    },
    GetDelegationStatus {
        delegator_pubkey: String,
        delegatee_pubkey: String,
    },
    GetProxyStatus {
        proxy_pubkey: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetAvailableProxiesResponse {
    pub proxies: Vec<ProxyAvailability>,
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
    pub per_proxy_request_reward_amount: Uint128,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetProxyTasksResponse {
    pub proxy_tasks: Vec<ProxyTask>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetDelegationStatusResponse {
    pub delegation_state: DelegationState,
    pub total_request_reward_amount: Coin,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct GetProxyStatusResponse {
    pub proxy_status: Option<ProxyStatus>,
}
