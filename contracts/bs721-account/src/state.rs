use bs_controllers::Admin;
use cosmwasm_std::{Addr, Binary};
use cw_storage_plus::{Item, Map};

pub type TokenUri = Addr;
pub type TokenId = String;

/// maps other bech32 address to bitsong addresses
pub const REVERSE_MAP_KEY: Map<&String, Binary> = Map::new("atm");
/// Address of the text record verification oracle
pub const REVERSE_MAP: Map<&TokenUri, TokenId> = Map::new("rm");
pub const VERIFIER: Admin = Admin::new("v");
pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sp");
pub const ACCOUNT_MARKETPLACE: Item<Addr> = Item::new("am");

#[cosmwasm_schema::cw_serde]
pub struct SudoParams {
    pub max_record_count: u32,
    // pub registry_addr: Addr,
}
