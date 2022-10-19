use cosmwasm_std::{
    coins, entry_point, to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Timestamp, Uint128,
};

use crate::msg::{
    ExecuteMsg, FeeResponse, FeesAccruedResponse, InstantiateMsg, OracleRoleResponse, QueryMsg,
    StatusResponse,
};
use crate::state::{OracleValue, State, ORACLE_VALUE, STATE};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    if info.funds.is_empty() {
        return Err(StdError::generic_err(
            "No funds transferred - can't determine denomination.",
        ));
    }

    let oracle_value = OracleValue {
        value: Uint128::from(0u128),
        decimals: Uint128::from(0u128),
        timestamp: Timestamp::from_seconds(0),
    };

    let state = State {
        oracle: info.sender.clone(),
        owner: info.sender,
        fee: msg.fee,
        fees_accrued: Uint128::from(0u128),
        denom: info.funds[0].denom.clone(),
    };
    STATE.save(deps.storage, &state)?;
    ORACLE_VALUE.save(deps.storage, &oracle_value)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::GrantOracleRole { address } => try_grant_role(deps, &info, address),
        ExecuteMsg::UpdateOracleValue { value, decimals } => {
            try_update_oracle_value(deps, &env, &info, value, decimals)
        }
        ExecuteMsg::QueryOracleValue {} => try_query_oracle_value(deps, &info),
        ExecuteMsg::SetFee { amount } => try_set_fee(deps, &info, amount),
        ExecuteMsg::WithdrawFees { amount } => try_withdraw_fees(deps, &info, amount),
    }
}

// Set the oracle address
fn try_grant_role(deps: DepsMut, info: &MessageInfo, address: Addr) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;

    if info.sender != state.owner {
        return Err(StdError::generic_err("Not authorized to grant oracle role"));
    }

    STATE.update(deps.storage, |mut state: State| -> Result<_, StdError> {
        state.oracle = address;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("action", "grant_oracle_role"))
}

// Update the oracle value stored on contract
fn try_update_oracle_value(
    deps: DepsMut,
    env: &Env,
    _info: &MessageInfo,
    value: Uint128,
    decimals: Uint128,
) -> StdResult<Response> {
    ORACLE_VALUE.update(
        deps.storage,
        |mut oracle_value: OracleValue| -> Result<_, StdError> {
            oracle_value.value = value;
            oracle_value.decimals = decimals;
            oracle_value.timestamp = env.block.time;
            Ok(oracle_value)
        },
    )?;

    Ok(Response::new().add_attribute("action", "update_oracle_value"))
}

pub fn try_query_oracle_value(deps: DepsMut, info: &MessageInfo) -> Result<Response, StdError> {
    let state = STATE.load(deps.storage)?;

    let required_coin = Coin {
        amount: state.fee,
        denom: state.denom.clone(),
    };

    let mut sent_amount = Uint128::from(0u128);
    if required_coin.amount > Uint128::zero() {
        for coin in info.funds.iter() {
            if coin.amount >= required_coin.amount {
                sent_amount = coin.amount;
            }
        }
        if sent_amount < required_coin.amount {
            return Err(StdError::generic_err(
                "Insufficient funds sent or incorrect denomination.",
            ));
        }
    }

    let new_fees_accrued = state.fees_accrued.checked_add(sent_amount)?;
    STATE.update(deps.storage, |mut state: State| -> Result<_, StdError> {
        state.fees_accrued = new_fees_accrued;
        Ok(state)
    })?;

    let oracle_value = ORACLE_VALUE.load(deps.storage)?;

    Ok(Response::new()
        .add_attribute("oracle-value", oracle_value.value.to_string())
        .add_attribute("decimals", oracle_value.decimals)
        .add_attribute("timestamp", oracle_value.timestamp.seconds().to_string()))
}

fn try_withdraw_fees(deps: DepsMut, info: &MessageInfo, amount: Uint128) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let new_fees_accrued = state.fees_accrued.checked_sub(amount)?;
    STATE.update(deps.storage, |mut state: State| -> Result<_, StdError> {
        state.fees_accrued = new_fees_accrued;
        Ok(state)
    })?;

    let message: BankMsg = BankMsg::Send {
        to_address: state.owner.into_string(),
        amount: coins(amount.into(), state.denom),
    };

    Ok(Response::new()
        .add_message(message)
        .add_attribute("action", "withdraw_fees")
        .add_attribute("amount", amount))
}

fn try_set_fee(deps: DepsMut, info: &MessageInfo, amount: Uint128) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;

    if info.sender != state.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    STATE.update(deps.storage, |mut state: State| -> Result<_, StdError> {
        state.fee = amount;
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("action", "set_fee")
        .add_attribute("amount", amount))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    match msg {
        QueryMsg::Status {} => {
            let out = to_binary(&StatusResponse {
                status: String::from("OK"),
            })?;
            Ok(out)
        }
        QueryMsg::OracleRole {} => {
            let out = to_binary(&OracleRoleResponse {
                address: state.oracle,
            })?;
            Ok(out)
        }
        QueryMsg::Fee {} => {
            let out = to_binary(&FeeResponse { fee: state.fee })?;
            Ok(out)
        }
        QueryMsg::FeesAccrued {} => {
            let out = to_binary(&FeesAccruedResponse {
                fees_accrued: state.fees_accrued,
            })?;
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, SubMsg};

    use crate::msg::{ExecuteMsg, InstantiateMsg};

    pub const DEFAULT_ORACLE_VALUE: u128 = 0u128;
    pub const DEFAULT_DECIMALS: u128 = 0u128;
    pub const UPDATED_ORACLE_VALUE: u128 = 100000u128;
    pub const UPDATED_DECIMALS: u128 = 5u128;

    fn init_msg(fee: Uint128) -> InstantiateMsg {
        InstantiateMsg { fee }
    }

    #[test]
    fn test_proper_initialization() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = init_msg(Uint128::from(100u128));
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_update_success() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = init_msg(Uint128::from(100u128));
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let oracle_address = Addr::unchecked("oracle");
        let msg = ExecuteMsg::GrantOracleRole {
            address: oracle_address.clone(),
        };

        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let ov = OracleValue {
            value: Uint128::from(DEFAULT_ORACLE_VALUE),
            decimals: Uint128::from(DEFAULT_DECIMALS),
            timestamp: Timestamp::from_seconds(0),
        };

        let state: State = STATE.load(&deps.storage).unwrap();
        assert_eq!(oracle_address, state.oracle);

        let oracle_value: OracleValue = ORACLE_VALUE.load(&deps.storage).unwrap();
        assert_eq!(ov, oracle_value);

        let msg = ExecuteMsg::UpdateOracleValue {
            value: Uint128::from(UPDATED_ORACLE_VALUE),
            decimals: Uint128::from(UPDATED_DECIMALS),
        };

        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let ov = OracleValue {
            value: Uint128::from(UPDATED_ORACLE_VALUE),
            decimals: Uint128::from(UPDATED_DECIMALS),
            timestamp: mock_env().block.time,
        };

        let oracle_value: OracleValue = ORACLE_VALUE.load(&deps.storage).unwrap();
        assert_eq!(ov, oracle_value);

        let msg = ExecuteMsg::QueryOracleValue {};
        let env = mock_env();
        let info = mock_info("oracle", &coins(1000, "earth"));
        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!("oracle-value", res.attributes[0].key);
        assert_eq!("100000", res.attributes[0].value);
        assert_eq!("decimals", res.attributes[1].key);
        assert_eq!("5", res.attributes[1].value);
        assert_eq!("timestamp", res.attributes[2].key);

        assert_eq!(3, res.attributes.len());
    }

    #[test]
    fn test_withdraw_fees_success() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = init_msg(Uint128::from(100u128));
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        STATE
            .update(
                &mut deps.storage,
                |mut state: State| -> Result<_, StdError> {
                    state.fees_accrued = Uint128::from(1000u128);
                    Ok(state)
                },
            )
            .unwrap();

        let msg = ExecuteMsg::WithdrawFees {
            amount: Uint128::from(900u128),
        };

        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));
        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        let state: State = STATE.load(&deps.storage).unwrap();

        assert_eq!(
            res.messages[0],
            SubMsg::new(BankMsg::Send {
                to_address: state.owner.into_string(),
                amount: coins(900u128.into(), state.denom),
            })
        );

        let state: State = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.fees_accrued, Uint128::from(100u128));
    }
}
