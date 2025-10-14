
use std::fmt::format;
use abstract_std::objects::namespace::Namespace;
use abstract_std::objects::AccountId;
use abstract_std::registry::{NamespaceResponse, QueryMsg as RegistryQueryMsg};
use abstract_std::AbstractError;
use bs721::NftInfoResponse;
use btsg_account::market::hooks::{AskHookMsg, BidHookMsg, HookAction, SaleHookMsg};
use btsg_account::Metadata;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw_storage_plus::Item;
use thiserror::Error;
#[cfg(not(target_arch = "wasm32"))]
pub mod interface;

pub const CONTRACT_NAME: &str = "crates.io:account-registry-middleware";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Storage constant for the contract's ownership

const CONFIG: Item<Config> = Item::new("marketplace");

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AbstractError(#[from] AbstractError),

    #[error("AccountTokenIsCorrupted")]
    AccountTokenIsCorrupted {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("ReplyErr: {e}")]
    ReplyErr { e: String },
}
#[cw_serde]
pub struct InstantiateMsg {
    pub market: String,
    pub collection: String,
    pub account_code_id: u64,
}

#[cw_serde]
pub struct Config {
    pub market: String,
    pub account_code_id: u64,
    pub registry: Option<String>,
    pub collection: String,
    pub current_admin: String,
}
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    WithdrawPayments {
        assets: Vec<String>,
    },
    RegistryAction(abstract_std::registry::ExecuteMsg),
    UpdateConfig {
        market: Option<String>,
        registry: Option<String>,
        collection: Option<String>,
        owner: Option<String>,
    },
    AskCreatedHook(AskHookMsg),
    AskUpdatedHook(AskHookMsg),
    AskDeletedHook(AskHookMsg),
    BidCreatedHook(BidHookMsg),
    BidUpdatedHook(BidHookMsg),
    BidDeletedHook(BidHookMsg),
    SaleHook(SaleHookMsg),
}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            market: msg.market,
            registry: None,
            collection: msg.collection,
            account_code_id: msg.account_code_id,
            current_admin: info.sender.to_string(),
        },
    )?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let c: Config = CONFIG.load(deps.storage)?;
    match msg {
        ExecuteMsg::WithdrawPayments { assets } => {
            withdraw_payments(deps, info, c.current_admin, assets)
        }
        ExecuteMsg::UpdateConfig {
            market,
            registry,
            collection,
            owner,
        } => update_config(deps, info, market, registry, collection, owner),
        ExecuteMsg::AskCreatedHook(a) => p_ask(info, a, &c, HookAction::Create),
        ExecuteMsg::AskUpdatedHook(a) => p_ask(info, a, &c, HookAction::Update),
        ExecuteMsg::AskDeletedHook(a) => p_ask(info, a, &c, HookAction::Delete),
        ExecuteMsg::BidCreatedHook(b) => p_bid(deps, info, b, c, HookAction::Create),
        ExecuteMsg::BidUpdatedHook(b) => p_bid(deps, info, b, c, HookAction::Update),
        ExecuteMsg::BidDeletedHook(b) => p_bid(deps, info, b, c, HookAction::Delete),
        ExecuteMsg::SaleHook(s) => process_sale_hook(info, s, &c.market),
        ExecuteMsg::RegistryAction(execute_msg) => {
            route_registry_action(info, c.current_admin, c.registry.unwrap(), &execute_msg)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => cosmwasm_std::to_json_binary(&CONFIG.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    println!("{:#?}", msg);
    match msg.result {
        cosmwasm_std::SubMsgResult::Ok(_) => Ok(Response::default()),
        cosmwasm_std::SubMsgResult::Err(e) => return Err(ContractError::ReplyErr { e }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let then = cw2::get_contract_version(deps.storage)?;
    if then.version >= CONTRACT_VERSION.to_owned() || then.contract != CONTRACT_NAME.to_owned() {
        return Err(ContractError::Std(StdError::generic_err(
            "unable to migrate contract.",
        )));
    }
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

pub fn p_ask(
    info: MessageInfo,
    hook: AskHookMsg,
    c: &Config,
    method: HookAction,
) -> Result<Response, ContractError> {
    if &info.sender.to_string() != &c.market {
        return Err(ContractError::Unauthorized {});
    }
    let r = c
        .registry
        .as_ref()
        .expect("registry is yet to be registered");

    // set anmespace for newly minted token on framework.
    let mut res = Response::default();
    let mut msgs = match method {
        HookAction::Create => claim_namespace(&mut res, c, hook)?,

        HookAction::Update => {
            // check that namespaces are still in sync with current account owners.
            vec![]
        }
        // forgoe namespace if account token is burnt (burning is only way ask-hook invokes HookAction::Delete)
        HookAction::Delete => forgoe_namespace(&hook.ask.token_id, r)?,
    };

    Ok(res.add_messages(msgs))
}

pub fn p_bid(
    deps: DepsMut,
    info: MessageInfo,
    hook: BidHookMsg,
    config: Config,
    method: HookAction,
) -> Result<Response, ContractError> {
    if info.sender.to_string() != config.market {
        return Err(ContractError::Unauthorized {});
    }

    match method {
        HookAction::Create => {
            let namespace = deps.querier.query_wasm_smart::<NamespaceResponse>(
                config.registry.unwrap(),
                &RegistryQueryMsg::Namespace {
                    namespace: Namespace::new(&hook.bid.token_id)?,
                },
            )?;

            match namespace {
                NamespaceResponse::Claimed(namespace_info) => {
                    let token: NftInfoResponse<Metadata> = deps.querier.query_wasm_smart(
                        config.collection,
                        &bs721::Bs721QueryMsg::NftInfo {
                            token_id: hook.bid.token_id,
                        },
                    )?;

                    if token.token_uri.expect("should have this set")
                        != namespace_info.account.into_addr().to_string()
                    {
                        return Err(ContractError::AccountTokenIsCorrupted {});
                    };
                }
                NamespaceResponse::Unclaimed {} => {}
            }
        }
        // noop
        HookAction::Update => {}
        // noop
        HookAction::Delete => {}
    }

    Ok(Response::default())
}

pub fn process_sale_hook(
    info: MessageInfo,
    hook: SaleHookMsg,
    market: &String,
) -> Result<Response, ContractError> {
    if &info.sender.to_string() != market {
        return Err(ContractError::Unauthorized {});
    }
    let mut res = Response::default();
    if hook.buyer == hook.seller {
        forgoe_namespace(&hook.token_id, market)?;
        // claim_namespace(&mut res, hook.ask_id, &hook.token_id, market)?;
    }
    Ok(res)
}

pub fn route_registry_action(
    info: MessageInfo,
    admin: String,
    registry: String,
    execute_msg: &abstract_std::registry::ExecuteMsg,
) -> Result<Response, ContractError> {
    if info.sender.to_string() != admin {
        return Err(ContractError::Unauthorized {});
    }
    Ok(Response::default().add_message(WasmMsg::Execute {
        contract_addr: registry,
        msg: to_json_binary(execute_msg)?,
        funds: vec![],
    }))
}

fn claim_namespace(res: &mut Response, c: &Config, hook: AskHookMsg) -> StdResult<Vec<CosmosMsg>> {
    // let namespace = format!(
    //     "{}:{}",
    //     hook.ask.token_id.clone(),
    //     hook.ask.seller.to_string()
    // );

    Ok(vec![
        // cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Instantiate {
        //     admin: None,
        //     code_id: c.account_code_id.clone(),
        //     msg: to_json_binary(&abstract_std::account::InstantiateMsg::<Empty> {
        //         code_id: c.account_code_id,
        //         owner: Some(abstract_std::objects::gov_type::GovernanceDetails::NFT {
        //             collection_addr: c.collection.clone(),
        //             token_id: hook.ask.token_id.clone(),
        //         }),
        //         account_id: Some(AccountId::local(hook.ask.id)),
        //         authenticator: None,
        //         namespace: None,
        //         install_modules: vec![], // TODO: install USB
        //         name: Some(hook.ask.token_id.clone()),
        //         description: Some("Powered By Bitsong Account Framework".into()),
        //         link: None,
        //     })?,
        //     funds: vec![],
        //     label: "Bitsong Abstract Account".into(),
        // }),
        // cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
        //     contract_addr: c.registry.to_owned().expect("registry was not set"),
        //     msg: to_json_binary(&abstract_std::registry::ExecuteMsg::ClaimNamespace {
        //         account_id: AccountId::local(hook.ask.id),
        //         namespace: hook.ask.token_id.clone(),
        //     })?,
        //     funds: vec![],
        // }),
    ])
}

fn forgoe_namespace(token_id: &String, market: &String) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: market.to_string(),
        msg: to_json_binary(&abstract_std::registry::ExecuteMsg::ForgoNamespace {
            namespaces: vec![token_id.into()],
        })?,
        funds: vec![],
    })])
}

fn withdraw_payments(
    deps: DepsMut,
    info: MessageInfo,
    to_address: String,
    assets: Vec<String>,
) -> Result<Response, ContractError> {
    // Only allow current admin to withdraw
    if &info.sender.to_string() != &to_address {
        return Err(ContractError::Unauthorized {});
    }

    let mut amount = Vec::with_capacity(assets.len());
    for ass in assets {
        amount.push(deps.querier.query_balance(&info.sender, ass)?);
    }

    // If there are funds, send them to the admin
    let bank_msg = BankMsg::Send { to_address, amount };

    Ok(Response::new()
        .add_message(bank_msg)
        .add_attribute("action", "withdraw_payments"))
}

fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    market: Option<String>,
    registry: Option<String>,
    collection: Option<String>,
    owner: Option<String>,
) -> Result<Response, ContractError> {
    // Only allow current admin to update config
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender.to_string() != config.current_admin {
        return Err(ContractError::Unauthorized {});
    }

    // Update fields if provided
    if let Some(market) = market {
        config.market = market;
    }
    if let Some(registry) = registry {
        config.registry = Some(registry);
    }
    if let Some(collection) = collection {
        config.collection = collection;
    }
    if let Some(owner) = owner {
        config.current_admin = owner;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("updated_market", config.market.clone())
        .add_attribute("updated_registry", config.registry.unwrap()))
}
