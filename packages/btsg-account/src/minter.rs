#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub public_mint_start_time: cosmwasm_std::Timestamp,
}

#[cosmwasm_schema::cw_serde]
pub struct SudoParams {
    /// 3 (same as DNS)
    pub min_account_length: u32,
    /// 63 (same as DNS)
    pub max_account_length: u32,
    /// 100_000_000 (5+ ASCII char price)
    pub base_price: cosmwasm_std::Uint256,
    /// 100_000_000 (5+ ASCII char price)
    pub base_delegation: cosmwasm_std::Uint256,
    // Fair Burn fee (rest goes to Community Pool)
    // pub fair_burn_percent: Decimal,
}

// #[cosmwasm_schema::cw_serde]
// #[derive(cosmwasm_schema::QueryResponses)]
// pub enum BsAccountMinterQueryMsg {
//     #[returns(cosmwasm_std::Addr)]
//     Admin {},
//     #[returns(Vec<cosmwasm_std::Addr>)]
//     Whitelists {},
//     #[returns(cosmwasm_std::Addr)]
//     Collection {},
//     #[returns(SudoParams)]
//     Params {},
//     #[returns(Config)]
//     Config {},
// }
