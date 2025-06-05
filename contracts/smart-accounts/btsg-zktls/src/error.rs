use cosmwasm_std::{StdError, VerificationError};

use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    VerificationError(#[from] VerificationError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("EPOCH id already exists")]
    AlreadyExists {},
    #[error("Key recovery error")]
    PubKeyErr {},
    #[error("Signature not appropriate")]
    SignatureErr {},
    #[error("Hash mismatch")]
    HashMismatchErr {},
    #[error("Not enough witness")]
    WitnessMismatchErr {},
    #[error("Cannot find")]
    NotFoundErr {},
}
