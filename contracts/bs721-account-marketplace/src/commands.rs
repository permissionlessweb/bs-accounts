use crate::hooks::{prepare_ask_hook, prepare_bid_hook, prepare_sale_hook};
use crate::msgs::HookAction;
use crate::{
    error::ContractError,
    // hooks::{prepare_ask_hook, prepare_bid_hook, prepare_sale_hook},
    state::*,
};
use btsg_account::{
    charge_fees, validate_aa_ownership, Metadata, DEFAULT_QUERY_LIMIT, DEPLOYMENT_DAO,
    MAX_QUERY_LIMIT, NATIVE_DENOM,
};
use btsg_account::{market::*, TokenId};

use cosmwasm_std::{
    coin, to_json_binary, Addr, BankMsg, Decimal, Deps, DepsMut, Empty, Env, Event, Fraction,
    MessageInfo, Order, Response, StdError, StdResult, Storage, SubMsg, SubMsgResult, Uint128,
    WasmMsg,
};
use std::marker::PhantomData;

use bs721::{Bs721ExecuteMsg, Bs721QueryMsg, NftInfoResponse, OwnerOfResponse};
use bs721_base::helpers::Bs721Contract;
use cw_storage_plus::Bound;
use cw_utils::{must_pay, nonpayable};

/// Setup this contract (can be run once only)
pub fn execute_setup(
    deps: DepsMut,
    minter: Addr,
    collection: Addr,
) -> Result<Response, ContractError> {
    if IS_SETUP.load(deps.storage)? {
        return Err(ContractError::AlreadySetup {});
    }
    ACCOUNT_MINTER.save(deps.storage, &minter)?;
    ACCOUNT_COLLECTION.save(deps.storage, &collection)?;
    IS_SETUP.save(deps.storage, &true)?;

    Ok(Response::new().add_event(
        Event::new("setup")
            .add_attribute("minter", minter)
            .add_attribute("collection", collection),
    ))
}

/// A seller may set an Ask on their NFT to list it on Marketplace
pub fn execute_set_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
    seller: Addr,
) -> Result<Response, ContractError> {
    let minter = ACCOUNT_MINTER.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::UnauthorizedMinter {});
    }

    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;

    // check if collection is approved to transfer on behalf of the seller
    let ops = Bs721Contract::<Empty, Empty>(collection, PhantomData, PhantomData).all_operators(
        &deps.querier,
        seller.to_string(),
        false,
        None,
        None,
    )?;

    // println!("{:#?}", ops);
    if ops.is_empty() {
        return Err(ContractError::NotApproved {});
    }

    let renewal_time = env.block.time.plus_seconds(31536000u64);

    let ask = Ask {
        token_id: token_id.to_string(),
        id: increment_asks(deps.storage)?,
        seller: seller.clone(),
    };
    store_ask(deps.storage, &ask)?;

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Create)?;

    let event = Event::new("set-ask")
        .add_attribute("token_id", token_id)
        .add_attribute("ask_id", ask.id.to_string())
        .add_attribute("renewal_time", renewal_time.to_string())
        .add_attribute("seller", seller);

    Ok(Response::new().add_event(event).add_submessages(hook))
}

/// Removes the ask on a particular NFT
pub fn execute_remove_ask(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // `ask` can only be removed by burning from the collection
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;
    if info.sender != collection {
        return Err(ContractError::Unauthorized {});
    }

    // don't allow burning if ask has bids on it
    let bid_count = bids()
        .prefix(token_id.to_string())
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    if bid_count > 0 {
        return Err(ContractError::ExistingBids {});
    }

    let key = ask_key(token_id);
    let ask = asks().load(deps.storage, key.clone())?;
    asks().remove(deps.storage, key)?;

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Delete)?;

    let event = Event::new("remove-ask").add_attribute("token_id", token_id);

    Ok(Response::new().add_event(event).add_submessages(hook))
}

/// When an NFT is transferred, the `ask` has to be updated with the new
/// seller. Also any renewal funds should be refunded to the previous owner.
pub fn execute_update_ask(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
    seller: Addr,
) -> Result<Response, ContractError> {
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;
    if info.sender != collection {
        return Err(ContractError::Unauthorized {});
    }

    // refund any renewal funds and update the seller
    let mut ask = asks().load(deps.storage, ask_key(token_id))?;
    ask.seller = seller.clone();
    asks().save(deps.storage, ask_key(token_id), &ask)?;

    let event = Event::new("update-ask")
        .add_attribute("token_id", token_id)
        .add_attribute("seller", seller);

    Ok(Response::new().add_event(event))
}

/// Places a bid on a account. The bid is escrowed in the contract.
pub fn execute_set_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    let params = SUDO_PARAMS.load(deps.storage)?;

    let ask_key = ask_key(token_id);
    asks().load(deps.storage, ask_key)?;

    let bid_price = must_pay(&info, NATIVE_DENOM)?;
    if bid_price < params.min_price {
        return Err(ContractError::PriceTooSmall(bid_price));
    }

    let bidder = info.sender;
    let mut res = Response::new();
    let bid_key = bid_key(token_id, &bidder);

    if let Some(existing_bid) = bids().may_load(deps.storage, bid_key.clone())? {
        bids().remove(deps.storage, bid_key)?;
        let refund_bidder = BankMsg::Send {
            to_address: bidder.to_string(),
            amount: vec![coin(existing_bid.amount.u128(), NATIVE_DENOM)],
        };
        res = res.add_message(refund_bidder)
    }

    let bid = Bid::new(token_id, bidder.clone(), bid_price, env.block.time);
    store_bid(deps.storage, &bid)?;

    let hook = prepare_bid_hook(deps.as_ref(), &bid.clone(), HookAction::Create)?;

    let event = Event::new("set-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder)
        .add_attribute("bid_price", bid_price.to_string());

    Ok(res.add_event(event).add_submessages(hook))
}

/// Removes a bid made by the bidder. Bidders can only remove their own bids
pub fn execute_cancel_cooldown(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    let cd_key = &ask_key(token_id);
    match cooldown_bids().may_load(deps.storage, cd_key)? {
        Some(p) => {
            // sender must be current token owner
            if info.sender != p.ask.seller {
                return Err(ContractError::Unauthorized {});
            };
            let mut res = Response::default();
            let params = SUDO_PARAMS.load(deps.storage)?;
            if params.cooldown_duration != 0 {
                // must have sent cancel cooldown fee
                let payment = must_pay(&info, NATIVE_DENOM)?;

                if payment != params.cooldown_fee.amount {
                    return Err(ContractError::IncorrectPayment {
                        got: payment.u128(),
                        expected: params.cooldown_fee.amount.u128(),
                    });
                }

                // cannot cancel if cooldown period is over
                if env.block.time >= p.unlock_time {
                    return Err(ContractError::InvalidDuration {});
                }

                // refund bidder
                let dev_cut = params
                    .cooldown_fee
                    .amount
                    .u128()
                    .checked_div(2)
                    .expect("fatal division error");

                let dev_cut_msg = BankMsg::Send {
                    to_address: DEPLOYMENT_DAO.to_string(),
                    amount: vec![coin(dev_cut, NATIVE_DENOM.to_string())],
                };
                // refund bidder
                let seller_share_msg = BankMsg::Send {
                    to_address: p.new_owner.to_string(),
                    amount: vec![coin(
                        p.amount.u128() + (params.cooldown_fee.amount.u128() - dev_cut),
                        NATIVE_DENOM.to_string(),
                    )],
                };
                res.messages.extend(vec![
                    SubMsg::new(seller_share_msg),
                    SubMsg::new(dev_cut_msg),
                ]);
            }
            cooldown_bids().remove(deps.storage, cd_key)?;
            Ok(res)
        }
        None => return Err(ContractError::AskNotFound {}),
    }
}
/// Removes a bid made by the bidder. Bidders can only remove their own bids
pub fn execute_remove_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let bidder = info.sender;

    let key = bid_key(token_id, &bidder);
    let bid = bids().load(deps.storage, key.clone())?;
    bids().remove(deps.storage, key)?;

    let refund_bidder_msg = BankMsg::Send {
        to_address: bid.bidder.to_string(),
        amount: vec![coin(bid.amount.u128(), NATIVE_DENOM)],
    };

    let hook = prepare_bid_hook(deps.as_ref(), &bid, HookAction::Delete)?;

    let event = Event::new("remove-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder);

    let res = Response::new()
        .add_message(refund_bidder_msg)
        .add_submessages(hook)
        .add_event(event);

    Ok(res)
}

pub fn execute_finalize_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;
    let cd_key = &ask_key(token_id);
    let pending = cooldown_bids().may_load(deps.storage, cd_key)?;
    let mut res = Response::default();
    match pending {
        Some(mut p) => {
            // check sender is either current or new owner
            if info.sender != p.ask.seller && info.sender != p.new_owner {
                return Err(ContractError::CannotFinalizeBid {});
            }
            // check if pending bid is ready to be finalized
            if env.block.time > p.unlock_time {
                return Err(ContractError::InvalidDuration {});
            }
            // Check if token is approved for transfer
            Bs721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData).approval(
                &deps.querier,
                token_id,
                &p.ask.seller.to_string(),
                None,
            )?;

            let token: NftInfoResponse<Metadata> =
                Bs721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
                    .nft_info(&deps.querier, token_id)?;

            // get the associated abstarct account we expect to exist
            if token.extension.account_ownership
                && validate_aa_ownership(
                    deps.as_ref(),
                    &token.token_uri.expect(
                        "should never have aa support enabled and not have account associated",
                    ),
                    token_id,
                    &collection.clone(),
                    true,
                )
                .is_err()
            {
                // abstract account does not use this token for its ownership method.
                // we refund the bidder by setting their address in the current ask, and still transfer the token to the bidder.
                // this penalizes the original owner for changing ownership of their account during the cooldown phase.
                p.ask.seller = p.new_owner.clone();
            }

            // Transfer funds and NFT
            finalize_sale(
                deps.as_ref(),
                p.ask.clone(),
                p.amount,
                p.new_owner.clone(),
                &mut res,
            )?;
            cooldown_bids().remove(deps.storage, cd_key)?;
            store_ask(
                deps.storage,
                &Ask {
                    token_id: token_id.to_string(),
                    id: p.ask.id,
                    seller: p.new_owner.clone(),
                },
            )?;
        }
        None => {
            return Err(ContractError::AskNotFound {});
        }
    }

    Ok(res)
}
/// Seller can accept a bid which transfers funds as well as the token.
/// The bid is removed, then a new ask is created for the same token.
pub fn execute_accept_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
    bidder: Addr,
) -> Result<Response, ContractError> {
    // println!("1.0 execute_accept bid ----------------------------");
    nonpayable(&info)?;
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;
    let cooldown = SUDO_PARAMS.load(deps.storage)?.cooldown_duration;
    only_owner(deps.as_ref(), &info, &collection, token_id)?;

    let ask_key = ask_key(token_id);
    let bid_key = bid_key(token_id, &bidder);

    let ask = asks().load(deps.storage, ask_key.clone())?;
    let bid = bids().load(deps.storage, bid_key.clone())?;

    // Check if token is approved for transfer
    Bs721Contract::<Empty, Empty>(collection, PhantomData, PhantomData).approval(
        &deps.querier,
        token_id,
        info.sender.as_ref(),
        None,
    )?;

    // Remove accepted bid
    bids().remove(deps.storage, bid_key)?;

    // begin cooldown period
    let unlock_time = env.block.time.plus_seconds(cooldown);
    let pending = PendingBid::new(ask.clone(), bidder.clone(), bid.amount, unlock_time);
    cooldown_bids().save(deps.storage, &ask_key, &pending)?;

    Ok(Response::default().add_event(
        Event::new("accept-bid")
            .add_attribute("token_id", token_id)
            .add_attribute("bidder", bidder)
            .add_attribute("price", bid.amount.to_string()),
    ))
}

/// Transfers funds and NFT, updates bid
fn finalize_sale(
    deps: Deps,
    ask: Ask,
    price: Uint128,
    buyer: Addr,
    res: &mut Response,
) -> StdResult<()> {
    // println!("1.1 finalize sale ----------------------------");
    payout(deps, price, ask.seller.clone(), res)?;

    let cw721_transfer_msg: Bs721ExecuteMsg<Metadata, Empty> = Bs721ExecuteMsg::TransferNft {
        token_id: ask.token_id.to_string(),
        recipient: buyer.to_string(),
    };

    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;

    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_json_binary(&cw721_transfer_msg)?,
        funds: vec![],
    };
    res.messages.push(SubMsg::new(exec_cw721_transfer));

    res.messages
        .append(&mut prepare_sale_hook(deps, &ask, buyer.clone())?);

    let event = Event::new("finalize-sale")
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("seller", ask.seller.to_string())
        .add_attribute("buyer", buyer.to_string())
        .add_attribute("price", price.to_string());
    res.events.push(event);

    Ok(())
}

/// Payout a bid
fn payout(
    deps: Deps,
    payment: Uint128,
    payment_recipient: Addr,
    res: &mut Response,
) -> StdResult<()> {
    let params = SUDO_PARAMS.load(deps.storage)?;

    let fee = payment.multiply_ratio(
        params.trading_fee_percent.numerator(),
        params.trading_fee_percent.denominator(),
    );
    if fee > payment {
        return Err(StdError::generic_err("Fees exceed payment"));
    }
    charge_fees(res, fee);

    // pay seller
    let seller_share_msg = BankMsg::Send {
        to_address: payment_recipient.to_string(),
        amount: vec![coin((payment - fee).u128(), NATIVE_DENOM.to_string())],
    };
    res.messages.push(SubMsg::new(seller_share_msg));

    Ok(())
}

fn store_bid(store: &mut dyn Storage, bid: &Bid) -> StdResult<()> {
    bids().save(store, bid_key(&bid.token_id, &bid.bidder), bid)
}

pub fn store_ask(store: &mut dyn Storage, ask: &Ask) -> StdResult<()> {
    asks().save(store, ask_key(&ask.token_id), ask)
}

/// Checks to enfore only NFT owner can call
fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<OwnerOfResponse, ContractError> {
    let res = Bs721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
        .owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender.to_string() {
        return Err(ContractError::UnauthorizedOwner {});
    }

    Ok(res)
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let minter = ACCOUNT_MINTER.load(deps.storage)?;
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;

    Ok(ConfigResponse { minter, collection })
}

pub fn query_asks(deps: Deps, start_after: Option<Id>, limit: Option<u32>) -> StdResult<Vec<Ask>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    asks()
        .idx
        .id
        .range(
            deps.storage,
            Some(Bound::exclusive(start_after.unwrap_or_default())),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_ask_count(deps: Deps) -> StdResult<u64> {
    ASK_COUNT.load(deps.storage)
}

// TODO: figure out how to paginate by `Id` instead of `TokenId`
pub fn query_asks_by_seller(
    deps: Deps,
    seller: Addr,
    start_after: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<Vec<Ask>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|start| Bound::exclusive(ask_key(&start)));

    asks()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_ask(deps: Deps, token_id: TokenId) -> StdResult<Option<Ask>> {
    asks().may_load(deps.storage, ask_key(&token_id))
}

pub fn query_bid(deps: Deps, token_id: TokenId, bidder: Addr) -> StdResult<Option<Bid>> {
    bids().may_load(deps.storage, (token_id, bidder))
}

pub fn query_bids_by_bidder(
    deps: Deps,
    bidder: Addr,
    start_after: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|start| Bound::exclusive((start, bidder.clone())));

    bids()
        .idx
        .bidder
        .prefix(bidder)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_bids_for_seller(
    deps: Deps,
    seller: Addr,
    start_after: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    // Query seller asks, then collect bids starting after token_id
    // Limitation: Can not collect bids in the middle using `start_after: token_id` pattern
    // This leads to imprecise pagination based on token id and not bid count
    let start_token_id =
        start_after.map(|start| Bound::<AskKey>::exclusive(ask_key(&start.token_id)));

    let bids = asks()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, start_token_id, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.0).unwrap())
        .flat_map(|token_id| {
            bids()
                .prefix(token_id)
                .range(deps.storage, None, None, Order::Ascending)
                .flat_map(|item| item.map(|(_, b)| b))
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(bids)
}

pub fn query_bids(
    deps: Deps,
    token_id: TokenId,
    start_after: Option<Bidder>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    bids()
        .prefix(token_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_highest_bid(deps: Deps, token_id: TokenId) -> StdResult<Option<Bid>> {
    let bid = bids()
        .idx
        .price
        .range(deps.storage, None, None, Order::Descending)
        .filter_map(|item| {
            let (key, bid) = item.unwrap();
            if key.0 == token_id {
                Some(bid)
            } else {
                None
            }
        })
        .take(1)
        .collect::<Vec<_>>()
        .first()
        .cloned();

    Ok(bid)
}

pub fn query_bids_sorted_by_price(
    deps: Deps,
    start_after: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|offset| {
        Bound::exclusive((
            (offset.token_id.clone(), offset.price.u128()),
            bid_key(&offset.token_id, &offset.bidder),
        ))
    });

    bids()
        .idx
        .price
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn reverse_query_bids_sorted_by_price(
    deps: Deps,
    start_before: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let end = start_before.map(|offset| {
        Bound::exclusive((
            (offset.token_id.clone(), offset.price.u128()),
            bid_key(&offset.token_id, &offset.bidder),
        ))
    });

    bids()
        .idx
        .price
        .range(deps.storage, None, end, Order::Descending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_params(deps: Deps) -> StdResult<SudoParams> {
    SUDO_PARAMS.load(deps.storage)
}

/// Only governance can update contract params
pub fn sudo_update_params(
    deps: DepsMut,
    _env: Env,
    param_info: ParamInfo,
) -> Result<Response, ContractError> {
    let ParamInfo {
        trading_fee_bps,
        min_price,
        ask_interval,
        cooldown_duration,
        cooldown_cancel_fee,
    } = param_info;
    if let Some(trading_fee_bps) = trading_fee_bps {
        if trading_fee_bps > MAX_FEE_BPS {
            return Err(ContractError::InvalidTradingFeeBps(trading_fee_bps));
        }
    }

    let mut params = SUDO_PARAMS.load(deps.storage)?;

    params.trading_fee_percent = trading_fee_bps
        .map(|bps| Decimal::percent(bps) / Uint128::from(100u128))
        .unwrap_or(params.trading_fee_percent);

    params.min_price = min_price.unwrap_or(params.min_price);
    params.ask_interval = ask_interval.unwrap_or(params.ask_interval);
    params.cooldown_duration = cooldown_duration.unwrap_or(params.cooldown_duration);
    params.cooldown_fee = cooldown_cancel_fee.unwrap_or(params.cooldown_fee);

    SUDO_PARAMS.save(deps.storage, &params)?;

    let event = Event::new("update-params")
        .add_attribute(
            "trading_fee_percent",
            params.trading_fee_percent.to_string(),
        )
        .add_attribute("min_price", params.min_price);
    Ok(Response::new().add_event(event))
}

pub fn sudo_update_account_minter(
    deps: DepsMut,
    collection: Addr,
) -> Result<Response, ContractError> {
    ACCOUNT_MINTER.save(deps.storage, &collection)?;

    let event = Event::new("update-account-minter").add_attribute("minter", collection);
    Ok(Response::new().add_event(event))
}

pub fn sudo_update_account_collection(
    deps: DepsMut,
    collection: Addr,
) -> Result<Response, ContractError> {
    ACCOUNT_COLLECTION.save(deps.storage, &collection)?;

    let event = Event::new("update-account-collection").add_attribute("collection", collection);
    Ok(Response::new().add_event(event))
}

pub fn sudo_add_sale_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    SALE_HOOKS.add_hook(deps.storage, hook.clone())?;

    let event = Event::new("add-sale-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_add_ask_hook(deps: DepsMut, _env: Env, hook: Addr) -> Result<Response, ContractError> {
    ASK_HOOKS.add_hook(deps.storage, hook.clone())?;

    let event = Event::new("add-ask-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_add_bid_hook(deps: DepsMut, _env: Env, hook: Addr) -> Result<Response, ContractError> {
    BID_HOOKS.add_hook(deps.storage, hook.clone())?;

    let event = Event::new("add-bid-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_remove_sale_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    SALE_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let event = Event::new("remove-sale-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_remove_ask_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    ASK_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let event = Event::new("remove-ask-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_remove_bid_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    BID_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let event = Event::new("remove-bid-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

/// Propose the marketplace as owner for escrow of account
fn _propose_accepted_bidder_a(_deps: Deps, _env: Env, _res: &mut Response) -> StdResult<()> {
    // propose owner as marketplace for escrow purposes
    Ok(())
}

pub(crate) fn _propose_accepted_bidder_a_response(
    _env: Env,
    _deps: DepsMut,
    _result: SubMsgResult,
) -> Result<Response, ContractError> {
    let res = Response::new();
    Ok(res)
}
