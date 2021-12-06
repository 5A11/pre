use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cw_proxy_reencryption::msg::{
    ExecuteMsg, GetAvailableProxiesResponse, GetDataIDResponse, GetFragmentsResponse,
    GetNextProxyTaskResponse, GetThresholdResponse, InstantiateMsg, QueryMsg,
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
    export_schema(&schema_for!(GetThresholdResponse), &out_dir);
    export_schema(&schema_for!(GetNextProxyTaskResponse), &out_dir);
}
