use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

use crate::msg::FantokenInfo;

// Storage
pub const ZKTLS_ENABLED: Item<bool> = Item::new("zktls");
pub const WAVS_SMART_ACCOUNT: Item<String> = Item::new("wsa");
pub const FANTOKEN_INFO: Item<FantokenInfo> = Item::new("fantoken_info");
pub const MINTED_AMOUNTS: Map<&str, Uint128> = Map::new("minted_amounts");
pub const EPOCHS: Map<u128, Epoch> = Map::new(&"epochs");
pub const CONFIG: Item<Config> = Item::new(&"config");

#[cw_serde]
pub struct Config {
    pub owner: String,
    pub current_epoch: Uint128,
}

#[cw_serde]
pub struct Witness {
    pub address: String,
    pub host: String,
}

impl Witness {
    pub fn get_addresses(witness: Vec<Witness>) -> Vec<String> {
        let mut vec_addresses = vec![];
        for wit in witness {
            vec_addresses.push(wit.address);
        }
        return vec_addresses;
    }
}

#[cw_serde]
pub struct Epoch {
    pub id: Uint128,
    pub timestamp_start: u64,
    pub timestamp_end: u64,
    pub minimum_witness_for_claim_creation: Uint128,
    pub witness: Vec<Witness>,
}
