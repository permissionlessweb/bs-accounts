use cosmwasm_std::{Timestamp, Uint128};

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
