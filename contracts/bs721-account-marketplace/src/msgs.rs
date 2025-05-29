use crate::state::*;
use bs_controllers::HooksResponse;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{to_json_binary, Addr, Binary, Coin, StdResult, Uint128};

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Community pool fee for winning bids
    /// 0.25% = 25, 0.5% = 50, 1% = 100, 2.5% = 250
    pub trading_fee_bps: u64,
    /// Min value for bids and asks
    pub min_price: Uint128,
    /// Interval to rate limit setting asks (in seconds)
    pub ask_interval: u64,
    /// The number of bids to query to when searching for the highest bid
    pub valid_bid_query_limit: u32,
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
    /// Remove an existing bid from an ask
    RemoveBid {
        token_id: TokenId,
    },
    /// Accept a bid on an existing ask
    AcceptBid {
        token_id: TokenId,
        bidder: String,
    },
    Setup {
        minter: String,
        collection: String,
    },
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

pub type Collection = String;
pub type Bidder = String;
pub type Seller = String;

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

/// Offset for ask pagination
#[cosmwasm_schema::cw_serde]
pub struct AskOffset {
    pub price: Uint128,
    pub token_id: TokenId,
}

impl AskOffset {
    pub fn new(price: Uint128, token_id: TokenId) -> Self {
        AskOffset { price, token_id }
    }
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
pub struct SaleHookMsg {
    pub token_id: String,
    pub seller: String,
    pub buyer: String,
}

impl SaleHookMsg {
    pub fn new(token_id: &str, seller: String, buyer: String) -> Self {
        SaleHookMsg {
            token_id: token_id.to_string(),
            seller,
            buyer,
        }
    }

    /// serializes the message
    pub fn into_json_binary(self) -> StdResult<Binary> {
        let msg = SaleExecuteMsg::SaleHook(self);
        to_json_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cosmwasm_schema::cw_serde]
pub enum SaleExecuteMsg {
    SaleHook(SaleHookMsg),
}

#[cosmwasm_schema::cw_serde]
pub enum HookAction {
    Create,
    Update,
    Delete,
}

#[cosmwasm_schema::cw_serde]
pub struct AskHookMsg {
    pub ask: Ask,
}

impl AskHookMsg {
    pub fn new(ask: Ask) -> Self {
        AskHookMsg { ask }
    }

    /// serializes the message
    pub fn into_json_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Create => AskHookExecuteMsg::AskCreatedHook(self),
            HookAction::Update => AskHookExecuteMsg::AskUpdatedHook(self),
            HookAction::Delete => AskHookExecuteMsg::AskDeletedHook(self),
        };
        to_json_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cosmwasm_schema::cw_serde]
pub enum AskHookExecuteMsg {
    AskCreatedHook(AskHookMsg),
    AskUpdatedHook(AskHookMsg),
    AskDeletedHook(AskHookMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct BidHookMsg {
    pub bid: Bid,
}

impl BidHookMsg {
    pub fn new(bid: Bid) -> Self {
        BidHookMsg { bid }
    }

    /// serializes the message
    pub fn into_json_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Create => BidExecuteMsg::BidCreatedHook(self),
            HookAction::Update => BidExecuteMsg::BidUpdatedHook(self),
            HookAction::Delete => BidExecuteMsg::BidDeletedHook(self),
        };
        to_json_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cosmwasm_schema::cw_serde]
pub enum BidExecuteMsg {
    BidCreatedHook(BidHookMsg),
    BidUpdatedHook(BidHookMsg),
    BidDeletedHook(BidHookMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
