use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::DAO;
use crate::BtsgAccountDao;

use btsg_account::traits::default::BtsgAccountTrait;

use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

// Constants
const CONTRACT_NAME: &str = "crates.io:btsg-irl";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const FANTOKEN_CREATE_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    InstantiateMsg { dao_addr }: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut submsg = vec![];
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_submessages(submsg)
        .add_attribute("method", "instantiate")
        .add_attribute("smart_account", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDao {} => to_json_binary(&DAO.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(
    deps: DepsMut,
    env: Env,
    req: <BtsgAccountDao as BtsgAccountTrait>::SudoMsg,
) -> Result<Response, ContractError> {
    BtsgAccountDao::process_sudo_auth(deps, env, &req)
}
