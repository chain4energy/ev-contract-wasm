use cosmwasm_std::StdError;
use thiserror::Error;
use crate::msg::{ChargerStatus, TransferStatus};

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Invalid signer: {0}")]
    InvalidSigner(String),

    #[error("Invalid charger status. Expected {0} got {1}")]
    InvalidChargerStatus(ChargerStatus, ChargerStatus),

    #[error("Invalid charger status. Expected {0} or {1} got {2}")]
    InvalidChargerMultipleStatuses(ChargerStatus, ChargerStatus, ChargerStatus),

    #[error("Energy offer not found: {0}")]
    EnergyOfferNotFound(u64),

    #[error("Invalid Driver")]
    InvalidDriver,

    #[error("Offered Tariff is Zero")]
    ZeroTariff,

    #[error("Energy to Transfer is Zero")]
    ZeroEnergy,

    #[error("Energy transfer with id {0} not found.")]
    EnergyTransferNotFound(u64),

    #[error("Invalid energy transfer status. Expected {0} got {1}")]
    InvalidEnergyTransferStatus(TransferStatus, TransferStatus),

    #[error("Invalid energy transfer status. Expected {0} or {1} got {2}")]
    InvalidEnergyTransferMultipleStatuses(TransferStatus, TransferStatus, TransferStatus),

    #[error("Invalid funds. Expected {0} got {1}")]
    InvalidFunds(String, String),

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}


impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}