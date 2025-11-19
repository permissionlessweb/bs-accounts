mod error;
mod state;

pub use crate::error::ContractError;
use crate::state::PAYLOAD;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cosmwasm_std::Response;
use saa::types::PasskeyPayload;
use saa::{PasskeyCredential, Verifiable};
use saa_common::from_json;
use serde::{Deserialize, Serialize};

use btsg_account::traits::default::BtsgAccountTrait;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;

use cosmwasm_std::entry_point;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiateMsg {
    pub owner: Option<Addr>,
    pub payload: PasskeyPayload,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountPasskeysAuthStruct {}

pub type SudoMsg = <BtsgAccountPasskey as btsg_account::traits::default::BtsgAccountTrait>::SudoMsg;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountPasskey {}

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:btsg-passkey";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Can only be called by governance
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    PAYLOAD.save(deps.storage, &msg.payload)?;
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(msg.owner.unwrap_or(info.sender).as_str()),
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    BtsgAccountPasskey::process_sudo_auth(deps, env, &msg)
}

impl btsg_account::traits::default::BtsgAccountTrait for BtsgAccountPasskey {
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
    type SudoMsg = btsg_auth::AuthSudoMsg;
    type ContractError = crate::error::ContractError;
    type AuthMethodStructs = BtsgAccountPasskeysAuthStruct;
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
        match req.authenticator_params {
            Some(_) => Ok(Response::new().add_attribute("action", "auth_added_req")),
            None => Err(ContractError::Unauthorized {}),
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
        let cred: PasskeyCredential = from_json(&req.signature)?;
        cw_ownable::is_owner(deps.storage, &req.account)?;

        // assert client origin is same as one registered
        if let Some(origin) = PAYLOAD.load(deps.storage)?.origin {
            if cred.client_data.origin != origin {
                return Err(ContractError::Unauthorized {});
            }
        }
        cred.verify(deps.as_ref())?;

        Ok(Response::new().add_attribute("action", "auth_req"))
    }

    fn on_auth_track(
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        req: &btsg_auth::TrackRequest,
    ) -> Self::AuthProcessResult {
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

pub fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::default().add_attributes(ownership.into_attributes()))
}
