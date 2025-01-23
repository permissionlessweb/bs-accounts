use btsg_account::minter::{Config, SudoParams};
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sp");

pub const ACCOUNT_COLLECTION: Item<Addr> = Item::new("ac");

pub const ACCOUNT_MARKETPLACE: Item<Addr> = Item::new("am");

/// Controls if minting is paused or not by admin
pub const PAUSED: Item<bool> = Item::new("paused");

pub const CONFIG: Item<Config> = Item::new("config");