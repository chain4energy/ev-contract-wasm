use cosmwasm_std::{ BankMsg, coins, DepsMut, Env, Event, MessageInfo, Response};
use crate::ContractError;
use crate::msg::{ChargerStatus, EnergyTransfer, EnergyTransferOffer, Location, PlugType, TransferStatus};
use crate::state::{DENOM, ENERGY_TRANSFER_COUNT, ENERGY_TRANSFER_OFFER_COUNT, ENERGY_TRANSFER_OFFERS, ENERGY_TRANSFERS};

pub fn publish_energy_transfer_offer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    charger_id: String,
    location: Location,
    tariff: u64,
    name: String,
    plug_type: PlugType,
) -> Result<Response, ContractError> {
    if charger_id.is_empty() || name.is_empty() {
        return Err(ContractError::CustomError {val: "charger_id and name must not be empty".parse().unwrap() });
    }

    let owner = info.sender.to_string();

    let mut counter: u64 = ENERGY_TRANSFER_OFFER_COUNT.load(deps.storage)?;
    counter += 1;
    ENERGY_TRANSFER_OFFER_COUNT.save(deps.storage, &counter)?;

    let new_energy_transfer_offer = EnergyTransferOffer {
        id: counter,
        owner: owner.clone(),
        charger_id: charger_id.clone(),
        charger_status: ChargerStatus::Active,
        location: location.clone(),
        tariff,
        name: name.clone(),
        plug_type: plug_type.clone(),
    };

    // Save to storage
    ENERGY_TRANSFER_OFFERS.save(deps.storage, counter, &new_energy_transfer_offer)?;

    let events = vec![
        Event::new("publish_energy_transfer_offer")
            .add_attribute("energy_transfer_offer_id", counter.to_string())
            .add_attribute("owner", owner.clone())
            .add_attribute("charger_id", charger_id.clone())
            .add_attribute("tariff", tariff.to_string())
            .add_attribute("name", name.clone())
            .add_attribute("plug_type", format!("{:?}", plug_type)),
    ];


    Ok(Response::new().add_events(events))
}

pub fn remove_energy_offer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    energy_offer_id: u64,
) -> Result<Response, ContractError> {

    let energy_offer = ENERGY_TRANSFER_OFFERS.load(deps.storage, energy_offer_id)
        .map_err(|_| ContractError::EnergyOfferNotFound(energy_offer_id))?;

    let sender = info.sender.to_string();
    if energy_offer.owner != sender {
        return Err(ContractError::InvalidSigner(sender));
    }

    if energy_offer.charger_status != ChargerStatus::Active && energy_offer.charger_status != ChargerStatus::Inactive {
        return Err(ContractError::InvalidChargerMultipleStatuses(
            ChargerStatus::Active,
            ChargerStatus::Inactive,
            energy_offer.charger_status,
        ))
    }

    ENERGY_TRANSFER_OFFERS.remove(deps.storage, energy_offer_id);

    let events = vec![
        Event::new("remove_energy_offer")
            .add_attribute("energy_offer_id", energy_offer_id.to_string())
            .add_attribute("owner", energy_offer.owner)
            .add_attribute("removed_id", energy_offer_id.to_string())
    ];

    Ok(Response::new() .add_events(events))
}

pub fn start_energy_transfer(
    deps: DepsMut,
    info: MessageInfo,
    driver: String,
    env: Env,
    energy_transfer_offer_id: u64,
    energy_to_transfer: u64,
) -> Result<Response, ContractError> {
    validate_start_energy_transfer(&deps, &driver, energy_to_transfer)?;

    let mut offer = ENERGY_TRANSFER_OFFERS
        .load(deps.storage, energy_transfer_offer_id)
        .map_err(|_| ContractError::EnergyOfferNotFound(energy_transfer_offer_id))?;

    if offer.charger_status != ChargerStatus::Active {
        return Err(ContractError::InvalidChargerStatus(ChargerStatus::Active, offer.charger_status));
    }

    offer.charger_status = ChargerStatus::Busy;
    ENERGY_TRANSFER_OFFERS.save(deps.storage, energy_transfer_offer_id, &offer)?;

    let mut transfer_count = ENERGY_TRANSFER_COUNT.load(deps.storage)?;
    transfer_count += 1;

    let collateral = offer.tariff * energy_to_transfer;
    let energy_transfer = EnergyTransfer {
        id: transfer_count,
        energy_transfer_offer_id,
        charger_id: offer.charger_id.clone(),
        owner: offer.owner.clone(),
        driver,
        offered_tariff: offer.tariff,
        status: TransferStatus::Requested,
        collateral,
        energy_to_transfer,
        energy_transferred: 0,
        paid_date: env.block.time,
    };
    ENERGY_TRANSFERS.save(deps.storage, transfer_count, &energy_transfer)?;
    ENERGY_TRANSFER_COUNT.save(deps.storage, &transfer_count)?;

    let denom = DENOM.load(deps.storage)?;

    let collateral_coins = coins(collateral.into(), &denom);

    if info.funds != collateral_coins {
        return Err(ContractError::InvalidFunds(format!("{:?}", collateral_coins), format!("{:?}", info)));
    }

    let events = vec![
        Event::new("start_energy_transfer")
            .add_attribute("energy_transfer_id", energy_transfer.id.to_string())
            .add_attribute("charger_id", offer.charger_id.to_string())
            .add_attribute("energy_transfer_offer_id", energy_transfer.energy_to_transfer.to_string())
            .add_attribute("new_transfer_id", energy_transfer.id.to_string()),
    ];

    Ok(Response::new().add_events(events))
}

fn validate_start_energy_transfer(
    deps: &DepsMut,
    driver: &str,
    energy_to_transfer: u64,
) -> Result<(), ContractError> {
    if driver.is_empty() {
        return Err(ContractError::InvalidDriver);
    }

    deps.api.addr_validate(driver)?;

    if energy_to_transfer == 0 {
        return Err(ContractError::ZeroEnergy);
    }
    Ok(())
}

pub(crate) fn energy_transfer_started(
    deps: DepsMut,
    _info: MessageInfo,
    energy_transfer_id: u64,
) -> Result<Response, ContractError> {
    let mut energy_transfer: EnergyTransfer = ENERGY_TRANSFERS
        .load(deps.storage, energy_transfer_id)
        .map_err(|_| ContractError::EnergyTransferNotFound(energy_transfer_id))?;

    if energy_transfer.status != TransferStatus::Requested {
        return Err(ContractError::InvalidEnergyTransferStatus(TransferStatus::Requested, energy_transfer.status,
        ));
    }

    energy_transfer.status = TransferStatus::Ongoing;

    ENERGY_TRANSFERS.save(deps.storage, energy_transfer_id, &energy_transfer)?;

    let events = vec![
        Event::new("energy_transfer_started")
            .add_attribute("energy_transfer_id", energy_transfer_id.to_string())
            .add_attribute("energy_transfer_offer_id", energy_transfer.energy_transfer_offer_id.to_string()),
    ];

    Ok(Response::new()
        .add_events(events)
    )
}

pub fn cancel_energy_transfer(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    energy_transfer_id: u64,
) -> Result<Response, ContractError> {
    let mut energy_transfer = ENERGY_TRANSFERS
        .load(deps.storage, energy_transfer_id)
        .map_err(|_| ContractError::EnergyTransferNotFound(energy_transfer_id))?;

    if energy_transfer.status != TransferStatus::Requested {
        return Err(ContractError::InvalidEnergyTransferStatus(TransferStatus::Requested, energy_transfer.status));
    }

    energy_transfer.status = TransferStatus::Cancelled;
    ENERGY_TRANSFERS.save(deps.storage, energy_transfer_id, &energy_transfer)?;

    let mut offer = ENERGY_TRANSFER_OFFERS
        .load(deps.storage, energy_transfer.energy_transfer_offer_id)
        .map_err(|_| ContractError::EnergyOfferNotFound(energy_transfer.energy_transfer_offer_id))?;

    offer.charger_status = ChargerStatus::Active;
    ENERGY_TRANSFER_OFFERS.save(deps.storage, energy_transfer.energy_transfer_offer_id, &offer)?;

    let denom = DENOM.load(deps.storage)?;
    let collateral_coins = coins(energy_transfer.collateral.into(), &denom);

    let bank_msg = BankMsg::Send {
        to_address: energy_transfer.driver.clone(),
        amount: collateral_coins,
    };

    let events = vec![
        Event::new("cancel_energy_transfer")
            .add_attribute("energy_transfer_id", energy_transfer.id.to_string())
            .add_attribute("charger_id", energy_transfer.charger_id)
            .add_attribute("status", "Cancelled"),
    ];

    Ok(Response::new()
        .add_message(bank_msg)
        .add_events(events)
    )
}

pub fn energy_transfer_completed(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    energy_transfer_id: u64,
    used_service_units: u64,
) -> Result<Response, ContractError> {
    let mut energy_transfer = ENERGY_TRANSFERS
        .load(deps.storage, energy_transfer_id)
        .map_err(|_| ContractError::EnergyTransferNotFound(energy_transfer_id))?;

    if energy_transfer.status != TransferStatus::Requested && energy_transfer.status != TransferStatus::Ongoing {
        return Err(ContractError::InvalidEnergyTransferMultipleStatuses (TransferStatus::Requested, TransferStatus::Ongoing, energy_transfer.status));
    }

    let mut amount_to_transfer_to_owner = energy_transfer.collateral;

    let mut bank_messages: Vec<BankMsg> = vec![];
    let denom = DENOM.load(deps.storage)?;

    if energy_transfer.energy_to_transfer > used_service_units {
        // Transfer the remaining collateral to the owner
        amount_to_transfer_to_owner = energy_transfer.offered_tariff * used_service_units;

        // Transfer the remaining collateral to the driver
        let amount_to_transfer_to_driver = energy_transfer.collateral - amount_to_transfer_to_owner;
        bank_messages.push(BankMsg::Send {
            to_address: energy_transfer.driver.clone(),
            amount: coins(amount_to_transfer_to_driver.into(), &denom),
        });
    }

    if amount_to_transfer_to_owner != 0 {
        bank_messages.push(BankMsg::Send {
            to_address: energy_transfer.owner.clone(),
            amount: coins(amount_to_transfer_to_owner.into(), &denom),
        });
    }


    energy_transfer.status = TransferStatus::Paid;
    energy_transfer.paid_date = env.block.time;
    energy_transfer.energy_transferred = used_service_units;
    ENERGY_TRANSFERS.save(deps.storage, energy_transfer_id, &energy_transfer)?;

    let mut offer = ENERGY_TRANSFER_OFFERS
        .load(deps.storage, energy_transfer.energy_transfer_offer_id)
        .map_err(|_| ContractError::EnergyOfferNotFound(energy_transfer.energy_transfer_offer_id))?;

    offer.charger_status = ChargerStatus::Active;
    ENERGY_TRANSFER_OFFERS.save(deps.storage, energy_transfer.energy_transfer_offer_id, &offer)?;

    let events = vec![
        Event::new("energy_transfer_completed")
            .add_attribute("energy_transfer_id", energy_transfer.id.to_string())
            .add_attribute("energy_transferred", used_service_units.to_string()),
    ];

    Ok(Response::new()
        .add_events(events)
        .add_messages(bank_messages)
    )
}
pub fn remove_energy_transfer(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    energy_transfer_id: u64,
) -> Result<Response, ContractError> {
    let energy_transfer = ENERGY_TRANSFERS
        .load(deps.storage, energy_transfer_id)
        .map_err(|_| ContractError::EnergyTransferNotFound(energy_transfer_id))?;

    if energy_transfer.status != TransferStatus::Paid && energy_transfer.status != TransferStatus::Cancelled {
        return Err(ContractError::InvalidEnergyTransferMultipleStatuses(TransferStatus::Paid, TransferStatus::Cancelled, energy_transfer.status));
    }

    ENERGY_TRANSFERS.remove(deps.storage, energy_transfer_id);

    let events = vec![
        Event::new("remove_energy_transfer")
            .add_attribute("energy_transfer_id", energy_transfer.id.to_string())
            .add_attribute("status", format!("{:?}", energy_transfer.status)),
    ];

    Ok(Response::new().add_events(events))
}

