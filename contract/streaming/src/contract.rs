use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    add_stream, close_stream, get_stream_data, get_stream_info, new_stream_id, save_state,
    update_stream, State,
};
use crate::types::{MetaData, StreamInfo, Uri};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Storage, Uint128,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        stream_next_id: Uint128::zero(),
    };
    save_state(deps.storage, &state)?;

    let response = Response::new().add_attribute("indexer", "fetchai.colearn.streaming");
    Ok(response)
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::CreateStream { metadata } => try_create_stream(deps, info.sender, metadata),
        ExecuteMsg::UpdateStream { id, data_uri } => {
            try_update_stream(deps, &env, &info.sender, id, &data_uri)
        }
        ExecuteMsg::CloseStream { id } => try_close_stream(deps, &info.sender, id),
    }
}

fn try_create_stream(
    deps: DepsMut,
    sender: Addr,
    metadata: Option<MetaData>,
) -> StdResult<Response> {
    let info = StreamInfo {
        id: new_stream_id(deps.storage)?,
        owner: sender,
        metadata,
        length: Uint128::zero(),
        latest_data_uri: None,
    };

    add_stream(deps.storage, &info)?;

    let response = Response::new()
        .add_attribute("action", "create_stream")
        .add_attribute("sender", info.owner)
        .add_attribute("id", info.id.to_string());
    Ok(response)
}

fn try_update_stream(
    deps: DepsMut,
    env: &Env,
    sender: &Addr,
    id: Uint128,
    data_uri: &Uri,
) -> StdResult<Response> {
    ensure_stream_owner(deps.storage, id, sender)?;

    update_stream(deps.storage, env, id, data_uri)?;

    let response = Response::new()
        .add_attribute("action", "update_stream")
        .add_attribute("id", id.to_string())
        .add_attribute("data_uri", data_uri);
    Ok(response)
}

fn try_close_stream(deps: DepsMut, sender: &Addr, id: Uint128) -> StdResult<Response> {
    ensure_stream_owner(deps.storage, id, sender)?;

    close_stream(deps.storage, id)?;

    let response = Response::new()
        .add_attribute("action", "close_stream")
        .add_attribute("id", id.to_string());
    Ok(response)
}

fn ensure_stream_owner(storage: &dyn Storage, id: Uint128, sender: &Addr) -> StdResult<()> {
    let info = get_stream_info(storage, id)?;
    if &info.owner != sender {
        return Err(StdError::generic_err("Unauthorized"));
    }
    Ok(())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStreamInfo { id } => Ok(to_binary(&get_stream_info(deps.storage, id)?)?),
        QueryMsg::GetStreamData { id, height, count } => Ok(to_binary(&get_stream_data(
            deps.storage,
            id,
            height,
            count,
        )?)?),
    }
}
