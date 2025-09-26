use btsg_account::traits::default::BtsgAccountTrait;
use cosmwasm_std::{from_json, Env, Event, Response, Timestamp};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::claims::Proof;
use crate::error::ContractError;
use crate::state::{Epoch, Witness, EPOCHS};

pub mod claims;
pub mod contract;
pub mod digest;
mod error;
pub mod msg;
pub mod state;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountIrlStructs {}
pub type SudoMsg = <BtsgAccountIrl as BtsgAccountTrait>::SudoMsg;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountIrl {}
impl BtsgAccountTrait for BtsgAccountIrl {
    // TODO: implement all under one storage key as sub-keys for organizing cosmwasm contract interaction on one line.
    type InstantiateMsg = crate::msg::InstantiateMsg;
    type ExecuteMsg = crate::msg::ExecuteMsg;
    type QueryMsg = crate::msg::QueryMsg;
    type SudoMsg = btsg_auth::AuthenticatorSudoMsg;
    type ContractError = crate::error::ContractError;
    type AuthMethodStructs = BtsgAccountIrlStructs;
    type AuthProcessResult = Result<Response, ContractError>;

    fn process_sudo_auth(
        deps: cosmwasm_std::DepsMut,
        env: Env,
        msg: &Self::SudoMsg,
    ) -> Self::AuthProcessResult {
        match msg {
            Self::SudoMsg::OnAuthAdded(auth_add) => Self::on_auth_added(deps, env, &auth_add),
            Self::SudoMsg::OnAuthRemoved(auth_remove) => {
                Self::on_auth_removed(deps, env, &auth_remove)
            }
            Self::SudoMsg::Authenticate(auth_req) => Self::on_auth_request(deps, env, &auth_req),
            Self::SudoMsg::Track(track_req) => Self::on_auth_track(deps, env, &track_req),
            Self::SudoMsg::ConfirmExecution(conf_exec_req) => {
                Self::on_auth_confirm(deps, env, &conf_exec_req)
            }
        }
    }

    fn on_auth_added(
        deps: cosmwasm_std::DepsMut,
        env: Env,
        req: &btsg_auth::OnAuthenticatorAddedRequest,
    ) -> Self::AuthProcessResult {
        //TODO: check member is a part of all DAOS registering for membership check
        //TODO: register RBAM json filters for specific DAOs, if any: https://github.com/DA0-DA0/dao-contracts/blob/development/packages/cw-jsonfilter/README.md
        // small storage writes, for example global contract entropy or count of registered accounts
        match req.authenticator_params {
            Some(_) => Ok(Response::new().add_attribute("action", "auth_added_req")),
            None => Err(ContractError::MissingAuthenticatorMetadata {}),
        }
    }

    fn on_auth_removed(
        deps: cosmwasm_std::DepsMut,
        env: Env,
        req: &btsg_auth::OnAuthenticatorRemovedRequest,
    ) -> Self::AuthProcessResult {
        //TODO: remove data set
        Ok(Response::new())
    }

    fn on_auth_request(
        deps: cosmwasm_std::DepsMut,
        env: Env,
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
                let expected_witness = BtsgAccountIrl::fetch_witness_for_claim(
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
            }
            None => return Err(ContractError::NotFoundErr {}),
        }
        Ok(resp)
    }

    fn on_auth_track(
        deps: cosmwasm_std::DepsMut,
        env: Env,
        req: &btsg_auth::TrackRequest,
    ) -> Self::AuthProcessResult {
        Ok(Response::new())
    }

    fn on_auth_confirm(
        deps: cosmwasm_std::DepsMut,
        env: Env,
        req: &btsg_auth::ConfirmExecutionRequest,
    ) -> Self::AuthProcessResult {
        Ok(Response::new())
    }

    fn on_hooks(deps: cosmwasm_std::DepsMut, env: Env) -> Self::AuthProcessResult {
        Ok(Response::new())
    }

    fn extended_authenticate(
        deps: cosmwasm_std::DepsMut,
        auth: Self::AuthMethodStructs,
    ) -> Self::AuthProcessResult {
        todo!()
    }
}
impl BtsgAccountIrl {
    pub fn fetch_witness_for_claim(
        epoch: Epoch,
        identifier: String,
        timestamp: Timestamp,
    ) -> Vec<Witness> {
        let mut selected_witness = vec![];

        // Create a hash from identifier+epoch+minimum+timestamp
        let hash_str = format!(
            "{}\n{}\n{}\n{}",
            hex::encode(identifier),
            epoch.minimum_witness_for_claim_creation,
            timestamp.nanos(),
            epoch.id
        );
        let result = hash_str.as_bytes().to_vec();
        let mut hasher = Sha256::new();
        hasher.update(result);
        let hash_result = hasher.finalize().to_vec();
        let witenesses_left_list = epoch.witness;
        let mut byte_offset = 0;
        let witness_left = witenesses_left_list.len();
        for _i in 0..epoch.minimum_witness_for_claim_creation.into() {
            let random_seed = Self::generate_random_seed(hash_result.clone(), byte_offset) as usize;
            let witness_index = random_seed % witness_left;
            let witness = witenesses_left_list.get(witness_index);
            if let Some(data) = witness {
                selected_witness.push(data.clone())
            }
            byte_offset = (byte_offset + 4) % hash_result.len();
        }

        selected_witness
    }

    fn generate_random_seed(bytes: Vec<u8>, offset: usize) -> u32 {
        // Convert the hash result into a u32 using the offset
        let hash_slice = &bytes[offset..offset + 4];
        let mut seed = 0u32;
        for (i, &byte) in hash_slice.iter().enumerate() {
            seed |= u32::from(byte) << (i * 8);
        }

        seed
    }
}
