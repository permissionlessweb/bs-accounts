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
    if let Some(a) = minter_params {
        // Store fantoken info (denom will be updated in reply)
        let fantoken_info = FantokenInfo {
            symbol: a.symbol.clone(),
            name: a.name.clone(),
            max_supply: a.max_supply,
            authority: env.contract.address.to_string(),
            uri: a.uri.clone(),
            minter: env.contract.address.to_string(),
            denom: String::new(), // Will be set in reply
        };
        // Create the fantoken creation message
        let create_msg = AnyMsg {
            type_url: "/bitsong.fantoken.v1beta1.MsgIssue".into(),
            value: to_json_binary(&CreateFantokenMsg {
                symbol: a.symbol.clone(),
                name: a.name.clone(),
                max_supply: a.max_supply.to_string(),
                authority: env.contract.address.to_string(),
                uri: a.uri.clone(),
                minter: env.contract.address.to_string(),
            })?,
        };
        FANTOKEN_INFO.save(deps.storage, &fantoken_info)?;
        submsg.push(SubMsg::reply_on_success(
            create_msg,
            FANTOKEN_CREATE_REPLY_ID,
        ))
    }

    CONFIG.save(deps.storage, &config)?;
    WAVS_SMART_ACCOUNT.save(deps.storage, &info.sender.to_string())?;
    ZKTLS_ENABLED.save(deps.storage, &enable_zktls)?;

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

    //Increment Epoch number
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

fn handle_mint_fantokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_requests: Vec<MintTicketObject>,
) -> Result<Response, ContractError> {
    // Ensure only the smart account can mint fantokens

    if info.sender.to_string() != WAVS_SMART_ACCOUNT.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }
    let fantoken_info = FANTOKEN_INFO.load(deps.storage)?;
    if fantoken_info.denom.is_empty() {
        return Err(ContractError::FantokenNotCreated {});
    }

    let mut mint_msgs = Vec::new();
    let mut total_to_mint = Uint128::zero();

    for mint_request in mint_requests {
        let amount = Uint128::from(mint_request.amount);
        total_to_mint += amount;

        let mint_msg = AnyMsg {
            type_url: "/bitsong.fantoken.v1beta1.MsgMint".into(),
            value: to_json_binary(&MintFantokenMsg {
                recipient: mint_request.ticket, // recipient address
                coin: coin(amount.u128(), fantoken_info.denom.clone()),
                minter: env.contract.address.to_string(),
            })?,
        };

        mint_msgs.push(mint_msg);
    }

    // Check if minting would exceed max supply
    let current_minted = MINTED_AMOUNTS
        .may_load(deps.storage, &fantoken_info.denom)?
        .unwrap_or_default();

    if current_minted + total_to_mint > fantoken_info.max_supply {
        return Err(ContractError::ExceedsMaxSupply {});
    }

    // Update minted amount
    MINTED_AMOUNTS.save(
        deps.storage,
        &fantoken_info.denom,
        &(current_minted + total_to_mint),
    )?;

    Ok(Response::new().add_messages(mint_msgs))
}

fn handle_set_uri(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_uri: String,
) -> Result<Response, ContractError> {
    // Ensure only the smart account can update URI
    if info.sender.to_string() != WAVS_SMART_ACCOUNT.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut fantoken_info = FANTOKEN_INFO.load(deps.storage)?;

    if fantoken_info.denom.is_empty() {
        return Err(ContractError::FantokenNotCreated {});
    }

    // Update stored URI
    fantoken_info.uri = new_uri.clone();
    FANTOKEN_INFO.save(deps.storage, &fantoken_info)?;

    let set_uri_msg = AnyMsg {
        type_url: "/bitsong.fantoken.v1beta1.MsgSetUri".into(),
        value: to_json_binary(&SetFantokenUriMsg {
            denom: fantoken_info.denom,
            uri: new_uri,
            authority: env.contract.address.to_string(),
        })?,
    };

    Ok(Response::new().add_message(set_uri_msg))
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
        None => Err(StdError::generic_err("No such epoch")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match ZKTLS_ENABLED.load(deps.storage)? {
        true => match msg {
            btsg_auth::AuthenticatorSudoMsg::OnAuthenticatorAdded(auth_add) => {
                sudo_on_authenticator_added_request(deps, auth_add)
            }
            btsg_auth::AuthenticatorSudoMsg::OnAuthenticatorRemoved(auth_remove) => {
                sudo_on_authenticator_removed_request(deps, auth_remove)
            }
            btsg_auth::AuthenticatorSudoMsg::Authenticate(auth_req) => {
                sudo_authentication_request(deps, env, auth_req)
            }
            btsg_auth::AuthenticatorSudoMsg::Track(track_req) => {
                sudo_track_request(deps, track_req)
            }
            btsg_auth::AuthenticatorSudoMsg::ConfirmExecution(conf_exec_req) => {
                sudo_confirm_execution_request(deps, conf_exec_req)
            }
        },
        false => Ok(Response::new()),
    }
}

fn sudo_on_authenticator_added_request(
    _deps: DepsMut,
    auth_added: OnAuthenticatorAddedRequest,
) -> Result<Response, ContractError> {
    // small storage writes, for example global contract entropy or count of registered accounts
    match auth_added.authenticator_params {
        Some(_) => Ok(Response::new().add_attribute("action", "auth_added_req")),
        None => Err(ContractError::MissingAuthenticatorMetadata {}),
    }
}

fn sudo_on_authenticator_removed_request(
    _deps: DepsMut,
    _auth_removed: OnAuthenticatorRemovedRequest,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "auth_removed_req"))
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

pub fn fetch_witness_for_claim(
    epoch: Epoch,
    identifier: String,
    timestamp: Timestamp,
) -> Vec<Witness> {
    let mut selected_witness = vec![];

    // Create a hash from identifier+epoch+minimum+timestamp
    let hash_str = format!(
        "{}\n{}\n{}\n{}",
        hex::encode(identifier),
        epoch.minimum_witness_for_claim_creation,
        timestamp.nanos(),
        epoch.id
    );
    let result = hash_str.as_bytes().to_vec();
    let mut hasher = Sha256::new();
    hasher.update(result);
    let hash_result = hasher.finalize().to_vec();
    let witenesses_left_list = epoch.witness;
    let mut byte_offset = 0;
    let witness_left = witenesses_left_list.len();
    for _i in 0..epoch.minimum_witness_for_claim_creation.into() {
        let random_seed = generate_random_seed(hash_result.clone(), byte_offset) as usize;
        let witness_index = random_seed % witness_left;
        let witness = witenesses_left_list.get(witness_index);
        if let Some(data) = witness {
            selected_witness.push(data.clone())
        }
        byte_offset = (byte_offset + 4) % hash_result.len();
    }

    selected_witness
}

fn generate_random_seed(bytes: Vec<u8>, offset: usize) -> u32 {
    // Convert the hash result into a u32 using the offset
    let hash_slice = &bytes[offset..offset + 4];
    let mut seed = 0u32;
    for (i, &byte) in hash_slice.iter().enumerate() {
        seed |= u32::from(byte) << (i * 8);
    }

    seed
}

fn sudo_track_request(
    _deps: DepsMut,
    TrackRequest { .. }: TrackRequest,
) -> Result<Response, ContractError> {
    // this is where we handle any processes after authentication, regarding message contents, prep to track balances prior to msg execution, etc..
    Ok(Response::new().add_attribute("action", "track_req"))
}

fn sudo_confirm_execution_request(
    _deps: DepsMut,
    _confirm_execution_req: ConfirmExecutionRequest,
) -> Result<Response, ContractError> {
    // here is were we compare balances post event execution, based on data saved from sudo_track_request,etc..
    Ok(Response::new().add_attribute("action", "conf_exec_req"))
}
