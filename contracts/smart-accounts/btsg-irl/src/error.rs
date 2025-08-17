use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized - only the smart account can perform this action")]
    Unauthorized {},

    #[error("Fantoken has not been created yet")]
    FantokenNotCreated {},

    #[error("Minting would exceed maximum supply")]
    ExceedsMaxSupply {},

    #[error("Unknown reply ID")]
    UnknownReplyId {},

    #[error("Invalid fantoken symbol")]
    InvalidSymbol {},

    #[error("Authenticator metadata must be provided when this contract to an account")]
    MissingAuthenticatorMetadata {},

    #[error("Invalid URI format")]
    InvalidUri {},

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
