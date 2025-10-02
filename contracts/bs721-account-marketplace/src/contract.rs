use cosmwasm_std::{
    to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::{commands::*, state::*, ContractError};
use btsg_account::market::{
    ExecuteMsg, MarketplaceInstantiateMsg, ParamInfo, QueryMsg, SudoMsg, SudoParams,
};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ACCOUNT_MARKETPLACE: &str = "crates.io:bs721-account-marketplace";

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: MarketplaceInstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, ACCOUNT_MARKETPLACE, CONTRACT_VERSION)?;
    if msg.trading_fee_bps > MAX_FEE_BPS {
        return Err(ContractError::InvalidTradingFeeBps(msg.trading_fee_bps));
    }

    let params = SudoParams {
        trading_fee_percent: Decimal::percent(msg.trading_fee_bps) / Uint128::from(100u128),
        min_price: msg.min_price,
        ask_interval: msg.ask_interval,
        valid_bid_query_limit: msg.valid_bid_query_limit,
        cooldown_duration: msg.cooldown_timeframe,
        cooldown_fee: msg.cooldown_cancel_fee,
    };

    SUDO_PARAMS.save(deps.storage, &params)?;
    IS_SETUP.save(deps.storage, &false)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
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
        ExecuteMsg::Setup { minter, collection } => execute_setup(
            deps,
            api.addr_validate(&minter)?,
            api.addr_validate(&collection)?,
        ),
        ExecuteMsg::SetAsk { token_id, seller } => {
            execute_set_ask(deps, env, info, &token_id, api.addr_validate(&seller)?)
        }
        ExecuteMsg::RemoveAsk { token_id } => execute_remove_ask(deps, info, &token_id),
        ExecuteMsg::UpdateAsk { token_id, seller } => {
            execute_update_ask(deps, info, &token_id, api.addr_validate(&seller)?)
        }
        ExecuteMsg::SetBid { token_id } => execute_set_bid(deps, env, info, &token_id),
        ExecuteMsg::RemoveBid { token_id } => execute_remove_bid(deps, env, info, &token_id),
        ExecuteMsg::AcceptBid { token_id, bidder } => {
            execute_accept_bid(deps, env, info, &token_id, api.addr_validate(&bidder)?)
        }
        ExecuteMsg::FinalizeBid { token_id } => execute_finalize_bid(deps, env, info, &token_id),
        ExecuteMsg::CancelCooldown { token_id } => {
            execute_cancel_cooldown(deps, env, info, &token_id)
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Ask { token_id } => to_json_binary(&query_ask(deps, token_id)?),
        QueryMsg::Asks { start_after, limit } => {
            to_json_binary(&query_asks(deps, start_after, limit)?)
        }
        QueryMsg::AsksBySeller {
            seller,
            start_after,
            limit,
        } => to_json_binary(&query_asks_by_seller(
            deps,
            api.addr_validate(&seller)?,
            start_after,
            limit,
        )?),
        QueryMsg::AskCount {} => to_json_binary(&query_ask_count(deps)?),
        QueryMsg::Bid { token_id, bidder } => {
            to_json_binary(&query_bid(deps, token_id, api.addr_validate(&bidder)?)?)
        }
        QueryMsg::Bids {
            token_id,
            start_after,
            limit,
        } => to_json_binary(&query_bids(deps, token_id, start_after, limit)?),
        QueryMsg::BidsByBidder {
            bidder,
            start_after,
            limit,
        } => to_json_binary(&query_bids_by_bidder(
            deps,
            api.addr_validate(&bidder)?,
            start_after,
            limit,
        )?),
        QueryMsg::BidsSortedByPrice { start_after, limit } => {
            to_json_binary(&query_bids_sorted_by_price(deps, start_after, limit)?)
        }
        QueryMsg::ReverseBidsSortedByPrice {
            start_before,
            limit,
        } => to_json_binary(&reverse_query_bids_sorted_by_price(
            deps,
            start_before,
            limit,
        )?),
        QueryMsg::BidsForSeller {
            seller,
            start_after,
            limit,
        } => to_json_binary(&query_bids_for_seller(
            deps,
            api.addr_validate(&seller)?,
            start_after,
            limit,
        )?),
        QueryMsg::HighestBid { token_id } => to_json_binary(&query_highest_bid(deps, token_id)?),
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::AskHooks {} => to_json_binary(&ASK_HOOKS.query_hooks(deps)?),
        QueryMsg::BidHooks {} => to_json_binary(&BID_HOOKS.query_hooks(deps)?),
        QueryMsg::SaleHooks {} => to_json_binary(&SALE_HOOKS.query_hooks(deps)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Cooldown { token_id } => {
            to_json_binary(&cooldown_bids().may_load(deps.storage, &ask_key(&token_id))?)
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        SudoMsg::UpdateParams {
            trading_fee_bps,
            min_price,
            ask_interval,
            cooldown_duration,
            cooldown_cancel_fee,
        } => sudo_update_params(
            deps,
            env,
            ParamInfo {
                trading_fee_bps,
                min_price,
                ask_interval,
                cooldown_duration,
                cooldown_cancel_fee,
            },
        ),
        SudoMsg::AddSaleHook { hook } => sudo_add_sale_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::AddAskHook { hook } => sudo_add_ask_hook(deps, env, api.addr_validate(&hook)?),
        SudoMsg::AddBidHook { hook } => sudo_add_bid_hook(deps, env, api.addr_validate(&hook)?),
        SudoMsg::RemoveSaleHook { hook } => sudo_remove_sale_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::RemoveAskHook { hook } => sudo_remove_ask_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::RemoveBidHook { hook } => sudo_remove_bid_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::UpdateAccountCollection { collection } => {
            sudo_update_account_collection(deps, api.addr_validate(&collection)?)
        }
        SudoMsg::UpdateAccountFactory { factory } => {
            sudo_update_account_minter(deps, api.addr_validate(&factory)?)
        } // SudoMsg::EndBlock {  } => todo!(),
    }
}
