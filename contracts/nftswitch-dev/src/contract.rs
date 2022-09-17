use crate::execute::{
    try_cancel_trade, try_create_trade, try_execute_trade, try_update_config, try_confirm_trade,
};
use crate::msg::QueryMsg::{GetConfig, GetTrade, GetTradesByBuyer, GetTradesBySeller};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{query_config, query_trade, query_trades_by_seller, query_trades_by_buyer};
use crate::state::{Config, ExecuteEnv, CONFIG};
use crate::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[cfg_attr(feature = "library", entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        admin: info.sender.clone(),
        fee_admin: deps.api.addr_validate(&msg.fee_admin)?,
        commission_addr: deps.api.addr_validate(&msg.commission_addr)?,
        seller_fee: msg.seller_fee,
        buyer_fee: msg.seller_fee,
        listing_fee: msg.listing_fee,
        e_break: false,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(feature = "library", entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateTrade {
            nft_addr,
            nft_id,
            buyer_addr,
            sale_price,
        } => try_create_trade(
            ExecuteEnv { deps, env, info },
            nft_addr,
            nft_id,
            buyer_addr,
            sale_price,
        ),
        ExecuteMsg::CancelTrade {
            buyer,
            seller,
            nft_collection,
            nft_id,
        } => try_cancel_trade(deps, info, buyer, seller, nft_collection, nft_id),
        ExecuteMsg::ExecuteTrade {
            buyer,
            nft_collection,
            nft_id,
        } => try_execute_trade(
            ExecuteEnv { deps, env, info },
            buyer,
            nft_collection,
            nft_id,
        ),
        ExecuteMsg::UpdateConfig {
            admin,
            fee_admin,
            commission_addr,
            e_break,
            buyer_fee,
            seller_fee,
            listing_fee
        } => try_update_config(
            deps,
            env,
            info,
            admin,
            fee_admin,
            commission_addr,
            buyer_fee,
            seller_fee,
            listing_fee,
            e_break,
        ),
        ExecuteMsg::ConfirmTrade {
            buyer,
            nft_collection,
            nft_id,
            seller_fee_pct,
            buyer_fee_pct,
            is_confirmed_by_fee_admin,
        } => try_confirm_trade(
            ExecuteEnv { deps, env, info },
            buyer,
            nft_collection,
            nft_id,
            seller_fee_pct,
            buyer_fee_pct,
            is_confirmed_by_fee_admin,
        ),
    }
}

#[cfg_attr(feature = "library", entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        GetConfig {} => to_binary(&query_config(deps)?),
        GetTrade {
            buyer,
            nft_collection,
            nft_id,
        } => to_binary(&query_trade(
            deps,
            api.addr_validate(&buyer)?,
            api.addr_validate(&nft_collection)?,
            nft_id,
        )?),
        // GetAllTrades {} => to_binary(&query_trades(deps)?),
        // GetTrades {} => to_binary(&query_trades(deps)?),
        GetTradesByBuyer { buyer, limit } => to_binary(&query_trades_by_buyer(deps, buyer, limit)?),
        GetTradesBySeller { seller, limit } => { to_binary(&query_trades_by_seller(deps, seller, limit)?) }
    }
}

/*
Using admin controller
use this line to assert that the info.sender is the admin and if it isnt it will throw an error
ADMIN.assert_admin(deps.branch(), maybe_addr(&info.sender));
 */

#[cfg(test)]
mod tests {
    // use super::*;
    // use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    // use cosmwasm_std::{coins, from_binary};
    // use crate::contract::query;
    //
    // #[test]
    // fn proper_initialization() {}
}
