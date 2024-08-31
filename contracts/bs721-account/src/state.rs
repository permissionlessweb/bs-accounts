use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

use crate::msg::SudoParams;

pub type TokenUri = Addr;
pub type TokenId = String;

/// Address of the text record verification oracle
pub const REVERSE_MAP: Map<&TokenUri, TokenId> = Map::new("rm");
pub const VERIFIER: Admin = Admin::new("v");
pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sp");
pub const ACCOUNT_MARKETPLACE: Item<Addr> = Item::new("am");
