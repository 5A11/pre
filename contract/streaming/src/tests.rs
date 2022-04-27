use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    from_binary, Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult, TransactionInfo,
    Uint128,
};
use global_counter::primitive::fast::FlushingCounterU32;

use crate::contract::{execute, instantiate, query};

use crate::msg::{
    ExecuteMsg, GetStreamDataResponse, GetStreamInfoResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{load_state, State};
use crate::types::Uri;

const DEFAULT_BLOCK_HEIGHT: u64 = 100;

static TXINDEX: FlushingCounterU32 = FlushingCounterU32::new(0);

fn mock_env_height(signer: &Addr, height: u64) -> (Env, MessageInfo) {
    let mut env = mock_env();
    env.block.height = height;
    env.transaction = Some(TransactionInfo {
        index: TXINDEX.get(),
    });
    TXINDEX.inc();
    TXINDEX.flush();

    let info = mock_info(signer.as_str(), &vec![]);

    return (env, info);
}

fn init_contract(deps: DepsMut, sender: &Addr, block_height: u64) -> StdResult<Response> {
    let env = mock_env_height(&sender, block_height);
    instantiate(deps, env.0, env.1, InstantiateMsg {})
}

fn create_stream(deps: DepsMut, sender: &Addr) -> StdResult<Response> {
    let env = mock_env_height(&sender, DEFAULT_BLOCK_HEIGHT);
    execute(
        deps,
        env.0,
        env.1,
        ExecuteMsg::CreateStream { metadata: None },
    )
}

fn update_stream_at_height(
    deps: DepsMut,
    sender: &Addr,
    id: Uint128,
    data_uri: Uri,
    height: u64,
) -> StdResult<Response> {
    let env = mock_env_height(&sender, height);
    execute(
        deps,
        env.0,
        env.1,
        ExecuteMsg::UpdateStream { id, data_uri },
    )
}

fn update_stream(deps: DepsMut, sender: &Addr, id: Uint128, data_uri: Uri) -> StdResult<Response> {
    update_stream_at_height(deps, sender, id, data_uri, DEFAULT_BLOCK_HEIGHT)
}

fn close_stream(deps: DepsMut, sender: &Addr, id: Uint128) -> StdResult<Response> {
    let env = mock_env_height(&sender, DEFAULT_BLOCK_HEIGHT);
    execute(deps, env.0, env.1, ExecuteMsg::CloseStream { id })
}

fn query_stream_info(deps: Deps, id: Uint128) -> StdResult<GetStreamInfoResponse> {
    let env = mock_env();
    from_binary(&query(deps, env, QueryMsg::GetStreamInfo { id })?)
}

fn query_stream_data(
    deps: Deps,
    id: Uint128,
    height: Option<u64>,
    count: Option<Uint128>,
) -> StdResult<GetStreamDataResponse> {
    let env = mock_env();
    from_binary(&query(
        deps,
        env,
        QueryMsg::GetStreamData { id, height, count },
    )?)
}

mod tests {
    use fetchai_std::{assert_call, assert_call_fails, cu128, error_msg, resp_msg};

    use super::*;

    #[test]
    fn test_instantiate_contract() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());

        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));

        let state: State = load_state(&deps.storage).unwrap();
        assert_eq!(state.stream_next_id, cu128!(0));
    }

    #[test]
    fn test_create_stream() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());
        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));

        // create first stream
        assert_call!(create_stream(deps.as_mut(), &sender));
        let resp = query_stream_info(deps.as_ref(), cu128!(0));
        assert_call!(resp);

        assert_eq!(
            GetStreamInfoResponse {
                owner: sender.clone(),
                id: cu128!(0),
                latest_data_uri: None,
                metadata: None,
                length: cu128!(0)
            },
            resp.unwrap()
        );

        // create a second stream
        assert_call!(create_stream(deps.as_mut(), &sender));
        let resp = query_stream_info(deps.as_ref(), cu128!(1));
        assert_call!(resp);

        assert_eq!(
            GetStreamInfoResponse {
                owner: sender,
                id: cu128!(1),
                latest_data_uri: None,
                metadata: None,
                length: cu128!(0)
            },
            resp.unwrap()
        );

        // third stream doesn't exist
        assert_call_fails!(query_stream_info(deps.as_ref(), cu128!(2)));
    }

    #[test]
    fn test_update_stream_succeeds() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());
        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));
        assert_call!(create_stream(deps.as_mut(), &sender));

        // add first data uri to stream
        assert_call!(update_stream(
            deps.as_mut(),
            &sender,
            cu128!(0),
            Uri::from("data_1")
        ));

        let resp = query_stream_info(deps.as_ref(), cu128!(0));
        assert_call!(resp);

        assert_eq!(
            GetStreamInfoResponse {
                owner: sender.clone(),
                id: cu128!(0),
                latest_data_uri: Some("data_1".to_string()),
                metadata: None,
                length: cu128!(1)
            },
            resp.unwrap()
        );

        let resp = query_stream_data(deps.as_ref(), cu128!(0), None, None);
        assert_call!(resp);

        assert_eq!(
            GetStreamDataResponse {
                id: cu128!(0),
                total_length: cu128!(1),
                next_height: None,
                data_uris: vec![Uri::from("data_1")],
            },
            resp.unwrap()
        );

        // add a second data uri to stream
        assert_call!(update_stream(
            deps.as_mut(),
            &sender,
            cu128!(0),
            Uri::from("data_2")
        ));

        let resp = query_stream_info(deps.as_ref(), cu128!(0));
        assert_call!(resp);

        assert_eq!(
            GetStreamInfoResponse {
                owner: sender,
                id: cu128!(0),
                latest_data_uri: Some("data_2".to_string()),
                metadata: None,
                length: cu128!(2)
            },
            resp.unwrap()
        );

        let resp = query_stream_data(deps.as_ref(), cu128!(0), None, None);
        assert_call!(resp);

        assert_eq!(
            GetStreamDataResponse {
                id: cu128!(0),
                total_length: cu128!(2),
                next_height: None,
                data_uris: vec![Uri::from("data_2"), Uri::from("data_1")],
            },
            resp.unwrap()
        );
    }

    #[test]
    fn test_update_stream_not_owner_fails() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());
        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));
        assert_call!(create_stream(deps.as_mut(), &sender));

        assert_call_fails!(update_stream(
            deps.as_mut(),
            &Addr::unchecked("not-owner".to_string()),
            cu128!(0),
            Uri::from("data_1")
        ));
    }

    #[test]
    fn test_update_stream_doesnt_exist_fails() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());
        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));
        assert_call!(create_stream(deps.as_mut(), &sender));

        assert_call_fails!(update_stream(
            deps.as_mut(),
            &sender,
            cu128!(1),
            Uri::from("data_1")
        ));
    }

    #[test]
    fn test_close_stream_succeeds() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());
        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));
        assert_call!(create_stream(deps.as_mut(), &sender));

        assert_call!(close_stream(deps.as_mut(), &sender, cu128!(0)));
        assert_call_fails!(close_stream(deps.as_mut(), &sender, cu128!(0)));
    }

    #[test]
    fn test_close_stream_not_owner_fails() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());
        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));
        assert_call!(create_stream(deps.as_mut(), &sender));

        assert_call_fails!(close_stream(
            deps.as_mut(),
            &Addr::unchecked("not-owner".to_string()),
            cu128!(0)
        ));
    }

    #[test]
    fn test_close_stream_doesnt_exist_fails() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());
        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));
        assert_call!(create_stream(deps.as_mut(), &sender));

        assert_call_fails!(close_stream(deps.as_mut(), &sender, cu128!(1)));
    }

    #[test]
    fn test_query_stream_data() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());
        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));
        assert_call!(create_stream(deps.as_mut(), &sender));

        let size = 10;

        let mut data_uris: Vec<Uri> = Vec::new();
        for i in 0..size {
            let uri = Uri::from(format!("data_{}", i));
            assert_call!(update_stream_at_height(
                deps.as_mut(),
                &sender,
                cu128!(0),
                uri.clone(),
                i,
            ));
            data_uris.insert(0, uri);
        }

        let mut height: Option<u64> = None;
        let mut count: Option<Uint128> = None;
        let mut next_height: Option<u64> = None;

        // query all data
        let resp = query_stream_data(deps.as_ref(), cu128!(0), height, count);
        assert_call!(resp);
        assert_eq!(
            GetStreamDataResponse {
                id: cu128!(0),
                total_length: cu128!(size),
                next_height,
                data_uris: data_uris.clone(),
            },
            resp.unwrap()
        );

        // query at a starting height
        height = Some(4);
        next_height = None;
        let index = (size - height.unwrap()) as usize;

        let resp = query_stream_data(deps.as_ref(), cu128!(0), height, count);
        assert_call!(resp);
        assert_eq!(
            GetStreamDataResponse {
                id: cu128!(0),
                total_length: cu128!(size),
                next_height,
                data_uris: data_uris[index..].to_vec(),
            },
            resp.unwrap()
        );

        // query at a given count
        height = Some(7); // heights goes from 1 to 10
        count = Some(cu128!(4));
        let index = (size - height.unwrap()) as usize;
        next_height = Some(height.unwrap() - count.unwrap().u128() as u64);

        let resp = query_stream_data(deps.as_ref(), cu128!(0), height, count);
        assert_call!(resp);
        assert_eq!(
            GetStreamDataResponse {
                id: cu128!(0),
                total_length: cu128!(size),
                next_height,
                data_uris: data_uris[index..index + count.unwrap().u128() as usize].to_vec(),
            },
            resp.unwrap()
        );
    }

    #[test]
    fn test_query_stream_data_pagination() {
        let mut deps = mock_dependencies();
        let sender = Addr::unchecked("sender".to_string());
        assert_call!(init_contract(deps.as_mut(), &sender, DEFAULT_BLOCK_HEIGHT));
        assert_call!(create_stream(deps.as_mut(), &sender));

        let size = 11;

        let mut data_uris: Vec<Uri> = Vec::new();
        for i in 0..size {
            let uri = Uri::from(format!("data_{}", i));
            assert_call!(update_stream_at_height(
                deps.as_mut(),
                &sender,
                cu128!(0),
                uri.clone(),
                i * 2 + i % 2,
            ));
            data_uris.insert(0, uri);
        }

        let mut height: u64 = size * 2 + 1;
        let count: Uint128 = cu128!(3);

        // query all data
        let mut data_uris_queried: Vec<Uri> = Vec::new();

        loop {
            let resp = query_stream_data(deps.as_ref(), cu128!(0), Some(height), Some(count));
            assert_call!(resp);

            let mut selection = resp.unwrap();

            if selection.next_height.is_none() {
                assert!(selection.data_uris.len() <= count.u128() as usize);
                data_uris_queried.append(&mut selection.data_uris);
                break;
            } else {
                assert_eq!(selection.data_uris.len(), count.u128() as usize);
                height = selection.next_height.unwrap();
                data_uris_queried.append(&mut selection.data_uris);
            }
        }

        assert_eq!(data_uris, data_uris_queried);
    }
}
