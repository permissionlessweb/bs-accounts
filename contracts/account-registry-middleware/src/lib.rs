use abstract_std::objects::namespace::Namespace;
use abstract_std::objects::AccountId;
use abstract_std::registry::{NamespaceResponse, QueryMsg as RegistryQueryMsg};
use abstract_std::AbstractError;
use bs721::NftInfoResponse;
use btsg_account::market::{AskHookMsg, BidHookMsg, HookAction, SaleHookMsg};
use btsg_account::Metadata;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, SubMsg, WasmMsg,
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
}
#[cw_serde]
pub struct InstantiateMsg {
    pub market: String,
    pub collection: String,
}

#[cw_serde]
pub struct Config {
    pub market: String,
    pub registry: Option<String>,
    pub collection: String,
    pub current_admin: String,
}
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    WithdrawPayments {},
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
        ExecuteMsg::WithdrawPayments {} => withdraw_payments(deps, info, c),
        ExecuteMsg::UpdateConfig {
            market,
            registry,
            collection,
            owner,
        } => update_config(deps, info, market, registry, collection, owner),
        ExecuteMsg::AskCreatedHook(a) => p_ask(info, a, &c.market, HookAction::Create),
        ExecuteMsg::AskUpdatedHook(a) => p_ask(info, a, &c.market, HookAction::Update),
        ExecuteMsg::AskDeletedHook(a) => p_ask(info, a, &c.market, HookAction::Delete),
        ExecuteMsg::BidCreatedHook(b) => p_bid(deps, info, b, c, HookAction::Create),
        ExecuteMsg::BidUpdatedHook(b) => p_bid(deps, info, b, c, HookAction::Update),
        ExecuteMsg::BidDeletedHook(b) => p_bid(deps, info, b, c, HookAction::Delete),
        ExecuteMsg::SaleHook(s) => process_sale_hook(info, s, &c.market),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => cosmwasm_std::to_json_binary(&CONFIG.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: Deps, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn p_ask(
    info: MessageInfo,
    hook: AskHookMsg,
    market: &String,
    method: HookAction,
) -> Result<Response, ContractError> {
    if &info.sender.to_string() != market {
        return Err(ContractError::Unauthorized {});
    }
    // set anmespace for newly minted token on framework.
    let mut res = Response::default();
    match method {
        HookAction::Create => claim_namespace(&mut res, hook.ask.id, &hook.ask.token_id, market)?,
        // noop
        HookAction::Update => {}
        HookAction::Delete => forgoe_namespace(&mut res, &hook.ask.token_id, market)?,
    }

    Ok(res)
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
        forgoe_namespace(&mut res, &hook.token_id, market)?;
        claim_namespace(&mut res, hook.ask_id, &hook.token_id, market)?;
    }
    Ok(res)
}

fn claim_namespace(
    res: &mut Response,
    seq: u32,
    token_id: &String,
    market: &String,
) -> StdResult<()> {
    res.messages.push(SubMsg::new(WasmMsg::Execute {
        contract_addr: market.to_string(),
        msg: to_json_binary(&abstract_std::registry::ExecuteMsg::ClaimNamespace {
            account_id: AccountId::local(seq),
            namespace: token_id.into(),
        })?,
        funds: vec![],
    }));
    Ok(())
}

fn forgoe_namespace(res: &mut Response, token_id: &String, market: &String) -> StdResult<()> {
    res.messages.push(SubMsg::new(WasmMsg::Execute {
        contract_addr: market.to_string(),
        msg: to_json_binary(&abstract_std::registry::ExecuteMsg::ForgoNamespace {
            namespaces: vec![token_id.into()],
        })?,
        funds: vec![],
    }));
    Ok(())
}

fn withdraw_payments(
    deps: DepsMut,
    info: MessageInfo,
    config: Config,
) -> Result<Response, ContractError> {
    // Only allow current admin to withdraw
    if &info.sender.to_string() != &config.current_admin {
        return Err(ContractError::Unauthorized {});
    }

    // Query the contract's balance
    let balance = deps.querier.query_balance(&info.sender, "ubtsg")?;

    // If there are funds, send them to the admin
    let bank_msg = BankMsg::Send {
        to_address: config.current_admin.to_string(),
        amount: vec![balance.clone()],
    };

    Ok(Response::new()
        .add_message(bank_msg)
        .add_attribute("action", "withdraw_payments")
        .add_attribute("to", config.current_admin)
        .add_attribute("amount", format!("{:?}", balance)))
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
