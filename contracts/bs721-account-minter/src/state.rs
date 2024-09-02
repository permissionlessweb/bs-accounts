use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::Item;

#[cosmwasm_schema::cw_serde]
pub struct SudoParams {
    /// 3 (same as DNS)
    pub min_account_length: u32,
    /// 63 (same as DNS)
    pub max_account_length: u32,
    /// 100_000_000 (5+ ASCII char price)
    pub base_price: Uint128,
    // Fair Burn fee (rest goes to Community Pool)
    // pub fair_burn_percent: Decimal,
}

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub public_mint_start_time: Timestamp,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sp");

pub const ACCOUNT_COLLECTION: Item<Addr> = Item::new("ac");

pub const ACCOUNT_MARKETPLACE: Item<Addr> = Item::new("am");

pub const ADMIN: Admin = Admin::new("a");

/// Controls if minting is paused or not by admin
pub const PAUSED: Item<bool> = Item::new("paused");

pub const CONFIG: Item<Config> = Item::new("config");
