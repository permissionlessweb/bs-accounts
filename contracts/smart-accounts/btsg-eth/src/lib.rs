mod error;
mod state;
pub use crate::error::ContractError;
use crate::state::PUBLIC_KEY;
use btsg_account::traits::default::BtsgAccountTrait;
use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use saa::{EthPersonalSign, Verifiable};
use serde::{Deserialize, Serialize};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:btsg-eth";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountEth {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtsgAccountEthStructs {}

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

/// Can only be called by governance
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    PUBLIC_KEY.save(deps.storage, &msg.pubkey)?;
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
    BtsgAccountEth::process_sudo_auth(deps, env, &msg)
}

impl BtsgAccountTrait for BtsgAccountEth {
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
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
