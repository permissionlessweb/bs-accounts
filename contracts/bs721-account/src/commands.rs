use crate::error::ContractError;

use crate::msg::Bs721AccountsQueryMsg;
use crate::state::{SudoParams, ACCOUNT_MARKETPLACE, REVERSE_MAP, SUDO_PARAMS, VERIFIER};
use crate::Bs721AccountContract;
use cosmwasm_std::{
    ensure, Addr, Binary, ContractInfoResponse, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdError, StdResult,
};
use cw_ownable::Ownership;
use cw_utils::nonpayable;

use bs721_base::msg::ExecuteMsg as Bs721ExecuteMsg;
use btsg_account::{TextRecord, MAX_TEXT_LENGTH, NFT};

use subtle_encoding::bech32;

pub mod manifest {
    use bs721::Expiration;
    use bs721_base::state::TokenInfo;
    use btsg_account::Metadata;
    use cosmwasm_std::{to_json_binary, WasmMsg};

    use super::*;

    pub fn associate_address(
        deps: DepsMut,
        info: MessageInfo,
        account: String,
        address: Option<String>,
    ) -> Result<Response, ContractError> {
        only_owner(deps.as_ref(), &info.sender, &account)?;

        // println!("// 1. remove old token_uri from reverse map if it exists");
        Bs721AccountContract::default()
            .tokens
            .load(deps.storage, &account)
            .map(|prev_token_info| {
                if let Some(address) = prev_token_info.token_uri {
                    REVERSE_MAP.remove(deps.storage, &Addr::unchecked(address));
                }
            })?;

        // println!("// 2. validate the new address");
        let token_uri = address
            .clone()
            .map(|address| {
                deps.api
                    .addr_validate(&address)
                    .map(|addr| validate_address(deps.as_ref(), &info.sender, addr))?
            })
            .transpose()?;

        // println!("// 3. look up prev account if it exists for the new address");
        let old_account = token_uri
            .clone()
            .and_then(|addr| REVERSE_MAP.may_load(deps.storage, &addr).unwrap_or(None));

        // println!("// 4. remove old token_uri / address from previous account");
        old_account.map(|token_id| {
            Bs721AccountContract::default()
                .tokens
                .update(deps.storage, &token_id, |token| match token {
                    Some(mut token_info) => {
                        token_info.token_uri = None;
                        Ok(token_info)
                    }
                    None => Err(ContractError::AccountNotFound {}),
                })
        });

        // println!("// 5. associate new token_uri / address with new account / token_id");
        Bs721AccountContract::default().tokens.update(
            deps.storage,
            &account,
            |token| match token {
                Some(mut token_info) => {
                    token_info.token_uri = token_uri.clone().map(|addr| addr.to_string());
                    Ok(token_info)
                }
                None => Err(ContractError::AccountNotFound {}),
            },
        )?;

        // println!("// 6. update new manager in token metadata");
        // println!("// 7. save new reverse map entry");

        token_uri.map(|addr| REVERSE_MAP.save(deps.storage, &addr, &account));

        let mut event = Event::new("associate-address")
            .add_attribute("account", account)
            .add_attribute("owner", info.sender);

        if let Some(address) = address {
            event = event.add_attribute("address", address);
        }

        Ok(Response::new().add_event(event))
    }

    pub fn execute_mint(
        deps: DepsMut,
        info: MessageInfo,
        msg: Bs721ExecuteMsg<Metadata>,
    ) -> Result<Response, ContractError> {
        let minter = Bs721AccountContract::default().minter.load(deps.storage)?;
        if info.sender != minter {
            return Err(ContractError::UnauthorizedMinter {});
        }

        let (token_id, owner, _token_uri, extension, _seller_fee_bps, _payment_addr) = match msg {
            Bs721ExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
                extension,
                seller_fee_bps,
                payment_addr,
            } => (
                token_id,
                owner,
                token_uri,
                extension,
                seller_fee_bps,
                payment_addr,
            ),
            _ => return Err(ContractError::NotImplemented {}),
        };

        // create the token
        let token = TokenInfo {
            owner: deps.api.addr_validate(&owner)?,
            approvals: vec![],
            token_uri: None,
            extension,
            seller_fee_bps: None,
            payment_addr: None,
        };

        Bs721AccountContract::default().tokens.update(
            deps.storage,
            &token_id,
            |old| match old {
                Some(_) => Err(ContractError::Base(bs721_base::ContractError::Claimed {})),
                None => Ok(token),
            },
        )?;

        Bs721AccountContract::default().increment_tokens(deps.storage)?;

        let event = Event::new("mint")
            .add_attribute("minter", info.sender)
            .add_attribute("token_id", &token_id)
            .add_attribute("owner", &owner);
        Ok(Response::new().add_event(event))
    }

    /// WIP Throw not implemented error
    pub fn execute_burn(
        _deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        _token_id: String,
    ) -> Result<Response, ContractError> {
        nonpayable(&info)?;
        // option1: 
        // ensure owner 
        // transfer to this contract with reply
        // on reply, call burn function, will work as owner is now this contract
        // save list of addr that has burnt their token to state

        // option2: iterate bs721 to use 
        Err(ContractError::NotImplemented {})
    }

    pub fn execute_transfer_nft(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
        token_id: String,
    ) -> Result<Response, ContractError> {
        nonpayable(&info)?;
        let recipient = deps.api.addr_validate(&recipient)?;

        let names_marketplace = ACCOUNT_MARKETPLACE.load(deps.storage)?;

        let update_ask_msg =
            _transfer_nft(deps, env, &info, &recipient, &token_id, &names_marketplace)?;

        let event = Event::new("transfer")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", recipient)
            .add_attribute("token_id", token_id);

        Ok(Response::new().add_message(update_ask_msg).add_event(event))
    }

    // Update the ask on the marketplace
    fn update_ask_on_marketplace(
        deps: Deps,
        token_id: &str,
        recipient: Addr,
    ) -> Result<WasmMsg, ContractError> {
        let msg = bs721_account_marketplace::msgs::ExecuteMsg::UpdateAsk {
            token_id: token_id.to_string(),
            seller: recipient.to_string(),
        };
        let update_ask_msg = WasmMsg::Execute {
            contract_addr: ACCOUNT_MARKETPLACE.load(deps.storage)?.to_string(),
            funds: vec![],
            msg: to_json_binary(&msg)?,
        };
        Ok(update_ask_msg)
    }

    fn reset_token_metadata_and_reverse_map(deps: &mut DepsMut, token_id: &str) -> StdResult<()> {
        let mut token = Bs721AccountContract::default()
            .tokens
            .load(deps.storage, token_id)?;

        // Reset image, records
        token.extension = Metadata::default();
        Bs721AccountContract::default()
            .tokens
            .save(deps.storage, token_id, &token)?;

        remove_reverse_mapping(deps, token_id)?;

        Ok(())
    }

    fn remove_reverse_mapping(deps: &mut DepsMut, token_id: &str) -> StdResult<()> {
        let mut token = Bs721AccountContract::default()
            .tokens
            .load(deps.storage, token_id)?;

        // remove reverse mapping if exists
        if let Some(token_uri) = token.token_uri {
            REVERSE_MAP.remove(deps.storage, &Addr::unchecked(token_uri));
            token.token_uri = None;
        }

        Bs721AccountContract::default()
            .tokens
            .save(deps.storage, token_id, &token)?;

        Ok(())
    }

    fn _transfer_nft(
        mut deps: DepsMut,
        env: Env,
        info: &MessageInfo,
        recipient: &Addr,
        token_id: &str,
        names_marketplace: &Addr,
    ) -> Result<WasmMsg, ContractError> {
        let update_ask_msg = update_ask_on_marketplace(deps.as_ref(), token_id, recipient.clone())?;

        reset_token_metadata_and_reverse_map(&mut deps, token_id)?;

        let msg = bs721_base::ExecuteMsg::TransferNft {
            recipient: recipient.to_string(),
            token_id: token_id.to_string(),
        };

        let bs721 = Bs721AccountContract::default();

        // Force account marketplace address as operator
        bs721.operators.save(
            deps.storage,
            (&info.sender, names_marketplace),
            &Expiration::Never {},
        )?;

        bs721.execute(deps, env, info.clone(), msg)?;

        Ok(update_ask_msg)
    }

    pub fn execute_send_nft(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        let contract_addr = deps.api.addr_validate(&contract)?;
        let update_ask_msg =
            update_ask_on_marketplace(deps.as_ref(), &token_id, contract_addr.clone())?;

        reset_token_metadata_and_reverse_map(&mut deps, &token_id)?;

        let msg = bs721_base::ExecuteMsg::SendNft {
            contract: contract_addr.to_string(),
            token_id: token_id.to_string(),
            msg,
        };

        Bs721AccountContract::default().execute(deps, env, info.clone(), msg)?;

        let event = Event::new("send")
            .add_attribute("sender", info.sender)
            .add_attribute("contract", contract_addr.to_string())
            .add_attribute("token_id", token_id);

        Ok(Response::new().add_message(update_ask_msg).add_event(event))
    }

    pub fn update_image_nft(
        deps: DepsMut,
        info: MessageInfo,
        account: String,
        nft: Option<NFT>,
    ) -> Result<Response, ContractError> {
        let token_id = account.clone();

        only_owner(deps.as_ref(), &info.sender, &token_id)?;
        nonpayable(&info)?;

        let mut event = Event::new("update_image_nft")
            .add_attribute("owner", info.sender.to_string())
            .add_attribute("token_id", account);

        Bs721AccountContract::default().tokens.update(
            deps.storage,
            &token_id,
            |token| match token {
                Some(mut token_info) => {
                    token_info.extension.image_nft.clone_from(&nft);
                    Ok(token_info)
                }
                None => Err(ContractError::AccountNotFound {}),
            },
        )?;

        if let Some(nft) = nft {
            event = event.add_attribute("image_nft", nft.into_json_string());
        }

        Ok(Response::new().add_event(event))
    }

    pub fn execute_add_text_record(
        deps: DepsMut,
        info: MessageInfo,
        account: String,
        mut record: TextRecord,
    ) -> Result<Response, ContractError> {
        let token_id = account;
        only_owner(deps.as_ref(), &info.sender, &token_id)?;

        let params = SUDO_PARAMS.load(deps.storage)?;
        let max_record_count = params.max_record_count;
        // new records should reset verified to None
        record.verified = None;

        nonpayable(&info)?;
        validate_record(&record)?;

        Bs721AccountContract::default().tokens.update(
            deps.storage,
            &token_id,
            |token| match token {
                Some(mut token_info) => {
                    // can not add a record with existing account
                    for r in token_info.extension.records.iter() {
                        if r.account == record.account {
                            return Err(ContractError::RecordAccountAlreadyExists {});
                        }
                    }
                    token_info.extension.records.push(record.clone());
                    // check record length
                    if token_info.extension.records.len() > max_record_count as usize {
                        return Err(ContractError::TooManyRecords {
                            max: max_record_count,
                        });
                    }
                    Ok(token_info)
                }
                None => Err(ContractError::AccountNotFound {}),
            },
        )?;

        let event = Event::new("add-text-record")
            .add_attribute("sender", info.sender)
            .add_attribute("account", token_id)
            .add_attribute("record", record.into_json_string());
        Ok(Response::new().add_event(event))
    }

    pub fn execute_remove_text_record(
        deps: DepsMut,
        info: MessageInfo,
        account: String,
        record_account: String,
    ) -> Result<Response, ContractError> {
        let token_id = account;
        only_owner(deps.as_ref(), &info.sender, &token_id)?;
        nonpayable(&info)?;

        Bs721AccountContract::default().tokens.update(
            deps.storage,
            &token_id,
            |token| match token {
                Some(mut token_info) => {
                    token_info
                        .extension
                        .records
                        .retain(|r| r.account != record_account);
                    Ok(token_info)
                }
                None => Err(ContractError::AccountNotFound {}),
            },
        )?;

        let event = Event::new("remove-text-record")
            .add_attribute("sender", info.sender)
            .add_attribute("account", token_id)
            .add_attribute("record_account", record_account);
        Ok(Response::new().add_event(event))
    }

    pub fn execute_update_text_record(
        deps: DepsMut,
        info: MessageInfo,
        account: String,
        mut record: TextRecord,
    ) -> Result<Response, ContractError> {
        let token_id = account;
        only_owner(deps.as_ref(), &info.sender, &token_id)?;
        let params = SUDO_PARAMS.load(deps.storage)?;
        let max_record_count = params.max_record_count;

        // updated records should reset verified to None
        record.verified = None;

        nonpayable(&info)?;
        validate_record(&record)?;

        Bs721AccountContract::default().tokens.update(
            deps.storage,
            &token_id,
            |token| match token {
                Some(mut token_info) => {
                    token_info
                        .extension
                        .records
                        .retain(|r| r.account != record.account);
                    token_info.extension.records.push(record.clone());
                    // check record length
                    if token_info.extension.records.len() > max_record_count as usize {
                        return Err(ContractError::TooManyRecords {
                            max: max_record_count,
                        });
                    }
                    Ok(token_info)
                }
                None => Err(ContractError::AccountNotFound {}),
            },
        )?;

        let event = Event::new("update-text-record")
            .add_attribute("sender", info.sender)
            .add_attribute("account", token_id)
            .add_attribute("record", record.into_json_string());
        Ok(Response::new().add_event(event))
    }

    pub fn execute_verify_text_record(
        deps: DepsMut,
        info: MessageInfo,
        account: String,
        record_account: String,
        result: bool,
    ) -> Result<Response, ContractError> {
        nonpayable(&info)?;
        VERIFIER.assert_admin(deps.as_ref(), &info.sender)?;

        let token_id = account;

        Bs721AccountContract::default().tokens.update(
            deps.storage,
            &token_id,
            |token| match token {
                Some(mut token_info) => {
                    if let Some(r) = token_info
                        .extension
                        .records
                        .iter_mut()
                        .find(|r| r.account == record_account)
                    {
                        r.verified = Some(result);
                    }
                    Ok(token_info)
                }
                None => Err(ContractError::AccountNotFound {}),
            },
        )?;

        let event = Event::new("verify-text-record")
            .add_attribute("sender", info.sender)
            .add_attribute("account", token_id)
            .add_attribute("record", record_account)
            .add_attribute("result", result.to_string());
        Ok(Response::new().add_event(event))
    }

    pub fn set_profile_marketplace(
        deps: DepsMut,
        info: MessageInfo,
        address: String,
    ) -> Result<Response, ContractError> {
        nonpayable(&info)?;
        // minter only function
        let minter = Bs721AccountContract::default().minter(deps.as_ref())?;
        if info.sender != minter.minter {
            return Err(ContractError::OwnershipError(
                cw_ownable::OwnershipError::NotOwner,
            ));
        }

        ACCOUNT_MARKETPLACE.save(deps.storage, &deps.api.addr_validate(&address)?)?;

        let event = Event::new("set-account-marketplace")
            .add_attribute("sender", info.sender)
            .add_attribute("address", address);
        Ok(Response::new().add_event(event))
    }

    fn only_owner(deps: Deps, sender: &Addr, token_id: &str) -> Result<Addr, ContractError> {
        let owner = Bs721AccountContract::default()
            .tokens
            .load(deps.storage, token_id)?
            .owner;

        if owner != sender {
            return Err(ContractError::OwnershipError(
                cw_ownable::OwnershipError::NotOwner,
            ));
        }

        Ok(owner)
    }

    fn validate_record(record: &TextRecord) -> Result<(), ContractError> {
        if record.verified.is_some() {
            return Err(ContractError::UnauthorizedVerification {});
        }
        let name_len = record.account.len();
        if name_len > MAX_TEXT_LENGTH as usize {
            return Err(ContractError::RecordAccountTooLong {});
        } else if name_len == 0 {
            return Err(ContractError::RecordAccountEmpty {});
        }

        if record.value.len() > MAX_TEXT_LENGTH as usize {
            return Err(ContractError::RecordValueTooLong {});
        } else if record.value.is_empty() {
            return Err(ContractError::RecordValueEmpty {});
        }
        Ok(())
    }
}

pub mod queries {

    use super::*;
    pub fn query_profile_marketplace(deps: Deps) -> StdResult<Addr> {
        ACCOUNT_MARKETPLACE.load(deps.storage)
    }

    pub fn query_account(deps: Deps, address: String) -> StdResult<String> {
        if !address.starts_with("bitsong") {
            return Err(StdError::generic_err("invalid address"));
            // todo: update to transcode if prefix is not one of the prefix we expect to have the same coin type as Bitsong (639)
            // address = transcode(&address)?;
        }

        REVERSE_MAP
            .load(deps.storage, &deps.api.addr_validate(&address)?)
            .map_err(|_| {
                StdError::generic_err(format!("No account associated with address {}", address))
            })
    }

    pub fn query_params(deps: Deps) -> StdResult<SudoParams> {
        SUDO_PARAMS.load(deps.storage)
    }

    pub fn query_associated_address(deps: Deps, account: &str) -> StdResult<String> {
        Bs721AccountContract::default()
            .tokens
            .load(deps.storage, account)?
            .token_uri
            .ok_or_else(|| StdError::generic_err("No associated address"))
    }

    pub fn query_image_nft(deps: Deps, account: &str) -> StdResult<Option<NFT>> {
        Ok(Bs721AccountContract::default()
            .tokens
            .load(deps.storage, account)?
            .extension
            .image_nft)
    }

    pub fn query_text_records(deps: Deps, account: &str) -> StdResult<Vec<TextRecord>> {
        Ok(Bs721AccountContract::default()
            .tokens
            .load(deps.storage, account)?
            .extension
            .records)
    }
    pub fn query_is_twitter_verified(deps: Deps, account: &str) -> StdResult<bool> {
        let records = Bs721AccountContract::default()
            .tokens
            .load(deps.storage, account)?
            .extension
            .records;

        for record in records {
            if record.account == "twitter" {
                return Ok(record.verified.unwrap_or(false));
            }
        }

        Ok(false)
    }
}

pub fn transcode(address: &str) -> StdResult<String> {
    let (_, data) =
        bech32::decode(address).map_err(|_| StdError::generic_err("Invalid bech32 address"))?;

    Ok(bech32::encode("bitsong", data))
}

fn validate_address(deps: Deps, sender: &Addr, addr: Addr) -> Result<Addr, ContractError> {
    // no need to validate if sender is address
    if sender == addr {
        return Ok(addr);
    }

    let contract_details = cw2::query_contract_info(&deps.querier, addr.to_string())?;
    if contract_details.contract.contains("bs721-base")
        || contract_details.contract.contains("sg721-updatable")
    {
        let collection_info: Ownership<Addr> = deps
            .querier
            .query_wasm_smart(&addr, &Bs721AccountsQueryMsg::Minter {})?;
        if let Some(ci) = collection_info.owner {
            if ci == *sender {
                return Ok(addr);
            }
        }
    }

    let ContractInfoResponse { admin, creator, .. } =
        deps.querier.query_wasm_contract_info(&addr)?;

    if let Some(admin) = admin {
        ensure!(
            &admin == sender,
            ContractError::UnauthorizedCreatorOrAdmin {}
        );
    } else {
        // If there is no admin and the creator is not the sender, check creator's admin
        let creator_info = deps.querier.query_wasm_contract_info(creator)?;
        if creator_info.admin.map_or(true, |a| &a != sender) {
            return Err(ContractError::UnauthorizedCreatorOrAdmin {});
        }
    }

    // we have a contract registration
    Ok(addr)
}

pub fn sudo_update_params(deps: DepsMut, max_record_count: u32) -> Result<Response, ContractError> {
    SUDO_PARAMS.save(deps.storage, &SudoParams { max_record_count })?;

    let event =
        Event::new("update-params").add_attribute("max_record_count", max_record_count.to_string());
    Ok(Response::new().add_event(event))
}
