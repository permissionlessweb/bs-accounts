use btsg_account::{market::*, TokenId};
use cosmwasm_std::{to_json_binary, Binary, StdResult, Uint128};

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
