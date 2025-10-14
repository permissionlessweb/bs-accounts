use btsg_account::market::MigrateMsg;
use btsg_account::minter::{Config, SudoParams};
use cosmwasm_std::{
    instantiate2_address, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, WasmMsg,
};
use cw2::set_contract_version;

// use cw2::set_contract_version;

use crate::commands::*;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{ACCOUNT_COLLECTION, ACCOUNT_MARKETPLACE, CONFIG, PAUSED, SUDO_PARAMS};

// version info for migration info
pub const ACCOUNT_MINTER: &str = "crates.io:bs721-account-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, ACCOUNT_MINTER, CONTRACT_VERSION)?;

    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(msg.admin.unwrap_or(info.sender.to_string()).as_str()),
    )?;

    PAUSED.save(deps.storage, &false)?;

    let marketplace = deps.api.addr_validate(&msg.marketplace_addr)?;
    ACCOUNT_MARKETPLACE.save(deps.storage, &marketplace)?;

    SUDO_PARAMS.save(
        deps.storage,
        &SudoParams {
            min_account_length: msg.min_account_length,
            max_account_length: msg.max_account_length,
            base_price: msg.base_price,
            base_delegation: msg.base_delegation,
        },
    )?;

    CONFIG.save(
        deps.storage,
        &Config {
            public_mint_start_time: env
                .block
                .time
                .plus_seconds(msg.mint_start_delay.unwrap_or(1)),
        },
    )?;

    let account_collection_init_msg = bs721_account::msg::InstantiateMsg {
        verifier: msg.verifier,
        marketplace: deps.api.addr_validate(&msg.marketplace_addr)?,
        base_init_msg: bs721_base::InstantiateMsg {
            name: "Bitsong Account Tokens".to_string(),
            symbol: "ACCOUNTS".to_string(),
            uri: None,
            minter: env.contract.address.to_string(),
        },
    };
    let salt = &env.block.height.to_be_bytes();
    let code_info = deps.querier.query_wasm_code_info(msg.collection_code_id)?;

    let addr = instantiate2_address(
        code_info.checksum.as_slice(),
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        salt,
    )?;

    let wasm_msg = WasmMsg::Instantiate2 {
        code_id: msg.collection_code_id,
        msg: to_json_binary(&account_collection_init_msg)?,
        funds: info.funds,
        admin: Some(info.sender.to_string()),
        label: "Account Collection".to_string(),
        salt: salt.into(),
    };
    ACCOUNT_COLLECTION.save(deps.storage, &deps.api.addr_humanize(&addr)?)?;

    Ok(Response::new()
        .add_message(wasm_msg)
        .add_attribute("action", "instantiate")
        .add_attribute("account_minter_addr", env.contract.address.to_string()))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintAndList { account } => {
            execute_mint_and_list(deps, info, env, account.trim())
        }
        ExecuteMsg::UpdateOwnership(action) => execute_update_owner(deps, info, env, action),
        ExecuteMsg::Pause { pause } => execute_pause(deps, info, pause),
        ExecuteMsg::UpdateConfig { config } => execute_update_config(deps, info, env, config),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
        QueryMsg::Collection {} => to_json_binary(&query_collection(deps)?),
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        SudoMsg::UpdateParams {
            min_account_length,
            max_account_length,
            base_price,
            base_delegation,
            // fair_burn_bps,
        } => sudo_update_params(
            deps,
            min_account_length,
            max_account_length,
            base_price,
            base_delegation,
        ),
        SudoMsg::UpdateAccountCollection { collection } => {
            sudo_update_account_collection(deps, api.addr_validate(&collection)?)
        }
        SudoMsg::UpdateAccountMarketplace { marketplace } => {
            sudo_update_account_marketplace(deps, api.addr_validate(&marketplace)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let then = cw2::get_contract_version(deps.storage)?;
    if then.version >= CONTRACT_VERSION.to_owned() || then.contract != ACCOUNT_MINTER.to_owned() {
        return Err(ContractError::Std(StdError::generic_err(
            "unable to migrate bs721-account minter.",
        )));
    }
    cw2::set_contract_version(deps.storage, ACCOUNT_MINTER, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use btsg_account::CURRENT_BASE_PRICE;
    use cosmwasm_std::{coin, Addr, MessageInfo};

    use crate::commands::{validate_account, validate_payment};

    #[test]
    fn check_validate_account() {
        let min = 3;
        let max = 63;
        assert!(validate_account("bobo", min, max).is_ok());
        assert!(validate_account("-bobo", min, max).is_err());
        assert!(validate_account("bobo-", min, max).is_err());
        assert!(validate_account("bo-bo", min, max).is_ok());
        assert!(validate_account("bo--bo", min, max).is_err());
        assert!(validate_account("bob--o", min, max).is_ok());
        assert!(validate_account("bo", min, max).is_err());
        assert!(validate_account("b", min, max).is_err());
        assert!(validate_account("bob", min, max).is_ok());
        assert!(validate_account(
            "bobobobobobobobobobobobobobobobobobobobobobobobobobobobobobobo",
            min,
            max
        )
        .is_ok());
        assert!(validate_account(
            "bobobobobobobobobobobobobobobobobobobobobobobobobobobobobobobob",
            min,
            max
        )
        .is_err());
        assert!(validate_account("0123456789", min, max).is_ok());
        assert!(validate_account("😬", min, max).is_err());
        assert!(validate_account("BOBO", min, max).is_err());
        assert!(validate_account("b-o----b", min, max).is_ok());
        assert!(validate_account("bobo.stars", min, max).is_err());
    }

    #[test]
    fn check_validate_payment() {
        let base_price = CURRENT_BASE_PRICE as u128;

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price, "ubtsg")],
        };
        assert_eq!(
            validate_payment(5, &info, base_price)
                .unwrap()
                .unwrap()
                .amount
                .u128(),
            base_price
        );

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price * 10, "ubtsg")],
        };
        assert_eq!(
            validate_payment(4, &info, base_price)
                .unwrap()
                .unwrap()
                .amount
                .u128(),
            base_price * 10
        );

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price * 100, "ubtsg")],
        };
        assert_eq!(
            validate_payment(3, &info, base_price)
                .unwrap()
                .unwrap()
                .amount
                .u128(),
            base_price * 100
        );
    }
}
