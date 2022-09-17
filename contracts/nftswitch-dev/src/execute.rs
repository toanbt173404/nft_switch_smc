
use cosmwasm_std::{
    coin, to_binary, BankMsg, Coin, Decimal, DepsMut, Env, Event, MessageInfo, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw721::Cw721ExecuteMsg;
use cw721_base::helpers::Cw721Contract;
use disburse::msg::DisburseReward;
use crate::helpers::{only_owner, price_validate};
use crate::state::{trade_key, trades, Config, ExecuteEnv, Trade, CONFIG};
use crate::ContractError;
use cw_utils::{must_pay, nonpayable};

const NATIVE_DENOM: &str = "uluna";

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    fee_admin: Option<String>,
    commission_addr: Option<String>,
    buyer_fee: Option<Decimal>,
    seller_fee: Option<Decimal>,
    listing_fee: Option<Uint128>,
    e_break: Option<bool>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(admin) = admin {
        config.admin = deps.api.addr_validate(&admin)?;
    }

    if let Some(fee_admin) = fee_admin {
        config.fee_admin = deps.api.addr_validate(&fee_admin)?;
    }

    if let Some(commission_addr) = commission_addr {
        config.commission_addr = deps.api.addr_validate(&commission_addr)?;
    }

    if let Some(buyer_fee) = buyer_fee {
        config.buyer_fee = buyer_fee;
    }

    if let Some(seller_fee) = seller_fee {
        config.seller_fee = seller_fee;
    }

    if let Some(listing_fee) = listing_fee {
        config.listing_fee = Uint128::from(listing_fee);
    }

    if let Some(e_break) = e_break {
        config.e_break = e_break;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn try_create_trade(
    env: ExecuteEnv,
    nft_addr: String,
    nft_id: String,
    buyer_addr: String,
    price: Coin,
) -> Result<Response, ContractError> {
    let ExecuteEnv { deps, info, env } = env;

    let mut res = Response::new();

    let cfg = CONFIG.load(deps.storage)?;

    if cfg.e_break {
        return Err(ContractError::EmergencyBreakActivated {});
    }

    let amount_send = must_pay(&info, NATIVE_DENOM)?;

    if cfg.listing_fee != amount_send {
        return Err(ContractError::MissingListingFee {});
    }

    let nft_collection_addr = deps.api.addr_validate(&nft_addr)?;
    // nonpayable(&info)?;
    price_validate(&price)?;

    // TODO: Find alternative to check if the owner of the NFT is calling this
    only_owner(
        deps.as_ref(),
        &info,
        &nft_collection_addr.clone(),
        nft_id.to_string(),
    )?;

    Cw721Contract(nft_collection_addr.clone()).approval(
        &deps.querier,
        nft_id.to_string(),
        env.contract.address.to_string(),
        None,
    )?;

    let seller = info.sender;
    let buyer = deps.api.addr_validate(&buyer_addr)?;

    let tradekey = trade_key(&buyer, &nft_collection_addr, nft_id.clone());

    let trade = Trade {
        seller: seller.clone(),
        buyer: buyer.clone(),
        price: price.clone(),
        nft_collection: nft_collection_addr.clone(),
        nft_id,
        seller_fee: cfg.seller_fee,
        buyer_fee: cfg.buyer_fee,
        is_confirmed_trade: false,
    };

    trades().save(deps.storage, tradekey, &trade)?;

    let event = Event::new("create-trade")
        .add_attribute("action", "try_create_trade")
        .add_attribute("seller", seller.to_string())
        .add_attribute("buyer", buyer.to_string())
        .add_attribute("price", price.amount.to_string());

    let transfer_fee_to_fee_admin = BankMsg::Send {
        to_address: cfg.fee_admin.to_string(),
        amount: vec![coin(cfg.listing_fee.u128(), NATIVE_DENOM)],
    };
    res = res.add_message(transfer_fee_to_fee_admin);

    Ok(res.add_event(event))
}

pub fn try_cancel_trade(
    deps: DepsMut,
    info: MessageInfo,
    buyer: Option<String>,
    seller: Option<String>,
    nft_collection: String,
    nft_id: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    if cfg.e_break {
        return Err(ContractError::EmergencyBreakActivated {});
    }

    nonpayable(&info)?;

    if let Some(buyer) = buyer {
        let buyer_addr = &deps.api.addr_validate(&buyer)?;

        let key = trade_key(
            buyer_addr,
            &deps.api.addr_validate(&nft_collection)?,
            nft_id.clone(),
        );
        let trade = trades().load(deps.storage, key.clone())?;
        if &trade.buyer != buyer_addr {
            return Err(ContractError::Unauthorized {});
        }

        trades().remove(deps.storage, key)?;
        let event = Event::new("buyer-cancels-trade")
            .add_attribute("collection", nft_collection)
            .add_attribute("nft_id", nft_id)
            .add_attribute("buyer", buyer);

        let res = Response::new().add_event(event);

        Ok(res)
    } else if let Some(seller) = seller {
        let seller_addr = &deps.api.addr_validate(&seller)?;
        let key = trade_key(
            seller_addr,
            &deps.api.addr_validate(&nft_collection)?,
            nft_id.clone(),
        );
        let trade = trades().load(deps.storage, key.clone())?;
        if &trade.seller != seller_addr {
            return Err(ContractError::Unauthorized {});
        }

        trades().remove(deps.storage, key)?;
        let event = Event::new("seller-cancels-trade")
            .add_attribute("collection", nft_collection)
            .add_attribute("nft_id", nft_id)
            .add_attribute("seller", seller);

        let res = Response::new().add_event(event);
        Ok(res)
    } else {
        return Err(ContractError::ParameterMissing {});
    }
}

pub fn try_execute_trade(
    env: ExecuteEnv,
    buyer: String,
    nft_collection: String,
    nft_id: String,
) -> Result<Response, ContractError> {
    let ExecuteEnv { deps, info, env: _ } = env;

    let mut res = Response::new();

    let cfg = CONFIG.load(deps.storage)?;

    if cfg.e_break {
        return Err(ContractError::EmergencyBreakActivated {});
    }

    let buyer = deps.api.addr_validate(&buyer)?;
    let nft_collection = deps.api.addr_validate(&nft_collection)?;

    // retrieve trade
    let trade_key = trade_key(&buyer, &nft_collection, nft_id.clone());
    let trade = trades().load(deps.storage, trade_key.clone())?;

    if !trade.is_confirmed_trade {
        return Err(ContractError::TradeNotConfirmed {});
    }

    // verify sender is the buyer
    if info.sender != trade.buyer {
        return Err(ContractError::Unauthorized {});
    }

    // calculate commission
    let buyer_fee = cfg.buyer_fee.clone() * trade.price.amount.clone();
    let seller_fee = cfg.seller_fee.clone() * trade.price.amount.clone();

    let expected_sent_amount = trade.price.amount.clone() + buyer_fee;

    let mut amount_send = Uint128::zero();

    if expected_sent_amount != Uint128::zero() {
        amount_send = must_pay(&info, NATIVE_DENOM)?;

        if expected_sent_amount != amount_send {
            return Err(ContractError::PaymentAmountMismatch {});
        }
    }
    let commission = buyer_fee + seller_fee;

    // send NFT
    transfer_nft(&trade, &mut res)?;

    // send amount to seller
    if amount_send != Uint128::zero() {
        transfer_coin_to_seller(&trade, amount_send.checked_sub(commission).unwrap(), &mut res)?;
    }

    // send commission
    if commission != Uint128::zero() {
        transfer_commission(&deps, commission.clone(), &mut res)?;
    }

    // remove trade from state
    trades().remove(deps.storage, trade_key)?;

    Ok(res
        .add_attribute("method", "execute_trade")
        .add_attribute("buyer", trade.buyer.to_string())
        .add_attribute("buyer_fee", buyer_fee)
        .add_attribute("seller", trade.seller.to_string())
        .add_attribute("seller_fee", seller_fee)
        .add_attribute("commission", commission)
        .add_attribute("nft_collection", trade.nft_collection.to_string())
        .add_attribute("nft_id", trade.nft_id.to_string()))
}

pub fn try_confirm_trade(
    env: ExecuteEnv,
    buyer: String,
    nft_collection: String,
    nft_id: String,
    seller_fee_pct: Decimal,
    buyer_fee_pct: Decimal,
    is_confirmed_by_fee_admin: bool,
) -> Result<Response, ContractError> {
    let ExecuteEnv { deps, info, env: _ } = env;

    let cfg = CONFIG.load(deps.storage)?;

    nonpayable(&info)?;

    if cfg.e_break {
        return Err(ContractError::EmergencyBreakActivated {});
    }

    if info.sender != cfg.fee_admin {
        return Err(ContractError::UnauthorizedOwner {});
    }

    let buyer = deps.api.addr_validate(&buyer)?;
    let nft_collection = deps.api.addr_validate(&nft_collection)?;

    let trade_key = trade_key(&buyer, &nft_collection, nft_id.clone());
    let mut trade = trades().load(deps.storage, trade_key.clone())?;

    // parameter is_confirmed false means that the buyer does not own the NFT collection or some other
    // reason to not confirm the trade, so the trade is removed.
    if !is_confirmed_by_fee_admin {
        trades().remove(deps.storage, trade_key.clone())?;
        Ok(Response::new().add_attribute("method", "remove_trade"))
    } else {
        if trade.is_confirmed_trade {
            return Err(ContractError::AlreadyConfirmedFees {});
        }

        trade.is_confirmed_trade = true;

        trade.buyer_fee = Decimal::from(buyer_fee_pct);
        trade.seller_fee = Decimal::from(seller_fee_pct);

        trades().save(deps.storage, trade_key, &trade)?;

        Ok(Response::new().add_attribute("method", "update_fees"))
    }
}

fn transfer_nft(trade: &Trade, res: &mut Response) -> StdResult<()> {
    let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
        token_id: trade.nft_id.to_string(),
        recipient: trade.buyer.to_string(),
    };

    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: trade.nft_collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    };
    res.messages.push(SubMsg::new(exec_cw721_transfer));

    let event = Event::new("finalize-switch")
        .add_attribute("nft_collection", trade.nft_collection.to_string())
        .add_attribute("nft_id", trade.nft_id.to_string())
        .add_attribute("seller", trade.seller.to_string())
        .add_attribute("buyer", trade.buyer.to_string())
        .add_attribute("price", trade.price.to_string());

    res.events.push(event);

    Ok(())
}

// send amount to seller
fn transfer_coin_to_seller(trade: &Trade, amount: Uint128, res: &mut Response) -> StdResult<()> {
    let transfer_amount: Coin = Coin::new(u128::from(amount), NATIVE_DENOM);

    let seller_transfer_msg = BankMsg::Send {
        to_address: trade.seller.to_string(),
        amount: vec![transfer_amount],
    };
    res.messages.push(SubMsg::new(seller_transfer_msg));
    Ok(())
}

// send commission
fn transfer_commission(deps: &DepsMut, commission: Uint128, res: &mut Response) -> StdResult<()> {
    let cfg = CONFIG.load(deps.storage)?;


    let disburse_reward_msg = DisburseReward {
        amount : commission.clone()
    };
    let exec_disburse_reward = WasmMsg::Execute {
        contract_addr: cfg.commission_addr.to_string(),
        msg: to_binary(&disburse_reward_msg)?,
        funds: vec![coin(commission.u128(), NATIVE_DENOM)],
    };
    res.messages.push(SubMsg::new(exec_disburse_reward));

    Ok(())
}
