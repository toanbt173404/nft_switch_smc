use crate::error::ContractError;
use crate::msg::{
     ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg
};
use crate::state::{Config, CONFIG, Payees};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult, Decimal, Response, Uint128, BankMsg, coin, SubMsg};
use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:disbursement";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;


    for payee in msg.payees.clone().into_iter() {
        let mut total_percent_paid = Decimal::zero();
        total_percent_paid = total_percent_paid + payee.percent_paid;

        if total_percent_paid != Decimal::one() {
            return Err(ContractError::InvalidPercentPaid {})
        }

    }

    let config = Config { admin: info.sender.clone(), nft_switch_address: deps.api.addr_validate(&msg.nft_switch_address)?, payees: msg.payees };
    CONFIG.save(deps.storage, &config)?;


    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { admin, nft_switch_address } => execute_update_config(deps, env, info, admin, nft_switch_address),
        ExecuteMsg::UpdatePayees { payees } => execute_update_payees(deps, env, info, payees),
        ExecuteMsg::DisburseReward { amount } => execute_update_disburse_reward(deps, env, info, amount),
    
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    nft_switch_address: Option<String>

) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(admin) = admin {
        config.admin = deps.api.addr_validate(&admin)?;
    }

    if let Some(nft_switch_address) = nft_switch_address {
        config.nft_switch_address = deps.api.addr_validate(&nft_switch_address)?;
    }


    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
)

}

pub fn execute_update_payees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    payees: Vec<Payees>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    for payee in payees.clone().into_iter() {
        let mut total_percent_paid = Decimal::zero();
        total_percent_paid = total_percent_paid + payee.percent_paid;

        if total_percent_paid != Decimal::one() {
            return Err(ContractError::InvalidPercentPaid {})
        }

    }
    config.payees = payees;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_payees")
        .add_attribute("sender", info.sender))
}


pub fn execute_update_disburse_reward(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut res = Response::new();

    if info.sender != config.nft_switch_address {
        return Err(ContractError::Unauthorized {});
    }

    

    for payee in config.payees.into_iter() {
        
        let amount_disburse = payee.percent_paid * amount;

   
        res.messages.push(SubMsg::new(BankMsg::Send {
            to_address: payee.payee_address.to_string(),
            amount: vec![coin(amount_disburse.u128(), "uluna")],
        }));

    }

    Ok(res
        .add_attribute("action", "disburse_reward")
        .add_attribute("sender", info.sender)
        .add_attribute("amount", amount))

}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
  
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
    }
}



fn query_config(deps: Deps, _env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: config.admin,
        nft_switch_address: config.nft_switch_address,
        payees: config.payees,
    })
}
