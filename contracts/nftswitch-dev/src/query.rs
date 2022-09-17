use crate::msg::{ConfigResponse, TradeResponse, TradesResponse};
use crate::state::{trades, CONFIG};
use cosmwasm_std::{Addr, Deps, Order, StdResult};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        admin: cfg.admin,
        commission_addr: cfg.commission_addr,
        fee_admin: cfg.fee_admin,
        buyer_fee: cfg.buyer_fee,
        seller_fee: cfg.seller_fee,
        e_break: cfg.e_break,
        listing_fee: cfg.listing_fee
    })
}

pub fn query_trade(
    deps: Deps,
    buyer: Addr,
    nft_collection: Addr,
    nft_id: String,
) -> StdResult<TradeResponse> {
    let trade = trades().may_load(deps.storage, (buyer.clone(), nft_collection, nft_id))?;

    Ok(TradeResponse { trade })
}

// fn query_trades(
//     deps: Deps
// ) -> StdResult<TradesResponse> {
//     let list: Vec<_> = trades()
//         .idx.id
//         .prefix(i32::from(1))
//         .range(deps.storage, None, None, Order::Ascending)
//         .collect::<StdResult<_>>().unwrap()?;
//
//     Ok(TradesResponse { trades: list })
// }
pub fn query_trades_by_buyer(
    deps: Deps,
    buyer: Addr,
    limit: Option<u32>,
) -> StdResult<TradesResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let trades = trades()
        .idx
        .buyer
        .prefix(buyer)
        .range(deps.storage, None, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|(_, b)|b))
        .collect::<StdResult<Vec<_>>>().unwrap();

    Ok(TradesResponse { trades })
}

pub fn query_trades_by_seller(
    deps: Deps,
    seller: Addr,
    limit: Option<u32>,
) -> StdResult<TradesResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let trades = trades()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, None, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|(_, b)|b))
        .collect::<StdResult<Vec<_>>>().unwrap();

    Ok(TradesResponse { trades })
}
