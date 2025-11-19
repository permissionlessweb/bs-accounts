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
    type SudoMsg = btsg_auth::AuthSudoMsg;
    type ContractError = crate::error::ContractError;
    type AuthMethodStructs = BtsgAccountDaoStructs;
    type AuthProcessResult = Result<Response, ContractError>;

    fn process_sudo_auth(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &Self::SudoMsg,
    ) -> Self::AuthProcessResult {
        match req {
            btsg_auth::AuthSudoMsg::OnAuthAdded(req) => {
                Self::on_auth_added(deps, env, req)
            }
            btsg_auth::AuthSudoMsg::OnAuthRemoved(req) => {
                Self::on_auth_removed(deps, env, req)
            }
            btsg_auth::AuthSudoMsg::Authenticate(req) => {
                Self::on_auth_request(deps, env, req)
            }
            btsg_auth::AuthSudoMsg::Track(req) => Self::on_auth_track(deps, env, req),
            btsg_auth::AuthSudoMsg::ConfirmExecution(req) => {
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

        /// Check if passed dao membership registration in json string form
        /// 
        /// check if dao member
        /// 
        /// save auth by addr prefix to binary object (to be typed-defined later)
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
        // ensure still dao-member (raw-request)

        // check msg involves dao-goodie bag?

        todo!()
    }

    fn on_auth_track(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::TrackRequest,
    ) -> Self::AuthProcessResult {
        // ?
        todo!()
    }

    fn on_auth_confirm(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::ConfirmExecutionRequest,
    ) -> Self::AuthProcessResult {
                // ?
        todo!()
    }

    fn on_hooks(deps: cosmwasm_std::DepsMut, env: cosmwasm_std::Env) -> Self::AuthProcessResult {
        todo!()
    }
}
