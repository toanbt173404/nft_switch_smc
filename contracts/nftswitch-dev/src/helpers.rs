use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdResult, WasmMsg, WasmQuery, Coin, Deps, QueryRequest, MessageInfo};
use cw721::{Cw721QueryMsg, OwnerOfResponse, TokensResponse};
use cw721_base::helpers::Cw721Contract;
use crate::ContractError;
use crate::msg::{ExecuteMsg};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
            .into())
    }
}

pub fn price_validate(price: &Coin) -> Result<(), ContractError> {
    if price.denom != "uluna" {
        return Err(ContractError::InvalidPrice {});
    }

    Ok(())
}

pub fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: String,
) -> Result<OwnerOfResponse, ContractError> {
    let res = Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id.clone(), false)?;
    if res.owner != info.sender {
        return Err(ContractError::UnauthorizedOwner {});
    }

    Ok(res)
}

// Query a wallet to see which NFTs it holds.
// This returns "tokens" which is a vector of all the token ids that wallet holds.
// Tokens{owner, start_after, limit} - List all token_ids that belong to a given owner.
// Return type is TokensResponse{tokens: Vec<token_id>}.

pub fn addr_owns_collection(deps: Deps, wallet: Addr, collection: &Addr) -> Result<bool, ContractError> {
    let query_response: TokensResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: collection.to_string(),
            msg: to_binary(&Cw721QueryMsg::Tokens {
                owner: wallet.to_string(),
                start_after: None,
                limit: None,
            })?,
        }))?;

    Ok(query_response.tokens.len() > 0)
}