use bs_controllers::AdminError;
use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Base(#[from] bs721_base::ContractError),

    #[error("AccountIsNotTokenized")]
    AccountIsNotTokenized {},

    #[error("IncorrectBitsongAccountOwnershipToken: {got} ,  {wanted}")]
    IncorrectBitsongAccountOwnershipToken { got: String, wanted: String },

    #[error("AccountNotFound")]
    AccountNotFound {},

    #[error("AddressAlreadyMapped")]
    AddressAlreadyMapped {},

    #[error("InvalidAddress")]
    InvalidAddress {},

    #[error("RecordAccountAlreadyExists")]
    RecordAccountAlreadyExists {},

    #[error("RecordAccountEmpty")]
    RecordAccountEmpty {},

    #[error("RecordAccountTooLong")]
    RecordAccountTooLong {},

    #[error("RecordValueTooLong")]
    RecordValueTooLong {},

    #[error("RecordValueEmpty")]
    RecordValueEmpty {},

    #[error("UnauthorizedVerification")]
    UnauthorizedVerification {},

    #[error("Invalid Metadata")]
    InvalidMetadata {},

    #[error("Unauthorized: Not collection minter")]
    UnauthorizedMinter {},

    #[error("Unauthorized: Not contract creator or admin")]
    UnauthorizedCreatorOrAdmin {},

    #[error("TooManyRecords max: {max}")]
    TooManyRecords { max: u32 },

    #[error("TooManyReverseMaps max: {max}, have: {have}")]
    TooManyReverseMaps { max: u32, have: u32 },

    #[error("CannotRemoveEmptyMap")]
    CannotRemoveEmptyMap {},

    #[error("CannotRemoveMoreThanWillExists")]
    CannotRemoveMoreThanWillExists {},

    #[error("NotImplemented")]
    NotImplemented {},
}
