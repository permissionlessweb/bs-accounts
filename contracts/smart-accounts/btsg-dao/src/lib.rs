use cosmwasm_std::Response;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;

// pub mod claims;
// pub mod contract;
// pub mod digest;
// mod error;
// pub mod msg;
// pub mod state;

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
        msg: &Self::SudoMsg,
    ) -> Self::AuthProcessResult {
        match msg {
            Self::SudoMsg::OnAuthAdded(auth_add) => Self::on_auth_added(deps, &auth_add),
            Self::SudoMsg::OnAuthRemoved(auth_remove) => {
                Self::on_auth_removed(deps, &auth_remove)
            }
            Self::SudoMsg::Authenticate(auth_req) => Self::on_auth_request(deps, &auth_req),
            Self::SudoMsg::Track(track_req) => Self::on_auth_track(deps, &track_req),
            Self::SudoMsg::ConfirmExecution(conf_exec_req) => {
                Self::on_auth_confirm(deps, &conf_exec_req)
            }
        }
    }

    fn on_auth_added(
        deps: cosmwasm_std::DepsMut,
        req: &btsg_auth::OnAuthenticatorAddedRequest,
    ) -> Self::AuthProcessResult {
        //TODO: check member is a part of all DAOS registering for membership check
        //TODO: register RBAM json filters for specific DAOs, if any: https://github.com/DA0-DA0/dao-contracts/blob/development/packages/cw-jsonfilter/README.md
        Ok(Response::new())
    }

    fn on_auth_removed(
        deps: cosmwasm_std::DepsMut,
        req: &btsg_auth::OnAuthenticatorRemovedRequest,
    ) -> Self::AuthProcessResult {
        //TODO: remove data set
        Ok(Response::new())
    }

    fn on_auth_request(
        deps: cosmwasm_std::DepsMut,
        req: &Box<btsg_auth::AuthenticationRequest>,
    ) -> Self::AuthProcessResult {
        Ok(Response::new())
    }

    fn on_auth_track(
        deps: cosmwasm_std::DepsMut,
        req: &btsg_auth::TrackRequest,
    ) -> Self::AuthProcessResult {
        Ok(Response::new())
    }

    fn on_auth_confirm(
        deps: cosmwasm_std::DepsMut,
        req: &btsg_auth::ConfirmExecutionRequest,
    ) -> Self::AuthProcessResult {
        Ok(Response::new())
    }

    fn on_hooks(deps: cosmwasm_std::DepsMut) -> Self::AuthProcessResult {
        Ok(Response::new())
    }
}
