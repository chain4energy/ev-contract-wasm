use std::fmt;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    PublishEnergyTransferOffer {
        charger_id: String,
        location: Location,
        tariff: u64,
        name: String,
        plug_type: PlugType,
    },
    RemoveEnergyOffer {
        energy_offer_id: u64
    },
    StartEnergyTransfer {
        driver: String,
        energy_transfer_offer_id: u64,
        energy_to_transfer: u64,
    },
    EnergyTransferStarted { energy_transfer_id: u64 },
    EnergyTransferCompleted { energy_transfer_id: u64, used_service_units: u64 },
    CancelEnergyTransfer { energy_transfer_id: u64 },
    RemoveEnergyTransfer { energy_transfer_id: u64 },
}

#[cw_serde]
pub struct EnergyTransferOffer {
    pub id: u64,
    pub owner: String,
    pub charger_id: String,
    pub charger_status: ChargerStatus,
    pub location: Location,
    pub tariff: u64,
    pub name: String,
    pub plug_type: PlugType,
}

#[cw_serde]
pub struct Location {
    pub latitude: Decimal,
    pub longitude: Decimal,
}

#[cw_serde]
pub struct OldEnergyTransferOffer {
    pub id: u64,
    pub owner: String,
    pub charger_id: String,
    pub charger_status: ChargerStatus,
    pub location: OldLocation,
    pub tariff: u64,
    pub name: String,
    pub plug_type: PlugType,
}
#[cw_serde]
pub struct OldLocation {
    pub latitude: String,
    pub longitude: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub enum ChargerStatus {
    Active,
    Busy,
    Inactive,
    Unspecified,
}

impl fmt::Display for ChargerStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChargerStatus::Active => write!(f, "Active"),
            ChargerStatus::Busy => write!(f, "Busy"),
            ChargerStatus::Inactive => write!(f, "Inactive"),
            ChargerStatus::Unspecified => write!(f, "Unspecified"),
        }
    }
}

#[cw_serde]
pub enum PlugType {
    Type1,
    Type2,
    CHAdeMO,
    CCS,
    Unspecified,
}

#[cw_serde]
pub struct EnergyTransfer {
    pub id: u64,
    pub energy_transfer_offer_id: u64,
    pub charger_id: String,
    pub owner: String,
    pub driver: String,
    pub offered_tariff: u64,
    pub status: TransferStatus,
    pub collateral: u64,
    pub energy_to_transfer: u64,
    pub energy_transferred: u64,
    pub paid_date: Timestamp,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, JsonSchema)]
pub enum TransferStatus {
    Requested,
    Ongoing,
    Paid,
    Cancelled,
    Unspecified,
}

impl fmt::Display for TransferStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransferStatus::Requested => write!(f, "Requested"),
            TransferStatus::Ongoing => write!(f, "Ongoing"),
            TransferStatus::Paid => write!(f, "Paid"),
            TransferStatus::Cancelled => write!(f, "Cancelled"),
            TransferStatus::Unspecified => write!(f, "Unspecified"),
        }
    }
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DenomResponse)]
    Denom {},
    #[returns(EnergyTransferOfferResponse)]
    EnergyTransferOffer { id: u64 },
    #[returns(AllEnergyTransferOffersResponse)]
    AllEnergyTransferOffers {},
    #[returns(EnergyTransferResponse)]
    EnergyTransfer { id: u64 },
    #[returns(AllEnergyTransfersResponse)]
    AllEnergyTransfers {},
    #[returns(EnergyTransferOffersByOwnerResponse)]
    EnergyTransferOffers { owner: String },
    #[returns(OwnEnergyTransfersResponse)]
    OwnEnergyTransfers { driver: String, transfer_status: TransferStatus},
    #[returns(EnergyTransfersByOwnerResponse)]
    EnergyTransfers { owner: String, },
}

#[cw_serde]
pub struct DenomResponse {
    pub denom: String,
}
#[cw_serde]
pub struct EnergyTransferOfferResponse {
    pub energy_transfer_offer: EnergyTransferOffer,
}
#[cw_serde]
pub struct AllEnergyTransferOffersResponse {
    pub energy_transfer_offers: Vec<EnergyTransferOffer>,
}
#[cw_serde]
pub struct EnergyTransferResponse {
    pub energy_transfer: EnergyTransfer,
}
#[cw_serde]
pub struct AllEnergyTransfersResponse {
    pub energy_transfers: Vec<EnergyTransfer>,
}
#[cw_serde]
pub struct EnergyTransferOffersByOwnerResponse {
    pub energy_transfer_offers: Vec<EnergyTransferOffer>,
}
#[cw_serde]
pub struct OwnEnergyTransfersResponse {
    pub energy_transfers: Vec<EnergyTransfer>,
}
#[cw_serde]
pub struct EnergyTransfersByOwnerResponse {
    pub energy_transfers: Vec<EnergyTransfer>,
}
