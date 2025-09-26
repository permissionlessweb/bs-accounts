pub mod contract;
mod error;
mod state;
pub use crate::error::ContractError;
use crate::state::PUBLIC_KEY;
use btsg_account::traits::default::BtsgAccountTrait;
use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Addr, Env, Response};

use saa::{EthPersonalSign, Verifiable};
use serde::{Deserialize, Serialize};

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
#[derive(QueryResponses)]
pub enum QueryMsg {}
pub type SudoMsg = <BtsgAccountEth as BtsgAccountTrait>::SudoMsg;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountEthStructs {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountEth {}
impl BtsgAccountTrait for BtsgAccountEth {
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
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
        //TODO: check member is a part of all DAOS registering for membership check
        //TODO: register RBAM json filters for specific DAOs, if any: https://github.com/DA0-DA0/dao-contracts/blob/development/packages/cw-jsonfilter/README.md
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
        todo!()
    }
}
