pub const MAX_TEXT_LENGTH: u32 = 512;
pub const NATIVE_DENOM: &str = "ubtsg";
pub const SECONDS_PER_YEAR: u64 = 31536000;

/// Custom bitsong name metadata.
#[cosmwasm_schema::cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image_nft: Option<NFT>,
    pub records: Vec<TextRecord>,
}

impl Metadata {
    pub fn into_json_string(self: &Metadata) -> Result<String, cosmwasm_std::StdError> {
        let json_vec = cosmwasm_std::to_json_vec(&self)?;
        String::from_utf8(json_vec).map_err(cosmwasm_std::StdError::from)
    }
}

pub type TokenId = String;

#[cosmwasm_schema::cw_serde]
pub struct NFT {
    pub collection: cosmwasm_std::Addr,
    pub token_id: TokenId,
}

impl NFT {
    pub fn into_json_string(self: &NFT) -> String {
        String::from_utf8(cosmwasm_std::to_json_vec(&self).unwrap_or_default()).unwrap_or_default()
    }
}

#[cosmwasm_schema::cw_serde]
pub struct TextRecord {
    pub account: String,        // "twitter"
    pub value: String,          // "shan3v"
    pub verified: Option<bool>, // can only be set by oracle
}

impl TextRecord {
    pub fn new(account: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            value: value.into(),
            verified: None,
        }
    }

    pub fn into_json_string(self: &TextRecord) -> String {
        String::from_utf8(cosmwasm_std::to_json_vec(&self).unwrap_or_default()).unwrap_or_default()
    }
}

pub fn charge_fees(res: &mut cosmwasm_std::Response, fee: cosmwasm_std::Uint128) {
    if fee > cosmwasm_std::Uint128::zero() {
        res.messages
            .push(cosmwasm_std::SubMsg::new(cosmwasm_std::BankMsg::Burn {
                amount: cosmwasm_std::coins(fee.u128(), NATIVE_DENOM),
            }));
    }
}

pub mod marketplace {
    use super::*;

    pub mod msgs {
        use super::*;
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
    }

    pub mod state {

        use super::*;
    }
}
