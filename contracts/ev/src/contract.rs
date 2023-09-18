#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, to_binary};
use cw2::{set_contract_version, get_contract_version};
use crate::{execute, query};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{DENOM, ENERGY_TRANSFER_COUNT, ENERGY_TRANSFER_OFFER_COUNT};
use semver::Version;
const CONTRACT_NAME: &str = "crates.io:ev";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    DENOM.save(deps.storage, &msg.denom)?;
    ENERGY_TRANSFER_OFFER_COUNT.save(deps.storage, &0u64)?;
    ENERGY_TRANSFER_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PublishEnergyTransferOffer {
            charger_id,
            location,
            tariff,
            name,
            plug_type,
        } => execute::publish_energy_transfer_offer(deps, env, info, charger_id, location, tariff, name, plug_type),
        ExecuteMsg::RemoveEnergyOffer {
            energy_offer_id
        } => execute::remove_energy_offer(deps, env, info, energy_offer_id),
        ExecuteMsg::StartEnergyTransfer {
            driver,
            energy_transfer_offer_id,
            energy_to_transfer,
        } => execute::start_energy_transfer(
            deps,
            info,
            driver,
            env,
            energy_transfer_offer_id,
            energy_to_transfer,
        ),
        ExecuteMsg::EnergyTransferStarted {
            energy_transfer_id
        } => execute::energy_transfer_started(deps, info, energy_transfer_id),
        ExecuteMsg::EnergyTransferCompleted {
            energy_transfer_id,
            used_service_units,
        } => execute::energy_transfer_completed(deps, env, info, energy_transfer_id, used_service_units),
        ExecuteMsg::CancelEnergyTransfer { energy_transfer_id } => {
            execute::cancel_energy_transfer(deps, env, info, energy_transfer_id)
        },
        ExecuteMsg::RemoveEnergyTransfer { energy_transfer_id } => {
            execute::remove_energy_transfer(deps, env, info, energy_transfer_id)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllEnergyTransferOffers {} => to_binary(&query::query_all_energy_transfer_offers(deps)?),
        QueryMsg::EnergyTransfer { id } => to_binary(&query::query_energy_transfer(deps, id)?),
        QueryMsg::AllEnergyTransfers {} => to_binary(&query::query_all_energy_transfers(deps)?),
        QueryMsg::EnergyTransferOffers { owner } => to_binary(&query::query_energy_transfer_offers(deps, owner)?),
        QueryMsg::OwnEnergyTransfers { driver, transfer_status } => to_binary(&query::query_own_energy_transfers(deps, driver, transfer_status)?),
        QueryMsg::EnergyTransfers { owner } => to_binary(&query::query_energy_transfers(deps, owner)?),
        QueryMsg::Denom {} => to_binary(&query::query_denom(deps)?),
        QueryMsg::EnergyTransferOffer { id } => to_binary(&query::query_energy_transfer_offer(deps, id)?),
    }
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // With `Response` type, it is still possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    todo!()
}


/// Handling contract migration
/// To make a contract migratable, you need
/// - this entry_point implemented
/// - only contract admin can migrate, so admin has to be set at contract initiation time
/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    }
    Ok(Response::default())
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{Addr, coins};
    use cw_multi_test::{App, BasicApp, ContractWrapper, Executor};
    use crate::msg::{AllEnergyTransferOffersResponse, AllEnergyTransfersResponse, ChargerStatus, DenomResponse, EnergyTransferOfferResponse, EnergyTransferOffersByOwnerResponse, EnergyTransferResponse, Location, PlugType, TransferStatus};

    static DRIVER_ADDRESS: &str = "c4e1n65nctlr97na2h9sjul94ge4y95uhtxwmhn9kx";
    static CONTRACT_CREATOR_ADDRESS: &str = "c4e185qx6dnqry2d3crk24u3h3vtfzkqscvuvympam";
    static CONNECTOR_ADDRESS: &str = "c4e15dwxa9jq7mjv3kpw3qxgx4v7asmh3yqh3zre47";
    static OWNER_ADDRESS: &str = "c4e1lt5npfrl4fnvkxm387d8fc59x3vwugagm4vnzm";

    fn setup_app() -> (App, Addr) {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(CONTRACT_CREATOR_ADDRESS), coins(10000, "uc4e"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &Addr::unchecked(DRIVER_ADDRESS), coins(10000, "uc4e"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &Addr::unchecked(OWNER_ADDRESS), coins(10000, "uc4e"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &Addr::unchecked(CONNECTOR_ADDRESS), coins(10000, "uc4e"))
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(CONTRACT_CREATOR_ADDRESS),
                &InstantiateMsg { denom: "uc4e".to_string() },
                &[],
                "Contract",
                None,
            )
            .unwrap();
        (app, addr)
    }

    fn execute_publish_offer(app: &mut App, addr: Addr) {
        let res = app.execute_contract(
            Addr::unchecked(OWNER_ADDRESS),
            addr,
            &ExecuteMsg::PublishEnergyTransferOffer {
                charger_id: "charger1".to_string(),
                location: Location { latitude: "60".to_string(), longitude: "60".to_string() },
                tariff: 50,
                name: "offer1".to_string(),
                plug_type: PlugType::Type1,
            },
            &[],
        );
        assert!(res.is_ok());
    }

    fn execute_start_energy_transfer(
        app: &mut App,
        addr: Addr,
        driver: String,
        energy_transfer_offer_id: u64,
        energy_to_transfer: u64,
    ) {
        let res = app.execute_contract(
            Addr::unchecked(driver.clone()),
            addr.clone(),
            &ExecuteMsg::StartEnergyTransfer {
                energy_transfer_offer_id,
                energy_to_transfer,
                driver,
            },
            &coins(500, "uc4e"),
        );
        assert!(res.is_ok());
    }

    fn execute_cancel_energy_transfer(
        app: &mut App,
        addr: Addr,
        user_address: String,
        energy_transfer_id: u64,
        energy_transfer_offer_id: u64,
    ) {
        let res = app.execute_contract(
            Addr::unchecked(user_address),
            addr.clone(),
            &ExecuteMsg::CancelEnergyTransfer {
                energy_transfer_id
            },
            &[],
        );
        assert!(res.is_ok());

        let query_res: EnergyTransferResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::EnergyTransfer { id: energy_transfer_id })
            .unwrap();

        assert_eq!(query_res.energy_transfer.status, TransferStatus::Cancelled);

        let query_res: EnergyTransferOfferResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::EnergyTransferOffer { id: energy_transfer_offer_id })
            .unwrap();

        assert_eq!(query_res.energy_transfer_offer.charger_status, ChargerStatus::Active);
    }

    pub fn execute_remove_energy_transfer(
        app: &mut App,
        addr: Addr,
        energy_transfer_id: u64,
    ) {
        let res = app.execute_contract(
            Addr::unchecked(CONNECTOR_ADDRESS),
            addr.clone(),
            &ExecuteMsg::RemoveEnergyTransfer {
                energy_transfer_id
            },
            &[],
        );
        assert!(res.is_ok());

        let query_res: Result<EnergyTransferResponse, cosmwasm_std::StdError> = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::EnergyTransfer { id: energy_transfer_id });

        match query_res {
            Ok(query_res) => assert_eq!(query_res.energy_transfer.id, 0),
            Err(err) => assert_eq!(err.to_string(), "Generic error: Querier contract error: ev::msg::EnergyTransfer not found"),
        }
    }

    pub fn execute_energy_transfer_started(app: &mut App, addr: Addr, energy_transfer_id: u64) {
        let res = app.execute_contract(
            Addr::unchecked(CONNECTOR_ADDRESS),
            addr.clone(),
            &ExecuteMsg::EnergyTransferStarted {
                energy_transfer_id
            },
            &[],
        );
        assert!(res.is_ok());

        let query_res: EnergyTransferResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::EnergyTransfer { id: energy_transfer_id })
            .unwrap();

        assert_eq!(query_res.energy_transfer.status, TransferStatus::Ongoing);
    }

    pub fn execute_energy_transfer_completed(
        app: &mut App,
        addr: Addr,
        energy_transfer_id: u64,
        used_service_units: u64,
        owner: String,
        driver: String,
        expected_owner_balance: u128,
        expected_driver_balance: u128,
    ) {
        let res = app.execute_contract(
            Addr::unchecked(CONNECTOR_ADDRESS),
            addr.clone(),
            &ExecuteMsg::EnergyTransferCompleted {
                energy_transfer_id,
                used_service_units,
            },
            &[],
        );
        assert!(res.is_ok());

        let query_res: EnergyTransferResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::EnergyTransfer { id: energy_transfer_id })
            .unwrap();

        assert_eq!(query_res.energy_transfer.status, TransferStatus::Paid);

        assert_eq!(
            app.wrap()
                .query_balance(owner, "uc4e")
                .unwrap()
                .amount
                .u128(),
            expected_owner_balance
        );

        assert_eq!(
            app.wrap()
                .query_balance(driver, "uc4e")
                .unwrap()
                .amount
                .u128(),
            expected_driver_balance
        );
    }

    pub fn query_all_transfers(app: &App, addr: Addr, expected_len: usize) {
        let resp: AllEnergyTransfersResponse = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AllEnergyTransfers {})
            .unwrap();
        assert_eq!(resp.energy_transfers.len(), expected_len);
    }

    #[test]
    fn test_valid_start_energy_transfer() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());

        execute_start_energy_transfer(
            &mut app,
            addr.clone(),
            DRIVER_ADDRESS.to_string(),
            1,
            10,
        );

        let query_res: EnergyTransferResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::EnergyTransfer { id: 1 })
            .unwrap();

        assert_eq!(query_res.energy_transfer.id, 1);
        assert_eq!(query_res.energy_transfer.offered_tariff, 50);
        assert_eq!(query_res.energy_transfer.energy_to_transfer, 10);
        assert_eq!(query_res.energy_transfer.status, TransferStatus::Requested);

        query_all_transfers(&app, addr, 1);
    }

    #[test]
    fn test_valid_energy_transfer_started() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());

        execute_start_energy_transfer(
            &mut app,
            addr.clone(),
            DRIVER_ADDRESS.to_string(),
            1,
            10,
        );

        execute_energy_transfer_started(&mut app, addr.clone(), 1);

        let err = app.execute_contract(
            Addr::unchecked("creator"),
            addr.clone(),
            &ExecuteMsg::EnergyTransferStarted {
                energy_transfer_id: 1
            },
            &[],
        ).unwrap_err();

        assert_eq!(
            ContractError::InvalidEnergyTransferStatus (TransferStatus::Requested, TransferStatus::Ongoing),
            err.downcast().unwrap()
        );
    }

    #[test]
    fn test_valid_energy_transfer_completed() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());

        execute_start_energy_transfer(
            &mut app,
            addr.clone(),
            DRIVER_ADDRESS.to_string(),
            1,
            10,
        );

        execute_energy_transfer_started(&mut app, addr.clone(), 1);

        execute_energy_transfer_completed(
            &mut app,
            addr.clone(),
            1,
            10,
            OWNER_ADDRESS.to_string(),
            DRIVER_ADDRESS.to_string(),
            10500,
            9500,
        );
    }

    #[test]
    fn test_valid_energy_transfer_completed_half() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());

        execute_start_energy_transfer(
            &mut app,
            addr.clone(),
            DRIVER_ADDRESS.to_string(),
            1,
            10,
        );

        execute_energy_transfer_started(&mut app, addr.clone(), 1);

        execute_energy_transfer_completed(
            &mut app,
            addr.clone(),
            1,
            5,
            OWNER_ADDRESS.to_string(),
            DRIVER_ADDRESS.to_string(),
            10250,
            9750,
        );
    }

    #[test]
    fn test_valid_energy_transfer_completed_none() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());

        execute_start_energy_transfer(
            &mut app,
            addr.clone(),
            DRIVER_ADDRESS.to_string(),
            1,
            10,
        );

        execute_energy_transfer_started(&mut app, addr.clone(), 1);

        execute_energy_transfer_completed(
            &mut app,
            addr.clone(),
            1,
            0,
            OWNER_ADDRESS.to_string(),
            DRIVER_ADDRESS.to_string(),
            10000,
            10000,
        );
    }

    #[test]
    fn test_valid_energy_transfer_completed_and_remove() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());

        execute_start_energy_transfer(
            &mut app,
            addr.clone(),
            DRIVER_ADDRESS.to_string(),
            1,
            10,
        );

        execute_energy_transfer_started(&mut app, addr.clone(), 1);

        execute_energy_transfer_completed(
            &mut app,
            addr.clone(),
            1,
            5,
            OWNER_ADDRESS.to_string(),
            DRIVER_ADDRESS.to_string(),
            10250,
            9750,
        );

        execute_remove_energy_transfer(
            &mut app,
            addr.clone(),
            1,
        );
    }

    #[test]
    fn test_valid_cancel_energy_transfer() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());

        execute_start_energy_transfer(
            &mut app,
            addr.clone(),
            DRIVER_ADDRESS.to_string(),
            1,
            10,
        );

        query_all_transfers(&app, addr.clone(), 1);

        execute_cancel_energy_transfer(
            &mut app,
            addr.clone(),
            DRIVER_ADDRESS.to_string(),
            1,
            1,
        );

        let err = app.execute_contract(
            Addr::unchecked(CONNECTOR_ADDRESS),
            addr.clone(),
            &ExecuteMsg::CancelEnergyTransfer {
                energy_transfer_id: 1,
            },
            &[],
        ).unwrap_err();

        assert_eq!(
            ContractError::InvalidEnergyTransferStatus (TransferStatus::Requested, TransferStatus::Cancelled),
            err.downcast().unwrap()
        );
    }


    pub fn query_all_offers(app: &BasicApp, addr: Addr, expected_len: usize) {
        let resp: AllEnergyTransferOffersResponse = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AllEnergyTransferOffers {})
            .unwrap();

        assert_eq!(resp.energy_transfer_offers.len(), expected_len);
    }

    #[test]
    fn test_publish_offer_and_query() {
        let (mut app, addr) = setup_app();
        validate_denom(&app, addr.clone(), "uc4e".to_string());

        execute_publish_offer(&mut app, addr.clone());

        let query_res: EnergyTransferOfferResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::EnergyTransferOffer { id: 1 })
            .unwrap();

        assert_eq!(query_res.energy_transfer_offer.id, 1);
        assert_eq!(query_res.energy_transfer_offer.charger_id, "charger1");
        assert_eq!(query_res.energy_transfer_offer.tariff, 50);
        assert_eq!(query_res.energy_transfer_offer.name, "offer1");
    }

    #[test]
    fn test_query_all_offers() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());
        execute_publish_offer(&mut app, addr.clone());

        query_all_offers(&app, addr.clone(), 2);
    }

    #[test]
    fn test_query_offers_by_owner() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());

        let query_res: EnergyTransferOffersByOwnerResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::EnergyTransferOffers { owner: OWNER_ADDRESS.to_string() })
            .unwrap();

        assert_eq!(query_res.energy_transfer_offers.len(), 1);
    }

    #[test]
    fn test_publish_wrong_energy_offer() {
        let (mut app, addr) = setup_app();

        // Erroneous case for PublishEnergyTransferOffer with empty charger_id
        let err = app.execute_contract(
            Addr::unchecked("creator"),
            addr.clone(),
            &ExecuteMsg::PublishEnergyTransferOffer {
                charger_id: "".to_string(),
                location: Location { latitude: "60".to_string(), longitude: "60".to_string() },
                tariff: 50,
                name: "offer2".to_string(),
                plug_type: PlugType::Type1,
            },
            &[],
        ).unwrap_err();

        assert_eq!(
            ContractError::CustomError { val: "charger_id and name must not be empty".to_string() },
            err.downcast().unwrap()
        );

        // Erroneous case for PublishEnergyTransferOffer with empty name
        let err = app.execute_contract(
            Addr::unchecked("creator"),
            addr.clone(),
            &ExecuteMsg::PublishEnergyTransferOffer {
                charger_id: "charger2".to_string(),
                location: Location { latitude: "60".to_string(), longitude: "60".to_string() },
                tariff: 0,
                name: "".to_string(),
                plug_type: PlugType::Type1,
            },
            &[],
        ).unwrap_err();

        assert_eq!(
            ContractError::CustomError { val: "charger_id and name must not be empty".to_string() },
            err.downcast().unwrap()
        );
    }

    #[test]
    fn test_remove_energy_offer() {
        let (mut app, addr) = setup_app();

        execute_publish_offer(&mut app, addr.clone());

        // Erroneous case for RemoveEnergyOffer with wrong energy_offer_id
        let err = app.execute_contract(
            Addr::unchecked("creator"),
            addr.clone(),
            &ExecuteMsg::RemoveEnergyOffer {
                energy_offer_id: 10,
            },
            &[],
        ).unwrap_err();

        assert_eq!(
            ContractError::EnergyOfferNotFound(10),
            err.downcast().unwrap()
        );

        // Erroneous case for RemoveEnergyOffer with wrong owner
        let err = app.execute_contract(
            Addr::unchecked("creator2"),
            addr.clone(),
            &ExecuteMsg::RemoveEnergyOffer {
                energy_offer_id: 1,
            },
            &[],
        ).unwrap_err();

        assert_eq!(
            ContractError::InvalidSigner("creator2".to_string()),
            err.downcast().unwrap()
        );

        // Correct case for RemoveEnergyOffer
        let res = app.execute_contract(
            Addr::unchecked(OWNER_ADDRESS),
            addr.clone(),
            &ExecuteMsg::RemoveEnergyOffer {
                energy_offer_id: 1,
            },
            &[],
        );
        assert!(res.is_ok());

        let query_res_result: Result<EnergyTransferOfferResponse, cosmwasm_std::StdError> = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::EnergyTransferOffer { id: 1 });

        match query_res_result {
            Ok(query_res) => assert_eq!(query_res.energy_transfer_offer.id, 0),
            Err(err) => assert_eq!(err.to_string(), "Generic error: Querier contract error: ev::msg::EnergyTransferOffer not found"),
        }
    }

    pub fn validate_denom(app: &BasicApp, addr: Addr, denom: String) {
        let resp: DenomResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::Denom {})
            .unwrap();

        assert_eq!(
            resp,
            DenomResponse {
                denom,
            }
        );
    }
}
