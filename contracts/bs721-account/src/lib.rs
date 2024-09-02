pub mod commands;
mod error;
pub mod helpers;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use btsg_account::Metadata;
use cosmwasm_std::Empty;
use msg::Bs721AccountsQueryMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:bs721-account";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Bs721AccountContract<'a> =
    cw721_base::Cw721Contract<'a, Metadata, Empty, Empty, Bs721AccountsQueryMsg>;
pub type ExecuteMsg = crate::msg::ExecuteMsg<Metadata>;
pub type QueryMsg = Bs721AccountsQueryMsg;

pub mod entry {
    use super::*;

    use commands::sudo_update_params;
    use cw_utils::maybe_addr;
    use msg::InstantiateMsg;

    use commands::manifest::*;
    use commands::queries::*;
    use msg::SudoMsg;
    use state::VERIFIER;

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let res = Bs721AccountContract::default().instantiate(
            deps.branch(),
            env.clone(),
            info,
            msg.base_init_msg,
        )?;

        Ok(res
            .add_attribute("action", "instantiate")
            .add_attribute("bs721_account_address", env.contract.address.to_string()))
    }
    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        let api = deps.api;
        match msg {
            crate::msg::ExecuteMsg::SetMarketplace { address } => {
                set_profile_marketplace(deps, info, address)
            }
            crate::msg::ExecuteMsg::AssociateAddress { account, address } => {
                associate_address(deps, info, account, address)
            }
            crate::msg::ExecuteMsg::UpdateImageNft { account, nft } => {
                update_image_nft(deps, info, account, nft)
            }
            crate::msg::ExecuteMsg::AddTextRecord { account, record } => {
                execute_add_text_record(deps, info, account, record)
            }
            crate::msg::ExecuteMsg::RemoveTextRecord {
                account,
                record_account,
            } => execute_remove_text_record(deps, info, account, record_account),
            crate::msg::ExecuteMsg::UpdateTextRecord { account, record } => {
                execute_update_text_record(deps, info, account, record)
            }
            crate::msg::ExecuteMsg::VerifyTextRecord {
                account,
                record_account,
                result,
            } => execute_verify_text_record(deps, info, account, record_account, result),
            crate::msg::ExecuteMsg::UpdateVerifier { verifier } => {
                Ok(VERIFIER.execute_update_admin(deps, info, maybe_addr(api, verifier)?)?)
            }
            crate::msg::ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => execute_transfer_nft(deps, env, info, recipient, token_id),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => execute_send_nft(deps, env, info, contract, token_id, msg),
            ExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
                // seller_fee_bps,
                // payment_addr,
                extension,
            } => execute_mint(
                deps,
                info,
                cw721_base::ExecuteMsg::Mint {
                    token_id,
                    owner,
                    token_uri,
                    extension,
                    // seller_fee_bps,
                    // payment_addr,
                },
            ),
            ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
            _ => Bs721AccountContract::default()
                .execute(deps, env, info, msg.into())
                .map_err(|e| e.into()),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
            QueryMsg::AccountMarketplace {} => to_json_binary(&query_profile_marketplace(deps)?),
            QueryMsg::Account { address } => to_json_binary(&query_account(deps, address)?),
            QueryMsg::Verifier {} => to_json_binary(&VERIFIER.query_admin(deps)?),
            QueryMsg::AssociatedAddress { account } => {
                to_json_binary(&query_associated_address(deps, &account)?)
            }
            QueryMsg::ImageNFT { account } => to_json_binary(&query_image_nft(deps, &account)?),
            QueryMsg::TextRecords { account } => {
                to_json_binary(&query_text_records(deps, &account)?)
            }
            QueryMsg::IsTwitterVerified { account } => {
                to_json_binary(&query_is_twitter_verified(deps, &account)?)
            }
            _ => Bs721AccountContract::default().query(deps, env, msg.into()),
        }
    }

    #[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
    pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
        match msg {
            SudoMsg::UpdateParams { max_record_count } => {
                sudo_update_params(deps, max_record_count)
            }
        }
    }
}