mod error;
mod state;
use saa::Ed25519;

pub use crate::error::ContractError;

use {btsg_account::traits::default::BtsgAccountTrait, saa::Verifiable};

use {
    cosmwasm_schema::{cw_serde, QueryResponses},
    cosmwasm_std::{
        entry_point, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    },
    cw2::set_contract_version,
    cw_storage_plus::Item,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<Addr>,
    /// binary representation of pubkey
    pub pubkey: Binary,
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

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:btsg-ed25519";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PUBLIC_KEY: Item<Binary> = Item::new("pk");

/// Can only be called by governance
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(msg.owner.unwrap_or(info.sender).as_str()),
    )?;

    assert!(msg.pubkey.len() <= 64);
    PUBLIC_KEY.save(deps.storage, &msg.pubkey)?;
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
pub fn sudo(
    deps: DepsMut,
    env: Env,
    msg: <BtsgAccountEd25519 as BtsgAccountTrait>::SudoMsg,
) -> Result<Response, ContractError> {
    BtsgAccountEd25519::process_sudo_auth(deps, env, &msg)
}

impl btsg_account::traits::default::BtsgAccountTrait for BtsgAccountEd25519 {
    type InstantiateMsg = crate::InstantiateMsg;
    type ExecuteMsg = crate::ExecuteMsg;
    type QueryMsg = crate::QueryMsg;
    type SudoMsg = btsg_auth::AuthSudoMsg;
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
        let params = req.authenticator_params.clone();
        let reg = params.expect("needs pubkey binary");
        match PUBLIC_KEY.may_load(deps.storage)? {
            Some(pk) => {
                if reg != pk {
                    return Err(ContractError::Unauthorized {});
                }
            }
            None => {
                assert!(reg.len() <= 64);
                PUBLIC_KEY.save(deps.storage, &reg)?
            }
        }
        Ok(Response::new())
    }

    fn on_auth_removed(
        deps: cosmwasm_std::DepsMut,
        env: Env,
        req: &btsg_auth::OnAuthenticatorRemovedRequest,
    ) -> Self::AuthProcessResult {
        PUBLIC_KEY.remove(deps.storage);
        Ok(Response::new())
    }

    fn on_auth_request(
        deps: cosmwasm_std::DepsMut,
        env: Env,
        req: &Box<btsg_auth::AuthenticationRequest>,
    ) -> Self::AuthProcessResult {
        Ed25519 {
            message: req.sign_mode_tx_data.sign_mode_direct.clone(),
            signature: req.signature.clone(),
            pubkey: PUBLIC_KEY.load(deps.storage)?,
        }
        .verify(deps.as_ref())?;
        Ok(Response::new().add_attribute("action", "auth_req"))
    }

    fn on_auth_track(
        deps: cosmwasm_std::DepsMut,
        env: Env,
        req: &btsg_auth::TrackRequest,
    ) -> Self::AuthProcessResult {
        // no op for stateful saves of ed25519 actions
        Ok(Response::new())
    }

    fn on_auth_confirm(
        deps: cosmwasm_std::DepsMut,
        env: Env,
        req: &btsg_auth::ConfirmExecutionRequest,
    ) -> Self::AuthProcessResult {
        // no op for post auth stateful actions
        Ok(Response::new())
    }

    fn on_hooks(deps: cosmwasm_std::DepsMut, env: Env) -> Self::AuthProcessResult {
        // no op for post auth stateful actions
        Ok(Response::new())
    }

    fn extended_authenticate(
        deps: cosmwasm_std::DepsMut,
        auth: Self::AuthMethodStructs,
    ) -> Self::AuthProcessResult {
        // no op for post auth stateful actions
        Ok(Response::new())
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

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::mock_dependencies;
    use saa::EthPersonalSign;
    use saa_common::Verifiable;

    #[test]
    fn eth_personal_verifiable() {
        let deps = mock_dependencies();

        let message = r#"{"chain_id":"elgafar-1","contract_address":"stars1gjgfp9wps9c0r3uqhr0xxfgu02rnzcy6gngvwpm7a78j7ykfqquqr2fuj4","messages":["Create TBA account"],"nonce":"0"}"#;
        let address = "0xac03048da6065e584d52007e22c69174cdf2b91a";
        let base = "eyJjaGFpbl9pZCI6ImVsZ2FmYXItMSIsImNvbnRyYWN0X2FkZHJlc3MiOiJzdGFyczFnamdmcDl3cHM5YzByM3VxaHIweHhmZ3UwMnJuemN5NmduZ3Z3cG03YTc4ajd5a2ZxcXVxcjJmdWo0IiwibWVzc2FnZXMiOlsiQ3JlYXRlIFRCQSBhY2NvdW50Il0sIm5vbmNlIjoiMCJ9";
        let message = cosmwasm_std::Binary::new(message.as_bytes().to_vec());
        assert!(message.to_base64() == base, "not euqal");

        let signature = cosmwasm_std::Binary::from_base64(
            "a/lQuaTyhcTEeRA2XFTPxoDSIdS3yUUH1VSKOm2zz5EURfheGzzLgXea6QAalswOM2njnUzblqIGiOC0P+j2rhw="
        ).unwrap();

        let cred = EthPersonalSign {
            signer: address.to_string(),
            signature: signature.clone(),
            message,
        };
        let res = cred.verify(deps.as_ref());
        println!("Res: {:?}", res);
        assert!(res.is_ok())
    }
}
