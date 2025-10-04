use btsg_account::minter::{Config, SudoParams};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use cw_ownable::cw_ownable_execute;

#[cw_serde]
pub struct InstantiateMsg {
    /// Temporary admin for managing whitelists
    pub admin: Option<String>,
    /// Oracle for verifying text records
    pub verifier: Option<String>,
    /// Code-id for BS721-Account. On Instantiate, minter will instantiate a new account collection.
    pub collection_code_id: u64,
    /// bs721-account marketplace address
    pub marketplace_addr: String,
    /// Minimum length an account id can be
    pub min_account_length: u32,
    /// Maximum length an account id can be
    pub max_account_length: u32,
    /// Base price for a account. Used to calculate premium for small account accounts
    pub base_price: Uint128,
    /// Base delegated tokens for an account. Used to calculate minimum required to mint a name
    pub base_delegation: Uint128,
    /// # of seconds to delay allowing minting to occur from contract creation. Defaults to 1 second
    pub mint_start_delay: Option<u64>,
}

#[cw_ownable_execute]
#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// Mint a account and list on Bitsong Account Marketplace
    MintAndList { account: String },
    /// Admin can pause minting during whitelist switching
    Pause { pause: bool },
    /// Update config, only callable by admin
    UpdateConfig { config: Config },
}

#[cw_serde]
pub enum SudoMsg {
    UpdateParams {
        min_account_length: u32,
        max_account_length: u32,
        base_price: Uint128,
        base_delegation: Uint128,
    },
    UpdateAccountCollection {
        collection: String,
    },
    UpdateAccountMarketplace {
        marketplace: String,
    },
}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    #[returns(::cw_ownable::Ownership::<::cosmwasm_std::Addr>)]
    Ownership {},
    #[returns(Addr)]
    Collection {},
    #[returns(SudoParams)]
    Params {},
    #[returns(Config)]
    Config {},
}
