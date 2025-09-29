use crate::TokenId;
use bs_controllers::HooksResponse;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp, Uint128};

pub type Collection = String;
pub type Bidder = String;
pub type Seller = String;
pub type AskKey = TokenId;
/// Primary key for bids: (token_id, bidder)
pub type BidKey = (TokenId, Addr);
pub type Id = u64;

/// Represents an ask on the marketplace
#[cosmwasm_schema::cw_serde]
pub struct Ask {
    pub token_id: TokenId,
    pub id: u64,
    pub seller: Addr,
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

#[cosmwasm_schema::cw_serde]
pub struct PendingBid {
    pub ask: Ask,
    pub new_owner: Addr,
    pub amount: Uint128,
    pub unlock_time: Timestamp,
}
impl PendingBid {
    pub fn new(ask: Ask, new_owner: Addr, amount: Uint128, unlock_time: Timestamp) -> Self {
        PendingBid {
            ask,
            new_owner,
            amount,
            unlock_time,
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct MarketplaceInstantiateMsg {
    /// Community pool fee for winning bids
    /// 0.25% = 25, 0.5% = 50, 1% = 100, 2.5% = 250
    pub trading_fee_bps: u64,
    /// Min value for bids and asks
    pub min_price: Uint128,
    /// Interval to rate limit setting asks (in seconds)
    pub ask_interval: u64,
    /// The number of bids to query to when searching for the highest bid
    pub valid_bid_query_limit: u32,
    /// Minimum time accepted bids are in escrow until they can be finalized.
    /// Improves security of account tokens.
    pub cooldown_timeframe: Timestamp,
    /// Fee required by token owner to cancel a bid they have accepted. Split betweeen Bitsong developers & biddee.
    pub cooldown_cancel_fee: Coin,
}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// List account NFT on the marketplace by creating a new ask.
    /// Only the account factory can call this.
    SetAsk {
        token_id: TokenId,
        seller: String,
    },
    /// Remove account on the marketplace.
    /// Only the account collection can call this (i.e: when burned).
    RemoveAsk {
        token_id: TokenId,
    },
    /// Update ask when an NFT is transferred
    /// Only the account collection can call this
    UpdateAsk {
        token_id: TokenId,
        seller: String,
    },
    /// Place a bid on an existing ask
    SetBid {
        token_id: TokenId,
    },
    /// Remove an existing bid from an ask.
    /// If bid is in cooldown period & current token_id owner is calling, this will revert
    RemoveBid {
        token_id: TokenId,
    },
    /// Accept a bid on an existing ask
    AcceptBid {
        token_id: TokenId,
        bidder: String,
    },
    /// Finalize a bid for an account once the delay period is complete.
    ///  Bidder or Bidee can call this function.
    FinalizeBid {
        token_id: TokenId,
    },
    CancelCooldown {
        token_id: TokenId,
    },
    Setup {
        minter: String,
        collection: String,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Get the current ask for specific name
    #[returns(Option<Ask>)]
    Ask { token_id: TokenId },
    /// Get all asks for a collection
    #[returns(Vec<Ask>)]
    Asks {
        start_after: Option<Id>,
        limit: Option<u32>,
    },
    /// Count of all asks
    #[returns(u64)]
    AskCount {},
    /// Get all asks by seller
    #[returns(Vec<Ask>)]
    AsksBySeller {
        seller: Seller,
        start_after: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get data for a specific bid
    #[returns(Option<Bid>)]
    Bid { token_id: TokenId, bidder: Bidder },
    /// Get all bids by a bidder
    #[returns(Vec<Bid>)]
    BidsByBidder {
        bidder: Bidder,
        start_after: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get all bids for a specific NFT
    #[returns(Vec<Bid>)]
    Bids {
        token_id: TokenId,
        start_after: Option<Bidder>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price
    #[returns(Vec<Bid>)]
    BidsSortedByPrice {
        start_after: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price in reverse
    #[returns(Vec<Bid>)]
    ReverseBidsSortedByPrice {
        start_before: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a specific account
    #[returns(Vec<Bid>)]
    BidsForSeller {
        seller: String,
        start_after: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get the highest bid for a name
    #[returns(Option<Bid>)]
    HighestBid { token_id: TokenId },
    /// Show all registered ask hooks
    #[returns(HooksResponse)]
    AskHooks {},
    /// Show all registered bid hooks
    #[returns(HooksResponse)]
    BidHooks {},
    /// Show all registered sale hooks
    #[returns(HooksResponse)]
    SaleHooks {},
    /// Get the config for the contract
    #[returns(SudoParams)]
    Params {},
    /// Get the minter and collection
    #[returns(ConfigResponse)]
    Config {},
    #[returns(Option<PendingBid>)]
    Cooldown { token_id: TokenId },
}

#[cosmwasm_schema::cw_serde]
pub enum SudoMsg {
    /// Update the contract parameters
    /// Can only be called by governance
    UpdateParams {
        trading_fee_bps: Option<u64>,
        min_price: Option<Uint128>,
        ask_interval: Option<u64>,
    },
    /// Update the contract address of the account factory
    UpdateAccountFactory { factory: String },
    /// Update the contract address of the name collection
    UpdateAccountCollection { collection: String },
    /// Add a new hook to be informed of all asks
    AddAskHook { hook: String },
    /// Remove a ask hook
    RemoveAskHook { hook: String },
    /// Add a new hook to be informed of all bids
    AddBidHook { hook: String },
    /// Remove a bid hook
    RemoveBidHook { hook: String },
    /// Add a new hook to be informed of all trades
    AddSaleHook { hook: String },
    /// Remove a trade hook
    RemoveSaleHook { hook: String },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub minter: Addr,
    pub collection: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct AskRenewPriceResponse {
    pub token_id: TokenId,
    pub price: Coin,
    pub bid: Option<Bid>,
}

/// Offset for bid pagination
#[cosmwasm_schema::cw_serde]
pub struct BidOffset {
    pub price: Uint128,
    pub token_id: TokenId,
    pub bidder: Addr,
}

impl BidOffset {
    pub fn new(price: Uint128, token_id: TokenId, bidder: Addr) -> Self {
        BidOffset {
            price,
            token_id,
            bidder,
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct SudoParams {
    /// Fair Burn + Community Pool fee for winning bids
    pub trading_fee_percent: Decimal,
    /// Min value for a bid
    pub min_price: Uint128,
    /// Interval to rate limit setting asks (in seconds)
    pub ask_interval: u64,
    /// The number of bids to query to when searching for the highest bid
    pub valid_bid_query_limit: u32,
    pub cooldown_duration: Timestamp,
    pub cooldown_fee: Coin,
}

pub struct ParamInfo {
    pub trading_fee_bps: Option<u64>,
    pub min_price: Option<Uint128>,
    pub ask_interval: Option<u64>,
}
