use cosmwasm_std::{Deps, StdResult};
use crate::msg::{AllEnergyTransferOffersResponse, AllEnergyTransfersResponse, DenomResponse, EnergyTransfer, EnergyTransferOffer, EnergyTransferOfferResponse, EnergyTransferOffersByOwnerResponse, EnergyTransferResponse, EnergyTransfersByOwnerResponse, OwnEnergyTransfersResponse, TransferStatus};

use crate::state::{DENOM, ENERGY_TRANSFER_OFFERS, ENERGY_TRANSFERS};

pub fn query_denom(deps: Deps) -> StdResult<DenomResponse> {
    let resp = DenomResponse {
        denom: DENOM.load(deps.storage)?,
    };
    Ok(resp)
}

pub fn query_energy_transfer_offer(deps: Deps, id: u64) -> StdResult<EnergyTransferOfferResponse> {
    let offer = ENERGY_TRANSFER_OFFERS.load(deps.storage, id)?;
    let resp = EnergyTransferOfferResponse {
        energy_transfer_offer: offer,
    };
    Ok(resp)
}

pub fn query_all_energy_transfer_offers(deps: Deps) -> StdResult<AllEnergyTransferOffersResponse> {
    let offers = get_energy_transfer_offers(deps)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<Vec<_>>>()?;

    let resp = AllEnergyTransferOffersResponse {
        energy_transfer_offers: offers,
    };
    Ok(resp)
}

pub fn query_energy_transfer(deps: Deps, id: u64) -> StdResult<EnergyTransferResponse> {
    let transfer = ENERGY_TRANSFERS.load(deps.storage, id)?;
    let resp = EnergyTransferResponse {
        energy_transfer: transfer,
    };
    Ok(resp)
}

pub fn query_all_energy_transfers(deps: Deps) -> StdResult<AllEnergyTransfersResponse> {
    let transfers = get_energy_transfers(deps)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<Vec<_>>>()?;
    let resp = AllEnergyTransfersResponse {
        energy_transfers: transfers,
    };
    Ok(resp)
}

pub fn query_energy_transfer_offers(deps: Deps, owner: String) -> StdResult<EnergyTransferOffersByOwnerResponse> {
    let offers = get_energy_transfer_offers(deps)
        .map(|item| item.map(|(_, v)| v))
        .filter(|offer| offer.as_ref().unwrap().owner == owner)
        .collect::<StdResult<Vec<_>>>()?;
    let resp = EnergyTransferOffersByOwnerResponse {
        energy_transfer_offers: offers,
    };
    Ok(resp)
}

pub fn query_own_energy_transfers(deps: Deps, driver: String, transfer_status: TransferStatus) -> StdResult<OwnEnergyTransfersResponse> {
    let transfers = get_energy_transfers(deps)
        .map(|item| item.map(|(_, v)| v))
        .filter(|transfer| transfer.as_ref().unwrap().driver == driver && transfer.as_ref().unwrap().status == transfer_status)
        .collect::<StdResult<Vec<_>>>()?;
    let resp = OwnEnergyTransfersResponse {
        energy_transfers: transfers,
    };
    Ok(resp)
}

pub fn query_energy_transfers(deps: Deps, owner: String) -> StdResult<EnergyTransfersByOwnerResponse> {
    let transfers = get_energy_transfers(deps)
        .map(|item| item.map(|(_, v)| v))
        .filter(|transfer| transfer.as_ref().unwrap().owner == owner)
        .collect::<StdResult<Vec<_>>>()?;
    let resp = EnergyTransfersByOwnerResponse {
        energy_transfers: transfers,
    };
    Ok(resp)
}

pub fn get_energy_transfers(deps: Deps<'_>) -> Box<dyn Iterator<Item=StdResult<(u64, EnergyTransfer)>> + '_> {
    return ENERGY_TRANSFERS.range(deps.storage, None, None, cosmwasm_std::Order::Ascending);
}

pub fn get_energy_transfer_offers(deps: Deps<'_>) -> Box<dyn Iterator<Item=StdResult<(u64, EnergyTransferOffer)>> + '_> {
    return ENERGY_TRANSFER_OFFERS.range(deps.storage, None, None, cosmwasm_std::Order::Ascending);
}
