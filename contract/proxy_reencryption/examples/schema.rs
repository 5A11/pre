use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cw_proxy_reencryption::msg::{
    ExecuteMsg, GetAvailableProxiesResponse, GetContractStateResponse, GetDataIDResponse,
    GetDelegationStatusResponse, GetFragmentsResponse, GetNextProxyTaskResponse,
    GetSelectedProxiesForDelegationResponse, InstantiateMsg, QueryMsg,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(GetAvailableProxiesResponse), &out_dir);
    export_schema(&schema_for!(GetDataIDResponse), &out_dir);
    export_schema(&schema_for!(GetFragmentsResponse), &out_dir);
    export_schema(&schema_for!(GetContractStateResponse), &out_dir);
    export_schema(&schema_for!(GetNextProxyTaskResponse), &out_dir);
    export_schema(&schema_for!(GetDelegationStatusResponse), &out_dir);
    export_schema(
        &schema_for!(GetSelectedProxiesForDelegationResponse),
        &out_dir,
    );
}
