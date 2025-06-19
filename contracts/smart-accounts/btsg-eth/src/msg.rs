use btsg_auth::AuthenticatorSudoMsg;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    /// address of account to make use of this authenticator
    pub owner: Option<Addr>,
    /// ethereum wallet public key to sign
    pub pubkey: String,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {}

pub type SudoMsg = AuthenticatorSudoMsg;
