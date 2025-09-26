use btsg_auth::*;
use cosmwasm_std::{DepsMut, Env};

pub trait AuthMethodStructTrait {}

pub trait BtsgAccountTrait {
    type InstantiateMsg;
    type ExecuteMsg;
    type QueryMsg;
    type SudoMsg;
    type ContractError;
    /// Any custom structure to extend for authentication functionality.
    type AuthMethodStructs;
    type AuthProcessResult;

    /// Authenticate is a wrapper for the specific authentication logic a structure will implement.\
    /// **Note that this function is separate from the `on_auth_request` used in the x/smart-account authentication workflow.**
    fn extended_authenticate(deps: DepsMut, auth: Self::AuthMethodStructs) -> Self::AuthProcessResult;
    /// Routes SudoMsg into their respective authentication logic. **Use this function in the contracts Sudo entrypoint**.
    fn process_sudo_auth(deps: DepsMut, env: Env, req: &Self::SudoMsg) -> Self::AuthProcessResult;
    /// Perform specific logic on an authenticator being added to an account.
    fn on_auth_added(
        deps: DepsMut,
        env: Env,
        req: &OnAuthenticatorAddedRequest,
    ) -> Self::AuthProcessResult;
    /// Perform specific logic on an authenticator being removed from account.\
    /// *Ideally, we cleanup any internal state specific to an account for optimization*.
    fn on_auth_removed(
        deps: DepsMut,
        env: Env,
        req: &OnAuthenticatorRemovedRequest,
    ) -> Self::AuthProcessResult;
    /// Logic to authenticate or reject messages using custom authentication will live in this function.\
    /// **DO NOT use logic updates any state, it will be discarded during this step.** 
    fn on_auth_request(
        deps: DepsMut,
        env: Env,
        req: &Box<AuthenticationRequest>,
    ) -> Self::AuthProcessResult;
    /// Logic to process stateful events. We expect an authentication request to have been valid, so we can update stateful data based on the message and params.
    fn on_auth_track(deps: DepsMut, env: Env, req: &TrackRequest) -> Self::AuthProcessResult;
    /// Logic to process stateful events. We expect an authentication request to have been valid, so we can update stateful data based on the message and params.
    fn on_auth_confirm(
        deps: DepsMut,
        env: Env,
        req: &ConfirmExecutionRequest,
    ) -> Self::AuthProcessResult;
    fn on_hooks(deps: DepsMut, env: Env) -> Self::AuthProcessResult;
}
