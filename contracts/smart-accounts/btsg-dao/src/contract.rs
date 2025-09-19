use crate::claims::Proof;
use crate::error::ContractError;
use crate::msg::{
    CreateFantokenMsg, ExecuteMsg, FantokenInfo, GetAllEpochResponse, GetEpochResponse,
    InstantiateMsg, MintFantokenMsg, MintTicketObject, QueryMsg, SetFantokenUriMsg, SudoMsg,
};
use crate::state::{
    Config, Epoch, Witness, CONFIG, EPOCHS, FANTOKEN_INFO, MINTED_AMOUNTS, WAVS_SMART_ACCOUNT,
    ZKTLS_ENABLED,
};
use crate::BtsgAccountDao;
use bitsong_rs::types::bitsong::fantoken::v1beta1::*;
use btsg_account::traits::default::BtsgAccountTrait;
use btsg_auth::{
    AuthenticationRequest, ConfirmExecutionRequest, OnAuthenticatorAddedRequest,
    OnAuthenticatorRemovedRequest, TrackRequest,
};
use cosmwasm_std::{
    coin, from_json, to_json_binary, Addr, AnyMsg, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, Uint128,
};
use cosmwasm_std::{entry_point, Event, Timestamp};
use cw2::set_contract_version;
use sha2::{Digest, Sha256};

// Constants
const CONTRACT_NAME: &str = "crates.io:btsg-irl";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const FANTOKEN_CREATE_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    InstantiateMsg {
        enable_zktls,
        minter_params,
    }: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut submsg = vec![];
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        owner: info.sender.to_string(),
        current_epoch: Uint128::zero(),
    };

    Ok(Response::new()
        .add_submessages(submsg)
        .add_attribute("method", "instantiate")
        .add_attribute("smart_account", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        FANTOKEN_CREATE_REPLY_ID => {
            let mut fantoken_info = FANTOKEN_INFO.load(deps.storage)?;
            fantoken_info.denom = format!("ft{}", fantoken_info.symbol.to_lowercase());

            FANTOKEN_INFO.save(deps.storage, &fantoken_info)?;

            Ok(Response::new()
                .add_attribute("method", "fantoken_created")
                .add_attribute("denom", &fantoken_info.denom))
        }
        _ => Err(ContractError::UnknownReplyId {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintFantokens { data } => handle_mint_fantokens(deps, env, info, data),
        ExecuteMsg::SetUri { uri } => handle_set_uri(deps, env, info, uri),
        ExecuteMsg::AddEpoch {
            witness,
            minimum_witness,
        } => add_epoch(deps, env, witness, minimum_witness, info.sender.clone()),
    }
}

// @dev - add epoch
pub fn add_epoch(
    deps: DepsMut,
    env: Env,
    witness: Vec<Witness>,
    minimum_witness: Uint128,
    sender: Addr,
) -> Result<Response, ContractError> {
    // load configs
    let mut config = CONFIG.load(deps.storage)?;

    if config.owner != sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }

    // Increment Epoch number
    let new_epoch = config.current_epoch + Uint128::one();
    // Create the new epoch
    let epoch = Epoch {
        id: new_epoch,
        witness,
        timestamp_start: env.block.time.nanos(),
        timestamp_end: env.block.time.plus_seconds(86400).nanos(),
        minimum_witness_for_claim_creation: minimum_witness,
    };

    // Upsert the new epoch into memory
    EPOCHS.save(deps.storage, new_epoch.into(), &epoch)?;

    // Save the new epoch
    config.current_epoch = new_epoch;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}
  

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetFantokenInfo {} => {
            let info = FANTOKEN_INFO.may_load(deps.storage)?;
            to_json_binary(&info)
        }
        QueryMsg::GetMintedAmount {} => {
            let fantoken_info = FANTOKEN_INFO.load(deps.storage)?;
            let minted = MINTED_AMOUNTS
                .may_load(deps.storage, &fantoken_info.denom)?
                .unwrap_or_default();
            to_json_binary(&minted)
        }
        QueryMsg::GetEpoch { id } => to_json_binary(&query_epoch_id(deps, id)?),
        QueryMsg::GetAllEpoch {} => to_json_binary(&query_all_epoch_ids(deps)?),
    }
}

//NOTE: Unimplemented as secret doesn't allow to iterate via keys
fn query_all_epoch_ids(_deps: Deps) -> StdResult<GetAllEpochResponse> {
    Ok(GetAllEpochResponse { ids: vec![] })
}

fn query_epoch_id(deps: Deps, id: u128) -> StdResult<GetEpochResponse> {
    match EPOCHS.may_load(deps.storage, id)? {
        Some(epoch) => Ok(GetEpochResponse { epoch }),
        None => Err(StdError::msg("No such epoch")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match ZKTLS_ENABLED.load(deps.storage)? {
        true => BtsgAccountDao::process_sudo_auth(deps, &msg),
        false => Ok(Response::new()),
    }
}

fn sudo_authentication_request(
    deps: DepsMut,
    env: Env,
    auth_req: Box<AuthenticationRequest>,
) -> Result<Response, ContractError> {
    // let pubkeys = WAVS_PUBKEY.load(deps.storage)?;
    let mut resp = Response::new().add_attribute("action", "auth_req");

    // if auth_req.signature_data.signers.len() != pubkeys.len() {
    //     return Err(ContractError::InvalidPubkeyCount { a, b });
    // }

    let Proof {
        claimInfo,
        signedClaim,
    }: Proof = from_json(&auth_req.signature)?;
    match EPOCHS.may_load(deps.storage, signedClaim.claim.epoch.into())? {
        Some(epoch) => {
            // Hash the claims, and verify with identifier hash
            let hashed = claimInfo.hash();
            if signedClaim.claim.identifier != hashed {
                return Err(ContractError::HashMismatchErr {});
            }

            // Fetch witness for claim
            let expected_witness = fetch_witness_for_claim(
                epoch,
                signedClaim.claim.identifier.clone(),
                env.block.time,
            );

            let expected_witness_addresses = Witness::get_addresses(expected_witness);

            // recover witness address from SignedClaims Object
            let signed_witness = signedClaim.recover_signers_of_signed_claim(deps)?;

            // make sure the minimum requirement for witness is satisfied
            if expected_witness_addresses.len() != signed_witness.len() {
                return Err(ContractError::WitnessMismatchErr {});
            }

            // Ensure for every signature in the sign, a expected witness exists from the database
            for signed in signed_witness {
                let signed_event = Event::new("signer").add_attribute("sig", signed.clone());
                resp = resp.add_event(signed_event);
                if !expected_witness_addresses.contains(&signed) {
                    return Err(ContractError::SignatureErr {});
                }
            }
        }
        None => return Err(ContractError::NotFoundErr {}),
    }

    // // EXAMPLE IMPLEMENTATION FOR BLS12_381 VERIFICATION

    Ok(resp)
}
