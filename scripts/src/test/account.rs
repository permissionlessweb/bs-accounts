use ::bs721_account::{commands::transcode, ContractError};
use abstract_interface::{Abstract, AccountI};
use abstract_sdk::std::objects::gov_type::GovernanceDetails;
use bs721_account::msg::{Bs721AccountsQueryMsgFns, ExecuteMsgFns};
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{from_json, StdError};
use cw_orch::prelude::CallAs;
use cw_orch::{anyhow, mock::MockBech32, prelude::*};
use queriers::Gov;
use std::error::Error;

use crate::BtsgAccountSuite;

/// creates an account with specific governance parameters
pub fn create_default_account<T: CwEnv>(
    sender: &Addr,
    abstr: &Abstract<T>,
    gov_details: GovernanceDetails<String>,
) -> anyhow::Result<AccountI<T>> {
    let account = AccountI::create_default_account(abstr, gov_details)?;
    Ok(account)
}

#[test]
fn init() -> anyhow::Result<()> {
    // new mock Bech32 chain environment
    let mock = MockBech32::new("mock");
    // simulate deploying the test suite to the mock chain env.
    BtsgAccountSuite::deploy_on(mock.clone(), mock.sender)?;
    Ok(())
}
#[test]
fn mint_and_update() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let deps = mock_dependencies();
    let mut suite = BtsgAccountSuite::new(mock.clone());
    suite.default_setup(mock.clone(), None, None)?;

    let not_minter = mock.addr_make("not-minter");
    let minter = suite.minter.address()?;

    // retrieve max record count
    let params = suite.account.params()?;
    let max_record_count = params.max_record_count;

    // mint token
    let token_id = "Enterprise";

    let err = suite
        .account
        .call_as(&not_minter)
        .mint(
            btsg_account::Metadata::default(),
            not_minter.clone(),
            token_id,
            None,
            None,
            None,
        )
        .unwrap_err();

    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::UnauthorizedMinter {}.to_string()
    );

    suite.account.call_as(&minter).mint(
        btsg_account::Metadata::default(),
        mock.sender,
        token_id,
        None,
        None,
        None,
    )?;

    // check token contains correct metadata
    let res = suite.account.nft_info(token_id)?;

    assert_eq!(res.token_uri, None);
    assert_eq!(res.extension, btsg_account::Metadata::default());

    // update image
    let new_nft = btsg_account::NFT::new(
        deps.as_ref(),
        deps.api.addr_make("checkout").to_string(),
        "64".into(),
    )?;
    let nft_value = suite
        .account
        .update_image_nft(token_id, Some(new_nft.clone()))?
        .event_attr_value("wasm-update_image_nft", "image_nft")?
        .into_bytes();
    let nft: btsg_account::NFT = from_json(nft_value).unwrap();
    assert_eq!(nft, new_nft);

    // add text record
    let new_record = btsg_account::TextRecord {
        account: "test".to_string(),
        value: "test".to_string(),
        verified: None,
    };
    let record_value = suite
        .account
        .update_text_record(token_id.to_string(), new_record.clone())?
        .event_attr_value("wasm-update-text-record", "record")?
        .into_bytes();

    let record: btsg_account::TextRecord = from_json(record_value)?;
    assert_eq!(record, new_record);
    let records = suite.account.text_records(token_id)?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].account, "test");
    assert_eq!(records[0].value, "test");

    assert!(!suite.account.is_twitter_verified(token_id)?);

    // trigger too many records error
    for i in 1..=(max_record_count) {
        let new_record = btsg_account::TextRecord {
            account: format!("key{:?}", i),
            value: "value".to_string(),
            verified: None,
        };
        if i == max_record_count {
            let res = suite.account.update_text_record(token_id, new_record);
            assert_eq!(
                res.unwrap_err().root().to_string(),
                ContractError::TooManyRecords {
                    max: max_record_count
                }
                .to_string()
            );
            break;
        } else {
            suite.account.update_text_record(token_id, new_record)?;
        }
    }

    // rm text records
    suite.account.remove_text_record(token_id, "test")?;

    for i in 1..=(max_record_count) {
        let record_account = format!("key{:?}", i);
        suite.account.remove_text_record(token_id, record_account)?;
    }
    // txt record count should be 0
    let res = suite.account.nft_info(token_id)?;
    assert_eq!(res.extension.records.len(), 0);

    // unauthorized add txt record
    let err = suite
        .account
        .call_as(&not_minter)
        .add_text_record(token_id, new_record.clone())
        .unwrap_err();
    assert_eq!(
        err.root().to_string(),
        ContractError::Base(bs721_base::ContractError::Ownership(
            cw_ownable::OwnershipError::NotOwner
        ))
        .to_string()
    );
    // passes
    suite.account.add_text_record(token_id, new_record)?;
    assert_eq!(suite.account.nft_info(token_id)?.extension.records.len(), 1);

    // add another txt record
    let record = btsg_account::TextRecord {
        account: "twitter".to_string(),
        value: "jackdorsey".to_string(),
        verified: None,
    };
    suite.account.add_text_record(token_id, record)?;
    assert_eq!(suite.account.nft_info(token_id)?.extension.records.len(), 2);

    // add duplicate record RecordAccountAlreadyExists
    let record = btsg_account::TextRecord {
        account: "test".to_string(),
        value: "testtesttest".to_string(),
        verified: None,
    };
    assert_eq!(
        suite
            .account
            .add_text_record(token_id, record.clone())
            .unwrap_err()
            .root()
            .to_string(),
        ContractError::RecordAccountAlreadyExists {}.to_string()
    );
    // update txt record
    suite.account.update_text_record(token_id, record.clone())?;
    let res = suite.account.nft_info(token_id)?;
    assert_eq!(res.extension.records.len(), 2);
    assert_eq!(res.extension.records[1].value, record.value);
    // rm txt record
    suite.account.remove_text_record(token_id, record.account)?;
    let res = suite.account.nft_info(token_id)?;
    assert_eq!(res.extension.records.len(), 1);

    Ok(())
}
#[test]
fn test_query_accounts() -> anyhow::Result<()> {
    let mock = MockBech32::new("bitsong");
    // setup addresses
    let creator = mock.addr_make("babber");
    let owner = mock.addr_make("jroc");
    let mut suite = BtsgAccountSuite::new(mock.clone());
    let mut accounts = Abstract::deploy_on(mock.clone(), ())?;

    suite.default_setup(mock.clone(), Some(creator.clone()), Some(owner.clone()))?;

    for i in 0..200 {
        let tokenid = "babber-".to_owned() + &i.to_string();
        suite.mint_and_list(mock.clone(), &tokenid, &creator)?;
        // create account with nft used as governance ownership
        let btsg_account = create_default_account(
            &creator.clone(),
            &accounts,
            GovernanceDetails::NFT {
                collection_addr: suite.account.addr_str()?,
                token_id: tokenid.clone(),
            },
        )?;

        // associate account addr
        suite
            .account
            .associate_address(tokenid, true, Some(btsg_account.to_string()))?;
    }

    let mock_deps = mock_dependencies();
    // cannot query mapping of unregistered address
    assert_eq!(
        suite
            .account
            .account(owner.clone().to_string())
            .unwrap_err()
            .to_string(),
        StdError::GenericErr {
            msg: format!(
                "Querier contract error: Generic error: No account associated with address {}",
                owner
            ),
            backtrace: todo!()
        }
        .to_string()
    );

    //  change ownership of account to new nft collection
    //  assert
    Ok(())
}
#[test]
fn test_transcode() -> anyhow::Result<()> {
    let res = transcode("cosmos1y54exmx84cqtasvjnskf9f63djuuj68p7hqf47");
    assert_eq!(
        res.unwrap(),
        "bitsong1y54exmx84cqtasvjnskf9f63djuuj68pj7jph3"
    );
    Ok(())
}
