use crate::{
    state::{Config, Epoch, Witness, CONFIG, EPOCHS},
    BtsgAccountZkTls, ContractError, GetAllEpochResponse, GetEpochResponse,
    {ExecuteMsg, InstantiateMsg, QueryMsg},
};
use btsg_account::traits::default::BtsgAccountTrait;

use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Timestamp, Uint128,
};

// version info for migration info
use cw2::set_contract_version;
use sha2::{Digest, Sha256};
const CONTRACT_NAME: &str = "crates.io:btsg-zktls";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        owner: msg.owner.to_string(),
        current_epoch: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetEpoch { id } => to_json_binary(&query_epoch_id(deps, id)?),
        QueryMsg::GetAllEpoch {} => to_json_binary(&query_all_epoch_ids(deps)?),
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
        ExecuteMsg::AddEpoch {
            witness,
            minimum_witness,
        } => add_epoch(deps, env, witness, minimum_witness, info.sender.clone()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(
    deps: DepsMut,
    env: Env,
    msg: <BtsgAccountZkTls as BtsgAccountTrait>::SudoMsg,
) -> Result<Response, ContractError> {
    BtsgAccountZkTls::process_sudo_auth(deps, env, &msg)
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
    let hash_result: [u8; 32] = Sha256::digest(&result).to_vec().try_into().unwrap();

    let witenesses_left_list = epoch.witness;
    let mut byte_offset = 0;
    let witness_left = witenesses_left_list.len();
    for _i in 0..epoch.minimum_witness_for_claim_creation.into() {
        let random_seed = generate_random_seed(hash_result.to_vec(), byte_offset) as usize;
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
