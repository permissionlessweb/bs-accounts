use btsg_auth::AuthenticatorSudoMsg;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};

#[cw_serde]
pub struct InstantiateMsg {
    /// Owner of this contract. Expected to be address that is making use of this custom authenticator.
    pub owner: Option<Addr>,
    /// Array of BLS12-381 pubkeys that are to be authorizing actions
    pub wavs_operator_pubkeys: Vec<Binary>,
    /// The minimum amount of `wavs_operator_pubkeys` required in authentication requests
    pub threshold: usize,
    /// Description of pubkey type for future support of different aggregate key algos.
    pub wavs_pubkey_type: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // TODO: implement key rotation, approved by all keys in set
}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {}
 
