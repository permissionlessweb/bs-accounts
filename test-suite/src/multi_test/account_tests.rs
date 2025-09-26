use crate::constants::*;
use crate::suite::BtsgTestSuite;

use bs721_account::{msg::ExecuteMsg as AccountExecuteMsg, QueryMsg};
use btsg_account::verify_generic::{
    preamble_msg_arb_036, pubkey_to_address, CosmosArbitrary, TestCosmosArb,
};
use btsg_account::{Metadata, TextRecord, NFT};
use cosmwasm_std::{Addr, Binary, StdResult};
use cw_multi_test::Executor;
use ecdsa::signature::rand_core::OsRng;
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use sha2::digest::Update;
use sha2::{Digest, Sha256};

type ExecuteFnType = bs721_account::msg::ExecuteMsg<Metadata>;

#[test]
fn test_mint_and_update_account() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "enterprise";
    let user = suite.creator.clone();

    // Mint account
    suite.mint_and_list(token_id, &user)?;

    // Verify ownership
    let owner = suite.get_owner(token_id)?;
    assert_eq!(owner, user);

    // Test updating image NFT
    let new_nft = NFT {
        collection: Addr::unchecked("contract"),
        token_id: "token_id".to_string(),
    };

    suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::UpdateImageNft {
            account: token_id.to_string(),
            nft: Some(new_nft.clone()),
        },
        &[],
    )?;

    // Test adding text record
    let text_record = TextRecord {
        account: "test".to_string(),
        value: "test_value".to_string(),
        verified: None,
    };

    suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::AddTextRecord {
            account: token_id.to_string(),
            record: text_record.clone(),
        },
        &[],
    )?;

    // Query NFT info to verify
    let nft_info: bs721::NftInfoResponse<Metadata> = suite.app.wrap().query_wasm_smart(
        suite.account_addr.as_ref().unwrap(),
        &QueryMsg::NftInfo {
            token_id: token_id.to_string(),
        },
    )?;

    assert_eq!(nft_info.extension.records.len(), 1);
    assert_eq!(nft_info.extension.records[0].account, "test");

    Ok(())
}

#[test]
fn test_burn_account() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "burntest";
    let user = suite.creator.clone();

    suite.mint_and_list(token_id, &user)?;

    // Burn the token
    suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::Burn {
            token_id: token_id.to_string(),
        },
        &[],
    )?;

    // Verify token is burned by trying to query owner (should fail)
    let result = suite.get_owner(token_id);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_reverse_mapping() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "reverse";
    let user = suite.creator.clone();

    suite.mint_and_list(token_id, &user)?;

    // Associate address with account
    suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::AssociateAddress {
            account: token_id.to_string(),
            address: Some(user.to_string()),
        },
        &[],
    )?;

    // Query associated address
    let associated: String = suite.app.wrap().query_wasm_smart(
        suite.account_addr.as_ref().unwrap(),
        &QueryMsg::AssociatedAddress {
            account: token_id.to_string(),
        },
    )?;

    assert_eq!(associated, user.to_string());

    Ok(())
}

#[test]
fn test_text_record_verification() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "verify";
    let user = suite.creator.clone();
    let verifier = Addr::unchecked(VERIFIER);

    suite.mint_and_list(token_id, &user)?;

    // Add text record
    let text_record = TextRecord {
        account: "twitter".to_string(),
        value: "test_handle".to_string(),
        verified: None,
    };

    suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::AddTextRecord {
            account: token_id.to_string(),
            record: text_record.clone(),
        },
        &[],
    )?;

    // Verify the text record as verifier
    suite.app.execute_contract::<ExecuteFnType>(
        verifier.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::VerifyTextRecord {
            account: token_id.to_string(),
            record_account: "twitter".to_string(),
            result: true,
        },
        &[],
    )?;

    // Query to verify it's been verified
    let nft_info: bs721::NftInfoResponse<Metadata> = suite.app.wrap().query_wasm_smart(
        suite.account_addr.as_ref().unwrap(),
        &QueryMsg::NftInfo {
            token_id: token_id.to_string(),
        },
    )?;

    assert_eq!(nft_info.extension.records[0].verified, Some(true));

    Ok(())
}

#[test]
fn test_transfer_account() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "transfer";
    let user = suite.creator.clone();
    let recipient = Addr::unchecked(RECIPIENT);

    suite.mint_and_list(token_id, &user)?;

    // Transfer NFT
    suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::TransferNft {
            recipient: recipient.to_string(),
            token_id: token_id.to_string(),
        },
        &[],
    )?;

    // Verify new ownership
    let owner = suite.get_owner(token_id)?;
    assert_eq!(owner, recipient);

    Ok(())
}

#[test]
fn test_unauthorized_minter() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let not_minter = Addr::unchecked(NOT_MINTER);
    let token_id = "enterprise";

    // Try to mint with unauthorized minter - should fail
    let result = suite.app.execute_contract(
        not_minter.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::Mint {
            owner: not_minter.to_string(),
            token_id: token_id.to_string(),
            token_uri: None,
            extension: Metadata::default(),
            seller_fee_bps: None,
            payment_addr: None,
        },
        &[],
    );

    assert!(result.is_err(), "Should fail with unauthorized minter");

    Ok(())
}

#[test]
fn test_record_count_limits() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "limits";
    let user = suite.creator.clone();

    suite.mint_and_list(token_id, &user)?;

    // Get max record count from params
    let params: bs721_account::state::SudoParams = suite
        .app
        .wrap()
        .query_wasm_smart(suite.account_addr.as_ref().unwrap(), &QueryMsg::Params {})?;

    let max_record_count = params.max_record_count;

    // Add records up to the limit
    for i in 0..max_record_count {
        let record = TextRecord {
            account: format!("key{}", i),
            value: "value".to_string(),
            verified: None,
        };

        suite.app.execute_contract::<ExecuteFnType>(
            user.clone(),
            suite.account_addr.as_ref().unwrap().clone(),
            &AccountExecuteMsg::AddTextRecord {
                account: token_id.to_string(),
                record,
            },
            &[],
        )?;
    }

    // Try to add one more - should fail with TooManyRecords
    let record = TextRecord {
        account: format!("key{}", max_record_count),
        value: "value".to_string(),
        verified: None,
    };

    let result = suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::AddTextRecord {
            account: token_id.to_string(),
            record,
        },
        &[],
    );

    assert!(result.is_err(), "Should fail with TooManyRecords");

    Ok(())
}

#[test]
fn test_unauthorized_text_record_operations() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "unauthorized";
    let owner = suite.creator.clone();
    let unauthorized = Addr::unchecked(UNAUTHORIZED);

    suite.mint_and_list(token_id, &owner)?;

    let record = TextRecord {
        account: "test".to_string(),
        value: "value".to_string(),
        verified: None,
    };

    // Try to add text record as non-owner - should fail
    let result = suite.app.execute_contract::<ExecuteFnType>(
        unauthorized.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::AddTextRecord {
            account: token_id.to_string(),
            record,
        },
        &[],
    );

    assert!(result.is_err(), "Should fail - not owner");

    Ok(())
}

#[test]
fn test_duplicate_record_handling() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "duplicate";
    let user = suite.creator.clone();

    suite.mint_and_list(token_id, &user)?;

    let record = TextRecord {
        account: token_id.to_string(),
        value: "twitter".to_string(),
        verified: None,
    };

    // Add first record
    suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::AddTextRecord {
            account: token_id.to_string(),
            record: record.clone(),
        },
        &[],
    )?;

    // Try to add duplicate - should fail with RecordAccountAlreadyExists
    let duplicate_record = TextRecord {
        account: token_id.to_string(),
        value: "twitter".to_string(),
        verified: None,
    };

    let result = suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::AddTextRecord {
            account: token_id.to_string(),
            record: duplicate_record,
        },
        &[],
    );

    assert!(
        result.is_err(),
        "Should fail with RecordAccountAlreadyExists"
    );

    Ok(())
}

#[test]
fn test_complex_reverse_mapping_with_crypto() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "complex";
    let user = suite.creator.clone();
    let hrp = "cosmos";

    suite.mint_and_list(token_id, &user)?;

    // Create multiple cosmos addresses with cryptographic proofs
    let mut carbs = vec![];
    for _ in 0..5 {
        let secret_key: SigningKey = SigningKey::random(&mut OsRng);
        let public_key: VerifyingKey = VerifyingKey::from(&secret_key);
        let base64btsgaddr = &Binary::new(user.as_bytes().to_vec()).to_base64();
        let hraddr = pubkey_to_address(public_key.to_encoded_point(false).as_bytes(), hrp)?;
        let adr036msgtohash = preamble_msg_arb_036(&hraddr.to_string(), base64btsgaddr);
        let msg_digest = Sha256::new().chain(&adr036msgtohash);
        let msg_hash = msg_digest.clone().finalize();

        let signature: Signature = secret_key.sign_prehash_recoverable(&msg_hash).unwrap().0;

        // Verify signature
        assert!(cosmwasm_crypto::secp256k1_verify(
            &msg_hash,
            signature.to_bytes().as_slice(),
            public_key.to_encoded_point(false).as_bytes()
        )
        .unwrap());

        let cosmosarb = CosmosArbitrary {
            pubkey: Binary::from(public_key.to_encoded_point(false).as_bytes()),
            signature: Binary::from(signature.to_bytes().as_slice()),
            message: Binary::from(user.as_bytes().to_vec()),
            hrp: Some(hrp.to_string()),
        };

        cosmosarb.verify_return_readable()?;

        carbs.push(TestCosmosArb {
            carb: cosmosarb,
            pk: Binary::new(secret_key.to_bytes().to_vec()),
        });
    }

    // Test reverse mapping with multiple addresses
    suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::UpdateMyReverseMapKey {
            to_add: carbs.iter().map(|c| c.carb.clone()).collect(),
            to_remove: vec![],
        },
        &[],
    )?;

    // Verify reverse mappings work
    for carb in &carbs {
        let addr = pubkey_to_address(&carb.carb.pubkey, carb.carb.hrp.as_ref().unwrap())?;

        let mapped_addr: String = suite.app.wrap().query_wasm_smart(
            suite.account_addr.as_ref().unwrap(),
            &QueryMsg::ReverseMapAddress {
                address: addr.to_string(),
            },
        )?;

        assert_eq!(mapped_addr, user.to_string());
    }

    Ok(())
}

#[test]
fn test_reverse_map_limits() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "maplimits";
    let user = suite.creator.clone();

    suite.mint_and_list(token_id, &user)?;

    // Create too many addresses (more than limit)
    let mut carbs = vec![];
    for _ in 0..15 {
        // Assuming limit is 10
        let secret_key: SigningKey = SigningKey::random(&mut OsRng);
        let public_key: VerifyingKey = VerifyingKey::from(&secret_key);
        let base64btsgaddr = &Binary::new(user.as_bytes().to_vec()).to_base64();
        let hraddr = pubkey_to_address(public_key.to_encoded_point(false).as_bytes(), "cosmos")?;
        let adr036msgtohash = preamble_msg_arb_036(&hraddr.to_string(), base64btsgaddr);
        let msg_digest = Sha256::new().chain(&adr036msgtohash);
        let msg_hash = msg_digest.clone().finalize();

        let signature: Signature = secret_key.sign_prehash_recoverable(&msg_hash).unwrap().0;

        let cosmosarb = CosmosArbitrary {
            pubkey: Binary::from(public_key.to_encoded_point(false).as_bytes()),
            signature: Binary::from(signature.to_bytes().as_slice()),
            message: Binary::from(user.as_bytes().to_vec()),
            hrp: Some("cosmos".to_string()),
        };

        carbs.push(TestCosmosArb {
            carb: cosmosarb,
            pk: Binary::new(secret_key.to_bytes().to_vec()),
        });
    }

    // Try to add too many at once - should fail with TooManyReverseMaps
    let result = suite.app.execute_contract::<ExecuteFnType>(
        user.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::UpdateMyReverseMapKey {
            to_add: carbs.iter().map(|c| c.carb.clone()).collect(),
            to_remove: vec![],
        },
        &[],
    );

    assert!(result.is_err(), "Should fail with TooManyReverseMaps");

    Ok(())
}

#[test]
fn test_account_query_errors() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let nonexistent_addr = Addr::unchecked("nonexistent");

    // Query account for address that doesn't have one - should fail
    let result: Result<String, _> = suite.app.wrap().query_wasm_smart(
        suite.account_addr.as_ref().unwrap(),
        &QueryMsg::Account {
            address: nonexistent_addr.to_string(),
        },
    );

    assert!(
        result.is_err(),
        "Should fail for nonexistent account mapping"
    );

    Ok(())
}
