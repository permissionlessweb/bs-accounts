use cosmwasm_std::Binary;
use cw_storage_plus::Item;

/// Stores the contract's configuration
pub const WAVS_PUBKEY: Item<BlsMetadata> = Item::new("wavs");

#[cosmwasm_schema::cw_serde]
pub struct BlsMetadata {
    pub operator_keys: Vec<Binary>,
    pub threshold: usize,
}
