use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Coin, Decimal, DepsMut, Env, MessageInfo, Uint128};
use cw_storage_plus::{Index, IndexedMap, IndexList, Item, MultiIndex};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Trade {
    pub seller: Addr,
    pub buyer: Addr,
    pub price: Coin,
    pub nft_collection: Addr,
    pub nft_id: String,
    pub is_confirmed_trade: bool,
    pub seller_fee: Decimal,
    pub buyer_fee: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub fee_admin: Addr,
    pub commission_addr: Addr,
    pub seller_fee: Decimal,
    pub buyer_fee: Decimal,
    pub listing_fee: Uint128,
    pub e_break: bool,
}

/// (Buyer, Seller, NFT Contract, NFT ID)
pub type TradeKey = (Addr, Addr, String);

pub fn trade_key(buyer: &Addr, nft_collection: &Addr, nft_id: String) -> TradeKey {
    (buyer.clone(), nft_collection.clone(), nft_id)
}

pub const CONFIG: Item<Config> = Item::new("config");

pub struct ExecuteEnv<'a> {
    pub(crate) deps: DepsMut<'a>,
    pub(crate) env: Env,
    pub(crate) info: MessageInfo,
}

/// Defines indices for accessing Asks
pub struct TradeIndices<'a> {
    pub collection: MultiIndex<'a, Addr, Trade, TradeKey>,
    pub seller: MultiIndex<'a, Addr, Trade, TradeKey>,
    pub buyer: MultiIndex<'a, Addr, Trade, TradeKey>,
}

impl<'a> IndexList<Trade> for TradeIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<Trade>> + '_> {
        let v: Vec<&dyn Index<Trade>> = vec![&self.collection, &self.seller, &self.buyer];
        Box::new(v.into_iter())
    }
}

pub fn trades<'a>() -> IndexedMap<'a, TradeKey, Trade, TradeIndices<'a>> {
    let indexes = TradeIndices {
        collection: MultiIndex::new(
            |d: &Trade| (d.nft_collection.clone()),
            "trades",
            "trades__nft_collection"),
        buyer: MultiIndex::new(
            |d: &Trade| (d.buyer.clone()),
            "trades",
            "trades__buyer"),
        seller: MultiIndex::new(
            |d: &Trade| (d.seller.clone()),
            "trades",
            "trades__seller"),
    };
    IndexedMap::new("trades", indexes)
}
