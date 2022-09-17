use cosmwasm_std::Uint128;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DisburseReward {
    pub amount: Uint128,
}
