use cosmwasm_std::{StdError, Timestamp};
use cw_controllers::AdminError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Account Minter: Unauthorized")]
    Unauthorized {},

    #[error("MintingPaused")]
    MintingPaused {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Invalid name")]
    InvalidAccount {},

    #[error("Account too short")]
    AccountTooShort {},

    #[error("Account too long")]
    AccountTooLong {},

    #[error("Incorrect payment, got: {got}, expected {expected}")]
    IncorrectPayment { got: u128, expected: u128 },

    #[error("InvalidTradingStartTime {0} < {1}")]
    InvalidTradingStartTime(Timestamp, Timestamp),

    #[error("MintingNotStarted")]
    MintingNotStarted {},

    #[error("Reply error")]
    ReplyOnSuccess {},

    #[error("Invalid Whitelist Type")]
    InvalidWhitelistType {},
}