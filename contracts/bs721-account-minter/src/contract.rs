#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{maybe_addr, parse_reply_instantiate_data};
// use cw2::set_contract_version;

use crate::commands::*;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{
    Config, SudoParams, ACCOUNT_COLLECTION, ACCOUNT_MARKETPLACE, ADMIN, CONFIG, PAUSED, SUDO_PARAMS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:bs721-account-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        },
    )?;

    CONFIG.save(
        deps.storage,
        &Config {
            public_mint_start_time: env.block.time.plus_seconds(1),
        },
    )?;

    let account_collection_init_msg = bs721_account::msg::InstantiateMsg {
        verifier: msg.verifier,
        base_init_msg: bs721_base::msg::InstantiateMsg {
            name: "Bitsong Account Tokens".to_string(),
            symbol: "ACCOUNTS".to_string(),
            minter: env.contract.address.to_string(),
            uri: None,
        },
    };

    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.collection_code_id,
        msg: to_json_binary(&account_collection_init_msg)?,
        funds: info.funds,
        admin: Some(info.sender.to_string()),
        label: "Account Collection".to_string(),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, INIT_COLLECTION_REPLY_ID);

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("account_minter_addr", env.contract.address.to_string())
        .add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::MintAndList { account } => {
            execute_mint_and_list(deps, info, env, account.trim())
        }
        ExecuteMsg::UpdateAdmin { admin } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, admin)?)?)
        }
        ExecuteMsg::Pause { pause } => execute_pause(deps, info, pause),
        ExecuteMsg::UpdateConfig { config } => execute_update_config(deps, info, env, config),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Collection {} => to_json_binary(&query_collection(deps)?),
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

/// Mint a account for the sender, or `contract` if specified
const INIT_COLLECTION_REPLY_ID: u64 = 1;
#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INIT_COLLECTION_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);

    println!("REPLYMSG{:#?}", reply);

    match reply {
        Ok(res) => {
            let collection_address = &res.contract_address;

            ACCOUNT_COLLECTION.save(deps.storage, &Addr::unchecked(collection_address))?;

            // let msg = WasmMsg::Execute {
            //     contract_addr: collection_address.to_string(),
            //     funds: vec![],
            //     msg: to_json_binary(
            //         &(bs721_account::msg::ExecuteMsg::<Metadata>::SetMarketplace {
            //             address: ACCOUNT_MARKETPLACE.load(deps.storage)?.to_string(),
            //         }),
            //     )?,
            // };

            Ok(Response::default().add_attribute("action", "init_collection_reply"))
        }
        Err(_) => Err(ContractError::ReplyOnSuccess {}),
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
            // fair_burn_bps,
        } => sudo_update_params(
            deps,
            min_account_length,
            max_account_length,
            base_price,
            // fair_burn_bps,
        ),
        SudoMsg::UpdateAccountCollection { collection } => {
            sudo_update_account_collection(deps, api.addr_validate(&collection)?)
        }
        SudoMsg::UpdateAccountMarketplace { marketplace } => {
            sudo_update_account_marketplace(deps, api.addr_validate(&marketplace)?)
        }
    }
}

#[cfg(test)]
mod tests {}
