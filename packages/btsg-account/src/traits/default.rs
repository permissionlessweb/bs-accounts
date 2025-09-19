use btsg_auth::*;
use cosmwasm_std::{DepsMut, Env};

pub trait AuthMethodStructTrait {}

pub trait BtsgAccountTrait {
    type InstantiateMsg;
    type ExecuteMsg;
    type QueryMsg;
    type SudoMsg;
    type ContractError;
    type AuthMethodStructs;
    type AuthProcessResult;

    fn authenticate(deps: DepsMut, auth: Self::AuthMethodStructs) -> Self::AuthProcessResult;
    fn process_sudo_auth(deps: DepsMut, env: Env, req: &Self::SudoMsg) -> Self::AuthProcessResult;
    fn on_auth_added(
        deps: DepsMut,
        env: Env,
        req: &OnAuthenticatorAddedRequest,
    ) -> Self::AuthProcessResult;
    fn on_auth_removed(
        deps: DepsMut,
        env: Env,
        req: &OnAuthenticatorRemovedRequest,
    ) -> Self::AuthProcessResult;
    fn on_auth_request(
        deps: DepsMut,
        env: Env,
        req: &Box<AuthenticationRequest>,
    ) -> Self::AuthProcessResult;
    fn on_auth_track(deps: DepsMut, env: Env, req: &TrackRequest) -> Self::AuthProcessResult;
    fn on_auth_confirm(
        deps: DepsMut,
        env: Env,
        req: &ConfirmExecutionRequest,
    ) -> Self::AuthProcessResult;
    fn on_hooks(deps: DepsMut, env: Env) -> Self::AuthProcessResult;
}
