
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub nft_switch_address: Addr,
    pub payees: Vec<Payees>
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Payees {
  pub payee_address: Addr, 
  pub percent_paid: Decimal,
  pub claimable_amount: Uint128
}

pub const CONFIG: Item<Config> = Item::new("config");
