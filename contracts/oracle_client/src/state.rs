use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct OracleValue {
    pub value: Uint128,
    pub decimals: Uint128,
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub oracle_contract_address: Addr,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const ORACLE_VALUE: Item<OracleValue> = Item::new("oracle_value");
