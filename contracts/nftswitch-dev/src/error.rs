use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError{val: String},

    #[error("Contract needs approval")]
    NeedsApproval {},

    #[error("InvalidPrice")]
    InvalidPrice {},

    #[error("InvalidDenom")]
    InvalidDenom {},

    #[error("Must own NFT Collection")]
    NotOwnerOfNFTCollection {},

    #[error("Trade not confirmed")]
    TradeNotConfirmed {},

    #[error("Fees already confirmed")]
    AlreadyConfirmedFees {},

    #[error("Native token balance mismatch between the argument and the transferred")]
    PaymentAmountMismatch {},

    #[error("Have to send listing fee to contract")]
    MissingListingFee {},

    #[error("Emergency Break Activated -- All Actions Paused")]
    EmergencyBreakActivated {},

    #[error("Seller or Buyer parameter required.")]
    ParameterMissing {},

    #[error("UnauthorizedOwner")]
    UnauthorizedOwner {},

    #[error("UnauthorizedOperator")]
    UnauthorizedOperator {},

    #[error("Token reserved")]
    TokenReserved {},

    #[error("Given operator address already registered as an operator")]
    OperatorAlreadyRegistered {},

    #[error("Given operator address is not registered as an operator")]
    OperatorNotRegistered {},

    #[error("InvalidContractVersion")]
    InvalidContractVersion {},

    #[error("{0}")]
    TradePaymentError(#[from] PaymentError),

    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

// #[derive(Error, Debug, PartialEq)]
// pub enum PaymentError {
//     #[error("Must send reserve token '{0}'")]
//     MissingDenom(String),
//
//     #[error("Received unsupported denom '{0}'")]
//     ExtraDenom(String),
//
//     #[error("Sent more than one denomination")]
//     MultipleDenoms {},
//
//     #[error("No funds sent")]
//     NoFunds {},
//
//     #[error("This message does no accept funds")]
//     NonPayable {},
// }