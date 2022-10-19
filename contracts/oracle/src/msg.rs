use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::OracleValue;

pub type Uint128 = cosmwasm_std::Uint128;
pub type Addr = cosmwasm_std::Addr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    GrantOracleRole { address: Addr },
    UpdateOracleValue { value: Uint128, decimals: Uint128 },
    QueryOracleValue {},
    SetFee { amount: Uint128 },
    WithdrawFees { amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConsumerMsg {
    SetOracleValue { value: OracleValue },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Status {},
    OracleRole {},
    Fee {},
    FeesAccrued {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct StatusResponse {
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct OracleRoleResponse {
    pub address: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct FeeResponse {
    pub fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct FeesAccruedResponse {
    pub fees_accrued: Uint128,
}
