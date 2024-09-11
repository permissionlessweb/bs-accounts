use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Timestamp, Uint128};
use cw_controllers::Hooks;
use cw_storage_macro::index_list;
use cw_storage_plus::{IndexedMap, Item, Map, MultiIndex, UniqueIndex};

// bps fee can not exceed 100%
pub const MAX_FEE_BPS: u64 = 10000;
/// Type for storing the `ask`
pub type TokenId = String;
/// Type for `ask` unique secondary index
pub type Id = u64;

#[cosmwasm_schema::cw_serde]
pub struct SudoParams {
    /// Fair Burn + Community Pool fee for winning bids
    pub trading_fee_percent: Decimal,
    /// Min value for a bid
    pub min_price: Uint128,
    /// Interval to rate limit setting asks (in seconds)
    pub ask_interval: u64,
    /// The maximum number of renewals that can be processed in each block
    pub max_renewals_per_block: u32,
    /// The number of bids to query to when searching for the highest bid
    pub valid_bid_query_limit: u32,
    /// The number of seconds before the current block time that a
    /// bid must have been created in order to be considered valid
    pub renew_window: u64,
    /// The percentage of the winning bid that must be paid to renew a name
    pub renewal_bid_percentage: Decimal,
    /// The address with permission to invoke process_renewals
    pub operator: Addr,
}

pub struct ParamInfo {
    pub trading_fee_bps: Option<u64>,
    pub min_price: Option<Uint128>,
    pub ask_interval: Option<u64>,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sp");

pub const ASK_HOOKS: Hooks = Hooks::new("ah");
pub const BID_HOOKS: Hooks = Hooks::new("bh");
pub const SALE_HOOKS: Hooks = Hooks::new("sh");

pub const ACCOUNT_MINTER: Item<Addr> = Item::new("am");
pub const ACCOUNT_COLLECTION: Item<Addr> = Item::new("ac");
pub const VERSION_CONTROL: Item<Addr> = Item::new("vc");

/// (renewal_time (in seconds), id) -> [token_id]
pub const RENEWAL_QUEUE: Map<(u64, u64), TokenId> = Map::new("rq");

pub const ASK_COUNT: Item<u64> = Item::new("ask-count");
pub const IS_SETUP: Item<bool> = Item::new("is");

pub fn ask_count(storage: &dyn Storage) -> StdResult<u64> {
    Ok(ASK_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_asks(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = ask_count(storage)? + 1;
    ASK_COUNT.save(storage, &val)?;
    Ok(val)
}

pub fn decrement_asks(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = ask_count(storage)? - 1;
    ASK_COUNT.save(storage, &val)?;
    Ok(val)
}

/// Represents an ask on the marketplace
#[cosmwasm_schema::cw_serde]
pub struct Ask {
    pub token_id: TokenId,
    pub id: u64,
    pub seller: Addr,
    pub renewal_time: Timestamp,
    pub renewal_fund: Uint128,
}

/// Primary key for asks: token_id
/// Name reverse lookup can happen in O(1) time
pub type AskKey = TokenId;
/// Convenience ask key constructor
pub fn ask_key(token_id: &str) -> AskKey {
    token_id.to_string()
}

/// Defines indices for accessing Asks
#[index_list(Ask)]
pub struct AskIndicies<'a> {
    /// Unique incrementing id for each ask
    /// This allows pagination when `token_id`s are strings
    pub id: UniqueIndex<'a, u64, Ask, AskKey>,
    /// Index by seller
    pub seller: MultiIndex<'a, Addr, Ask, AskKey>,
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndicies<'a>> {
    let indexes = AskIndicies {
        id: UniqueIndex::new(|d| d.id, "ask__id"),
        seller: MultiIndex::new(
            |_pk: &[u8], d: &Ask| d.seller.clone(),
            "asks",
            "asks__seller",
        ),
    };
    IndexedMap::new("asks", indexes)
}

/// Represents a bid (offer) on the marketplace
#[cosmwasm_schema::cw_serde]
pub struct Bid {
    pub token_id: TokenId,
    pub bidder: Addr,
    pub amount: Uint128,
    pub created_time: Timestamp,
}

impl Bid {
    pub fn new(token_id: &str, bidder: Addr, amount: Uint128, created_time: Timestamp) -> Self {
        Bid {
            token_id: token_id.to_string(),
            bidder,
            amount,
            created_time,
        }
    }
}

/// Primary key for bids: (token_id, bidder)
pub type BidKey = (TokenId, Addr);
/// Convenience bid key constructor
pub fn bid_key(token_id: &str, bidder: &Addr) -> BidKey {
    (token_id.to_string(), bidder.clone())
}

/// Defines indices for accessing bids
#[index_list(Bid)]
pub struct BidIndicies<'a> {
    pub price: MultiIndex<'a, u128, Bid, BidKey>,
    pub bidder: MultiIndex<'a, Addr, Bid, BidKey>,
    pub created_time: MultiIndex<'a, u64, Bid, BidKey>,
}

pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndicies<'a>> {
    let indexes = BidIndicies {
        price: MultiIndex::new(
            |_pk: &[u8], d: &Bid| d.amount.u128(),
            "bids",
            "bids__collection_price",
        ),
        bidder: MultiIndex::new(
            |_pk: &[u8], d: &Bid| d.bidder.clone(),
            "bids",
            "bids__bidder",
        ),
        created_time: MultiIndex::new(
            |_pk: &[u8], d: &Bid| d.created_time.seconds(),
            "bids",
            "bids__time",
        ),
    };
    IndexedMap::new("bids", indexes)
}
