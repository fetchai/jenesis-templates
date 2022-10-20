use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, SubMsgResponse, Timestamp, Uint128, WasmMsg,
};

use crate::msg::{
    ExecuteMsg, InstantiateMsg, OracleContractResponse, OracleMsg, OracleValueResponse, QueryMsg,
};
use crate::state::{OracleValue, State, ORACLE_VALUE, STATE};

const ORACLE_REPLY_ID: u64 = 1u64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let oracle_value = OracleValue {
        value: Uint128::from(0u128),
        decimals: Uint128::from(0u128),
        timestamp: Timestamp::from_seconds(0),
    };

    let state = State {
        oracle_contract_address: msg.oracle_contract_address,
        owner: info.sender,
    };
    STATE.save(deps.storage, &state)?;
    ORACLE_VALUE.save(deps.storage, &oracle_value)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::QueryOracleValue {} => try_query_oracle_value(deps, &env, &info),
    }
}

// Get the oracle value stored on the oracle contract
fn try_query_oracle_value(deps: DepsMut, env: &Env, info: &MessageInfo) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;

    let msg = to_binary(&OracleMsg::QueryOracleValue {
        address: env.contract.address.clone(),
    })?;

    let message: CosmosMsg = WasmMsg::Execute {
        contract_addr: state.oracle_contract_address.to_string(),
        msg,
        funds: info.funds.clone(),
    }
    .into();

    let submessage = SubMsg::reply_on_success(message, ORACLE_REPLY_ID);

    Ok(Response::new()
        .add_submessage(submessage)
        .add_attribute("action", "query_oracle_value"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::OracleContract {} => {
            let state = STATE.load(deps.storage)?;
            let out = to_binary(&OracleContractResponse {
                address: state.oracle_contract_address,
            })?;
            Ok(out)
        }
        QueryMsg::OracleValue {} => {
            let oracle_value = ORACLE_VALUE.load(deps.storage)?;
            let out = to_binary(&OracleValueResponse { oracle_value })?;
            Ok(out)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
    match reply.id {
        ORACLE_REPLY_ID => handle_oracle_reply(deps, reply),
        id => Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}

pub fn unwrap_reply(reply: Reply) -> StdResult<SubMsgResponse> {
    reply.result.into_result().map_err(StdError::generic_err)
}

pub fn event_contains_attr(event: &Event, key: &str) -> bool {
    event.attributes.iter().any(|attr| attr.key == key)
}

fn handle_oracle_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    let sub_response = unwrap_reply(msg)?;

    // SubMsgResponse
    // https://docs.rs/cosmwasm-std/latest/cosmwasm_std/struct.SubMsgResponse.html

    let event = sub_response
        .events
        .iter()
        .find(|event| event_contains_attr(event, "oracle-value"))
        .ok_or_else(|| StdError::generic_err("cannot find oracle reply event"))?;

    let value = event
        .attributes
        .iter()
        .find(|attr| attr.key == "oracle-value")
        .cloned()
        .ok_or_else(|| StdError::generic_err("cannot find `oracle-value` attribute"))?
        .value;

    let decimals = event
        .attributes
        .iter()
        .find(|attr| attr.key == "decimals")
        .cloned()
        .ok_or_else(|| StdError::generic_err("cannot find `decimals` attribute"))?
        .value;

    let timestamp = event
        .attributes
        .iter()
        .find(|attr| attr.key == "timestamp")
        .cloned()
        .ok_or_else(|| StdError::generic_err("cannot find `timestamp` attribute"))?
        .value;

    ORACLE_VALUE.update(
        deps.storage,
        |mut oracle_value: OracleValue| -> Result<_, StdError> {
            oracle_value.value = Uint128::from(value.parse::<u128>().unwrap());
            oracle_value.decimals = Uint128::from(decimals.parse::<u128>().unwrap());
            oracle_value.timestamp = Timestamp::from_seconds(timestamp.parse::<u64>().unwrap());
            Ok(oracle_value)
        },
    )?;

    Ok(Response::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Addr};

    use crate::msg::InstantiateMsg;

    pub const TEST_ORACLE_ADDRESS: &str = "fetch1egrrjxyt0506aq2r6jh7nldd6hw73a55pg0smj";

    fn init_msg(oracle_contract_address: Addr) -> InstantiateMsg {
        InstantiateMsg {
            oracle_contract_address,
        }
    }

    #[test]
    fn test_proper_initialization() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let addr = Addr::unchecked(TEST_ORACLE_ADDRESS);
        let msg = init_msg(addr);
        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
