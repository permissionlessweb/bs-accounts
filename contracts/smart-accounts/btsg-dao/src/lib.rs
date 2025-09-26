use crate::error::ContractError;
use cosmwasm_std::Response;
use serde::{Deserialize, Serialize};

pub mod contract;
mod error;
pub mod msg;
pub mod state;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountDaoStructs {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountDao {}
impl btsg_account::traits::default::BtsgAccountTrait for BtsgAccountDao {
    type InstantiateMsg = crate::msg::InstantiateMsg;
    type ExecuteMsg = crate::msg::ExecuteMsg;
    type QueryMsg = crate::msg::QueryMsg;
    type SudoMsg = btsg_auth::AuthenticatorSudoMsg;
    type ContractError = crate::error::ContractError;
    type AuthMethodStructs = BtsgAccountDaoStructs;
    type AuthProcessResult = Result<Response, ContractError>;

    fn process_sudo_auth(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &Self::SudoMsg,
    ) -> Self::AuthProcessResult {
        match req {
            btsg_auth::AuthenticatorSudoMsg::OnAuthAdded(req) => {
                Self::on_auth_added(deps, env, req)
            }
            btsg_auth::AuthenticatorSudoMsg::OnAuthRemoved(req) => {
                Self::on_auth_removed(deps, env, req)
            }
            btsg_auth::AuthenticatorSudoMsg::Authenticate(req) => {
                Self::on_auth_request(deps, env, req)
            }
            btsg_auth::AuthenticatorSudoMsg::Track(req) => Self::on_auth_track(deps, env, req),
            btsg_auth::AuthenticatorSudoMsg::ConfirmExecution(req) => {
                Self::on_auth_confirm(deps, env, req)
            }
        }
    }

    fn extended_authenticate(
        deps: cosmwasm_std::DepsMut,
        auth: Self::AuthMethodStructs,
    ) -> Self::AuthProcessResult {
        todo!()
    }

    fn on_auth_added(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::OnAuthenticatorAddedRequest,
    ) -> Self::AuthProcessResult {
        todo!()
    }

    fn on_auth_removed(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::OnAuthenticatorRemovedRequest,
    ) -> Self::AuthProcessResult {
        todo!()
    }

    fn on_auth_request(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &Box<btsg_auth::AuthenticationRequest>,
    ) -> Self::AuthProcessResult {
        todo!()
    }

    fn on_auth_track(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::TrackRequest,
    ) -> Self::AuthProcessResult {
        todo!()
    }

    fn on_auth_confirm(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::ConfirmExecutionRequest,
    ) -> Self::AuthProcessResult {
        todo!()
    }

    fn on_hooks(deps: cosmwasm_std::DepsMut, env: cosmwasm_std::Env) -> Self::AuthProcessResult {
        todo!()
    }
}
