use std::cmp::max;

use btsg_account::common::{charge_fees, SECONDS_PER_YEAR};
use cosmwasm_std::{Addr, Deps, DepsMut, Env, Event, Order, Response, Timestamp, Uint128};
use cw_storage_plus::Bound;

use crate::{commands::store_ask, state::*, ContractError};

// Renewal price is the max of the char based price and a percentage of highest valid bid
pub fn get_renewal_price_and_bid(
    deps: Deps,
    block_time: &Timestamp,
    sudo_params: &SudoParams,
    token_id: &str,
    base_price: u128,
) -> Result<(Uint128, Option<Bid>), ContractError> {
    let renewal_char_price = get_char_price(base_price, token_id.len());
    let valid_bid = find_valid_bid(deps, block_time, sudo_params, token_id, renewal_char_price)?;

    let renewal_bid_price = valid_bid.as_ref().map_or(Uint128::zero(), |bid| {
        bid.amount * sudo_params.renewal_bid_percentage
    });

    let renewal_price = max(renewal_char_price, renewal_bid_price);

    Ok((renewal_price, valid_bid))
}

/// Iterate over top n priced bids, if one is within the time window then it is valid
pub fn find_valid_bid(
    deps: Deps,
    block_time: &Timestamp,
    sudo_params: &SudoParams,
    token_id: &str,
    min_price: Uint128,
) -> Result<Option<Bid>, ContractError> {
    let max_time = block_time.seconds() - sudo_params.renew_window;

    let bid = bids()
        .idx
        .price
        .sub_prefix(())
        .range(
            deps.storage,
            Some(Bound::inclusive((
                min_price.u128(),
                (token_id.to_string(), Addr::unchecked("")),
            ))),
            None,
            Order::Descending,
        )
        .take(sudo_params.valid_bid_query_limit as usize)
        .find_map(|item| {
            item.map_or(None, |(_, bid)| {
                if bid.created_time.seconds() <= max_time {
                    Some(bid)
                } else {
                    None
                }
            })
        });

    Ok(bid)
}

// Calculate the renewal price based on the account length
pub fn get_char_price(base_price: u128, account_len: usize) -> Uint128 {
    match account_len {
        0..=2 => unreachable!("account_len should be at least 3"),
        3 => base_price * 100,
        4 => base_price * 10,
        _ => base_price,
    }
    .into()
}

pub fn renew_account(
    deps: DepsMut,
    _env: &Env,
    _sudo_params: &SudoParams,
    mut ask: Ask,
    renewal_price: Uint128,
    mut response: Response,
) -> Result<Response, ContractError> {
    if !renewal_price.is_zero() {
        // Take renewal payment
        ask.renewal_fund -= renewal_price;
        charge_fees(
            &mut response,
            renewal_price,
            // sudo_params.trading_fee_percent,
        );
    }

    // Update renewal time
    RENEWAL_QUEUE.remove(deps.storage, (ask.renewal_time.seconds(), ask.id));
    ask.renewal_time = ask.renewal_time.plus_seconds(SECONDS_PER_YEAR);
    RENEWAL_QUEUE.save(
        deps.storage,
        (ask.renewal_time.seconds(), ask.id),
        &ask.token_id,
    )?;

    store_ask(deps.storage, &ask)?;

    response = response.add_event(
        Event::new("renew-account")
            .add_attribute("token_id", ask.token_id.to_string())
            .add_attribute("renewal_price", renewal_price)
            .add_attribute("next_renewal_time", ask.renewal_time.to_string()),
    );

    Ok(response)
}
