#[cfg(not(feature = "library"))]

use btsg_account::minter::SudoParams;
use cosmwasm_std::{Addr, DepsMut, Event, Response, Uint128};

use crate::{
    state::{ACCOUNT_COLLECTION, ACCOUNT_MARKETPLACE, SUDO_PARAMS},
    ContractError,
};

pub fn sudo_update_params(
    deps: DepsMut,
    min_name_length: u32,
    max_name_length: u32,
    base_price: Uint128,
    // fair_burn_bps: u64,
) -> Result<Response, ContractError> {
    SUDO_PARAMS.save(
        deps.storage,
        &SudoParams {
            min_name_length,
            max_name_length,
            base_price,
            // fair_burn_percent: Decimal::percent(fair_burn_bps) / Uint128::from(100u128),
        },
    )?;

    Ok(Response::new().add_attribute("action", "sudo_update_params"))
}

pub fn sudo_update_name_collection(
    deps: DepsMut,
    collection: Addr,
) -> Result<Response, ContractError> {
    ACCOUNT_COLLECTION.save(deps.storage, &collection)?;

    let event = Event::new("update-account-collection").add_attribute("collection", collection);
    Ok(Response::new().add_event(event))
}

pub fn sudo_update_account_marketplace(
    deps: DepsMut,
    marketplace: Addr,
) -> Result<Response, ContractError> {
    ACCOUNT_MARKETPLACE.save(deps.storage, &marketplace)?;

    let event = Event::new("update-account-marketplace").add_attribute("marketplace", marketplace);
    Ok(Response::new().add_event(event))
}
