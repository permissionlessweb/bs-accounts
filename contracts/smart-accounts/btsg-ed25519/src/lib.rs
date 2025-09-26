pub mod contract;
mod error;
mod state;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Env, Response};

use saa::{EthPersonalSign, Verifiable};

pub use crate::{error::ContractError, state::PUBLIC_KEY};

use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<Addr>,
    pub pubkey: String,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {}

#[cw_serde]
pub struct BtsgAccountEthStructs {}

#[cw_serde]
pub struct BtsgAccountEd25519 {}
impl btsg_account::traits::default::BtsgAccountTrait for BtsgAccountEd25519 {
    type InstantiateMsg = crate::InstantiateMsg;
    type ExecuteMsg = crate::ExecuteMsg;
    type QueryMsg = crate::QueryMsg;
    type SudoMsg = btsg_auth::AuthenticatorSudoMsg;
    type ContractError = ContractError;
    type AuthMethodStructs = BtsgAccountEthStructs;
    type AuthProcessResult = Result<cosmwasm_std::Response, ContractError>;

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
        Ok(Response::new())
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
        let cred = EthPersonalSign {
            message: req.sign_mode_tx_data.sign_mode_direct.clone(),
            signature: req.signature.clone(),
            signer: PUBLIC_KEY.load(deps.storage)?,
        };

        // verify ethereum personal signature
        cred.verify(deps.as_ref())?;

        Ok(Response::new().add_attribute("action", "auth_req"))
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
        Ok(Response::new())
    }
}
