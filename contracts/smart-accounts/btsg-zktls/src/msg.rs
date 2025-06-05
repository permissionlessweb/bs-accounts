use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::{
    claims::Proof,
    state::{Epoch, Witness},
};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub pubkey: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    VerifyProof(ProofMsg),
    AddEpoch {
        witness: Vec<Witness>,
        minimum_witness: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAllEpochResponse)]
    GetAllEpoch {},
    #[returns(GetEpochResponse)]
    GetEpoch { id: u128 },
}

#[cw_serde]
pub struct GetAllEpochResponse {
    pub ids: Vec<u128>,
}

#[cw_serde]
pub struct GetEpochResponse {
    pub epoch: Epoch,
}

#[cw_serde]
pub struct ProofMsg {
    pub proof: Proof,
}
