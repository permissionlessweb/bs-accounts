use cosmwasm_std::Coin;

pub mod minter;
pub mod traits;
pub mod verify_generic;

pub const MAX_TEXT_LENGTH: u32 = 512;
pub const NATIVE_DENOM: &str = "ubtsg";
pub const SECONDS_PER_YEAR: u64 = 31536000;

/// Custom bitsong name metadata.
#[cosmwasm_schema::cw_serde]
#[derive(Default)]
pub struct Metadata {
    /// signals if this account token is making use of an tokenized-account authentication system.
    pub account_ownership: bool,
    pub image_nft: Option<NFT>,
    pub records: Vec<TextRecord>,
}

impl Metadata {
    pub fn into_json_string(self: &Metadata) -> Result<String, cosmwasm_std::StdError> {
        let json_vec = cosmwasm_std::to_json_vec(&self)?;
        String::from_utf8(json_vec).map_err(cosmwasm_std::StdError::from)
    }
    pub fn default_with_account() -> Self {
        Self {
            account_ownership: true,
            image_nft: None,
            records: vec![],
        }
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
    pub fn new(
        deps: cosmwasm_std::Deps,
        collection: String,
        token_id: String,
    ) -> cosmwasm_std::StdResult<Self> {
        Ok(Self {
            collection: deps.api.addr_validate(&collection)?,
            token_id,
        })
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

pub fn charge_fees(res: &mut cosmwasm_std::Response, fee: cosmwasm_std::Uint256) {
    if fee > cosmwasm_std::Uint256::zero() {
        res.messages
            .push(cosmwasm_std::SubMsg::new(cosmwasm_std::BankMsg::Burn {
                amount: vec![Coin::new(fee, NATIVE_DENOM)],
            }));
    }
}
