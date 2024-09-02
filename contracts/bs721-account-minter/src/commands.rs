use btsg_account::charge_fees;
use btsg_account::Metadata;
use btsg_account::NATIVE_DENOM;
use cosmwasm_std::{
    coin, Coin, Decimal, DepsMut, Env, Event, MessageInfo, Response, Uint128, WasmMsg,
};
use cosmwasm_std::{to_json_binary, Addr, Deps, StdResult};
use cw_utils::must_pay;

use crate::state::Config;
use crate::state::SudoParams;
use crate::{
    state::{ACCOUNT_COLLECTION, ADMIN, CONFIG, PAUSED, SUDO_PARAMS},
    ContractError,
};

pub fn execute_mint_and_list(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    account: &str,
) -> Result<Response, ContractError> {
    if PAUSED.load(deps.storage)? {
        return Err(ContractError::MintingPaused {});
    }

    let sender = &info.sender.to_string();
    let config = CONFIG.load(deps.storage)?;
    let params = SUDO_PARAMS.load(deps.storage)?;

    if env.block.time < config.public_mint_start_time {
        return Err(ContractError::MintingNotStarted {});
    }

    validate_account(
        account,
        params.min_account_length,
        params.max_account_length,
    )?;
    let price = validate_payment(account.len(), &info, params.base_price.u128())?;

    let mut res = Response::new();
    // burns any tokens sent as fees if required
    if price.is_some() {
        charge_fees(&mut res, price.clone().unwrap().amount);
    }

    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;

    // mint token
    let mint_msg = bs721_account::msg::ExecuteMsg::Mint {
        token_id: account.to_string(),
        owner: sender.to_string(),
        token_uri: None,
        extension: Metadata::default(),
        seller_fee_bps: None,
        payment_addr: None,
    };
    let mint_msg_exec = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    };

    let event = Event::new("mint-and-list")
        .add_attribute("account", account)
        .add_attribute("owner", sender)
        .add_attribute(
            "price",
            price
                .unwrap_or_else(|| coin(0u128, NATIVE_DENOM))
                .amount
                .to_string(),
        );
    Ok(res.add_event(event).add_message(mint_msg_exec))
}

/// Pause or unpause minting
pub fn execute_pause(
    deps: DepsMut,
    info: MessageInfo,
    pause: bool,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    PAUSED.save(deps.storage, &pause)?;

    let event = Event::new("pause").add_attribute("pause", pause.to_string());
    Ok(Response::new().add_event(event))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    config: Config,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let start_time = config.public_mint_start_time;

    // Can not set public mint time in the past
    if env.block.time > start_time {
        return Err(ContractError::InvalidTradingStartTime(
            env.block.time,
            start_time,
        ));
    }

    CONFIG.save(deps.storage, &config)?;

    let event = Event::new("update-config").add_attribute("address", info.sender.to_string());
    Ok(Response::new().add_event(event))
}

// This follows the same rules as Internet domain accounts
pub fn validate_account(account: &str, min: u32, max: u32) -> Result<(), ContractError> {
    let len = account.len() as u32;
    if len < min {
        return Err(ContractError::AccountTooShort {});
    } else if len >= max {
        return Err(ContractError::AccountTooLong {});
    }

    account
        .find(invalid_char)
        .map_or(Ok(()), |_| Err(ContractError::InvalidAccount {}))?;

    (if account.starts_with('-') || account.ends_with('-') {
        Err(ContractError::InvalidAccount {})
    } else {
        Ok(())
    })?;

    if len > 4u32 && account[2..4].contains("--") {
        return Err(ContractError::InvalidAccount {});
    }

    Ok(())
}

pub enum Discount {
    Percent(Decimal),
}

pub fn validate_payment(
    account_len: usize,
    info: &MessageInfo,
    base_price: u128,
    // discount: Option<Discount>,
) -> Result<Option<Coin>, ContractError> {
    // Because we know we are left with ASCII chars, a simple byte count is enough
    let amount: Uint128 = (match account_len {
        0..=2 => {
            return Err(ContractError::AccountTooShort {});
        }
        3 => base_price * 100,
        4 => base_price * 10,
        _ => base_price,
    })
    .into();

    if amount.is_zero() {
        return Ok(None);
    }

    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment != amount {
        return Err(ContractError::IncorrectPayment {
            got: payment.u128(),
            expected: amount.u128(),
        });
    }

    Ok(Some(coin(amount.u128(), NATIVE_DENOM)))
}

pub fn invalid_char(c: char) -> bool {
    let is_valid = c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-';
    !is_valid
}

pub fn query_collection(deps: Deps) -> StdResult<Addr> {
    ACCOUNT_COLLECTION.load(deps.storage)
}

pub fn query_params(deps: Deps) -> StdResult<SudoParams> {
    SUDO_PARAMS.load(deps.storage)
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn sudo_update_params(
    deps: DepsMut,
    min_account_length: u32,
    max_account_length: u32,
    base_price: Uint128,
    // fair_burn_bps: u64,
) -> Result<Response, ContractError> {
    SUDO_PARAMS.save(
        deps.storage,
        &SudoParams {
            min_account_length,
            max_account_length,
            base_price,
            // fair_burn_percent: Decimal::percent(fair_burn_bps) / Uint128::from(100u128),
        },
    )?;

    Ok(Response::new().add_attribute("action", "sudo_update_params"))
}

pub fn sudo_update_account_collection(
    deps: DepsMut,
    collection: Addr,
) -> Result<Response, ContractError> {
    ACCOUNT_COLLECTION.save(deps.storage, &collection)?;

    let event = Event::new("update-account-collection").add_attribute("collection", collection);
    Ok(Response::new().add_event(event))
}

// pub fn sudo_update_account_marketplace(
//     deps: DepsMut,
//     marketplace: Addr,
// ) -> Result<Response, ContractError> {
//     ACCOUNT_MARKETPLACE.save(deps.storage, &marketplace)?;

//     let event = Event::new("update-account-marketplace").add_attribute("marketplace", marketplace);
//     Ok(Response::new().add_event(event))
// }
