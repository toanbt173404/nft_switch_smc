use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use crate::state::{Trade};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub fee_admin: String,
    pub commission_addr: String,
    pub buyer_fee: Decimal,
    pub seller_fee: Decimal,
    pub listing_fee: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateTrade { nft_addr: String, nft_id: String, buyer_addr: String, sale_price: Coin},
    CancelTrade { buyer: Option<String>, seller: Option<String>, nft_collection: String, nft_id: String },
    ExecuteTrade { buyer: String, nft_collection: String, nft_id: String },
    UpdateConfig {
        admin: Option<String>,
        fee_admin: Option<String>,
        commission_addr: Option<String>,
        buyer_fee: Option<Decimal>,
        seller_fee: Option<Decimal>,
        listing_fee: Option<Uint128>,
        e_break: Option<bool>
    },
    ConfirmTrade {
        buyer: String,
        nft_collection: String,
        nft_id: String,
        seller_fee_pct: Decimal,
        buyer_fee_pct: Decimal,
        is_confirmed_by_fee_admin: bool
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    // GetTrades {},
    GetTrade { buyer: String, nft_collection: String, nft_id: String  },
    GetTradesByBuyer { buyer: Addr, limit: Option<u32> },
    GetTradesBySeller { seller: Addr, limit: Option<u32> },
    // GetAllTrades {},
}


/// Offset for bid pagination
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeOffset {
    pub buyer: String,
    pub nft_collection: String,
    pub nft_id: String,
}

impl TradeOffset {
    pub fn new(  buyer: String,nft_collection: String, nft_id: String,) -> Self {
        TradeOffset {
            buyer,
            nft_collection,
            nft_id,
        }
    }
}
// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradesResponse {
    pub trades: Vec<Trade>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TradeResponse {
    pub trade: Option<Trade>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: Addr,
    pub fee_admin: Addr,
    pub commission_addr: Addr,
    pub e_break: bool,
    pub(crate) buyer_fee: Decimal,
    pub(crate) seller_fee: Decimal,
    pub listing_fee: Uint128
}