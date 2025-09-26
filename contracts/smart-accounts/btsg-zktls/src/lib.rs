pub mod claims;
pub mod contract;
pub mod digest;
mod error;
mod state;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{from_json, Event, Response, Uint128};
use serde::{Deserialize, Serialize};

use crate::{
    claims::Proof,
    contract::fetch_witness_for_claim,
    state::{Epoch, Witness, EPOCHS},
};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
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

pub use crate::error::ContractError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountZkTls {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountZkTslAuthStuct {}

impl btsg_account::traits::default::BtsgAccountTrait for BtsgAccountZkTls {
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
    type SudoMsg = btsg_auth::AuthenticatorSudoMsg;

    type ContractError = ContractError;

    type AuthMethodStructs = BtsgAccountZkTslAuthStuct;

    type AuthProcessResult = Result<Response, ContractError>;

    fn extended_authenticate(
        deps: cosmwasm_std::DepsMut,
        auth: Self::AuthMethodStructs,
    ) -> Self::AuthProcessResult {
        todo!()
    }

    fn process_sudo_auth(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &Self::SudoMsg,
    ) -> Self::AuthProcessResult {
        todo!()
    }

    fn on_auth_added(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::OnAuthenticatorAddedRequest,
    ) -> Self::AuthProcessResult {
        // small storage writes, for example global contract entropy or count of registered accounts
        match req.authenticator_params {
            Some(_) => Ok(Response::new().add_attribute("action", "auth_added_req")),
            None => Err(ContractError::MissingAuthenticatorMetadata {}),
        }
    }

    fn on_auth_removed(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::OnAuthenticatorRemovedRequest,
    ) -> Self::AuthProcessResult {
        Ok(Response::new().add_attribute("action", "auth_removed_req"))
    }

    fn on_auth_request(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &Box<btsg_auth::AuthenticationRequest>,
    ) -> Self::AuthProcessResult {
        let mut resp = Response::new().add_attribute("action", "auth_req");

        let Proof {
            claimInfo,
            signedClaim,
        }: Proof = from_json(&req.signature)?;
        match EPOCHS.may_load(deps.storage, signedClaim.claim.epoch.into())? {
            Some(epoch) => {
                // Hash the claims, and verify with identifier hash
                let hashed = claimInfo.hash();
                if signedClaim.claim.identifier != hashed {
                    return Err(ContractError::HashMismatchErr {});
                }

                // Fetch witness for claim
                let expected_witness = fetch_witness_for_claim(
                    epoch,
                    signedClaim.claim.identifier.clone(),
                    env.block.time,
                );

                let expected_witness_addresses = Witness::get_addresses(expected_witness);

                // recover witness address from SignedClaims Object
                let signed_witness = signedClaim.recover_signers_of_signed_claim(deps)?;

                // make sure the minimum requirement for witness is satisfied
                if expected_witness_addresses.len() != signed_witness.len() {
                    return Err(ContractError::WitnessMismatchErr {});
                }
                // Ensure for every signature in the sign, a expected witness exists from the database
                for signed in signed_witness {
                    let signed_event = Event::new("signer").add_attribute("sig", signed.clone());
                    resp = resp.add_event(signed_event);
                    if !expected_witness_addresses.contains(&signed) {
                        return Err(ContractError::SignatureErr {});
                    }
                }
                Ok(resp)
            }
            None => return Err(ContractError::NotFoundErr {}),
        }
    }

    fn on_auth_track(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::TrackRequest,
    ) -> Self::AuthProcessResult {
        // this is where we handle any processes after authentication, regarding message contents, prep to track balances prior to msg execution, etc..
        Ok(Response::new().add_attribute("action", "track_req"))
    }

    fn on_auth_confirm(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::ConfirmExecutionRequest,
    ) -> Self::AuthProcessResult {
        // here is were we compare balances post event execution, based on data saved from sudo_track_request,etc..
        Ok(Response::new().add_attribute("action", "conf_exec_req"))
    }

    fn on_hooks(deps: cosmwasm_std::DepsMut, env: cosmwasm_std::Env) -> Self::AuthProcessResult {
        todo!()
    }
}
