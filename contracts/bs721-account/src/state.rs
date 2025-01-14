use cosmwasm_std::Addr;
use bs_controllers::Admin;
use cw_storage_plus::{Item, Map};

pub type TokenUri = Addr;
pub type TokenId = String;

#[cosmwasm_schema::cw_serde]
pub struct SudoParams {
    pub max_record_count: u32,
    // pub registry_addr: Addr,
}

/// Address of the text record verification oracle
pub const REVERSE_MAP: Map<&TokenUri, TokenId> = Map::new("rm");
pub const VERIFIER: Admin = Admin::new("v");
pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sp");
pub const ACCOUNT_MARKETPLACE: Item<Addr> = Item::new("am");
