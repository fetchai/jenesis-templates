use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::OracleValue;

pub type Uint128 = cosmwasm_std::Uint128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub oracle_contract_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    QueryOracleValue {},
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OracleMsg {
    QueryOracleValue { address: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    OracleContract {},
    OracleValue {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct OracleContractResponse {
    pub address: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct OracleValueResponse {
    pub oracle_value: OracleValue,
}
