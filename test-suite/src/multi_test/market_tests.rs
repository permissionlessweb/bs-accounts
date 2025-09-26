use crate::constants::*;
use crate::suite::BtsgTestSuite;
use bs721_account::{msg::ExecuteMsg as AccountExecuteMsg, QueryMsg as AccountQueryMsg};
use bs721_account_marketplace::msgs::{
    ExecuteMsg as MarketExecuteMsg, QueryMsg as MarketQueryMsg, SudoMsg as MarketplaceSudoMsg,
};
use bs721_account_marketplace::state::{Bid, BidOffset};
use bs721_account_minter::msg::ExecuteMsg as MinterExecuteMsg;
use btsg_account::{Metadata, TextRecord};
use cosmwasm_std::{coins, Addr, Decimal, StdResult, Uint128, Uint256};
use cw_multi_test::{Executor, SudoMsg, WasmSudo};

type ExecuteFnType = bs721_account::msg::ExecuteMsg<Metadata>;

#[test]
fn test_basic_marketplace_flow() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "market";
    let owner = suite.creator.clone();
    let bidder = Addr::unchecked(BIDDER);

    // Give bidder some funds
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bidder.to_string(),
            amount: coins(BID_AMOUNT, "ubtsg"),
        },
    ))?;

    suite.mint_and_list(token_id, &owner)?;

    // Verify account is listed in marketplace
    let ask: Option<bs721_account_marketplace::state::Ask> = suite.app.wrap().query_wasm_smart(
        suite.marketplace_addr.as_ref().unwrap(),
        &MarketQueryMsg::Ask {
            token_id: token_id.to_string(),
        },
    )?;

    assert!(ask.is_some());
    assert_eq!(ask.unwrap().token_id, token_id);

    Ok(())
}

#[test]
fn test_bid_and_accept() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "bidtest";
    let owner = suite.creator.clone();
    let bidder = Addr::unchecked(BIDDER);

    // Give bidder some funds
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bidder.to_string(),
            amount: coins(BID_AMOUNT, "ubtsg"),
        },
    ))?;

    suite.mint_and_list(token_id, &owner)?;
    suite.bid_with_funds(token_id, &bidder, BID_AMOUNT)?;

    // Check initial balances
    let owner_balance_before = suite.app.wrap().query_balance(&owner, "ubtsg")?;

    // Accept the bid
    suite.accept_bid(&bidder, token_id)?;

    // Verify ownership changed
    let new_owner = suite.get_owner(token_id)?;
    assert_eq!(new_owner, bidder);

    // Verify owner received payment
    let owner_balance_after = suite.app.wrap().query_balance(&owner, "ubtsg")?;
    assert!(owner_balance_after.amount > owner_balance_before.amount);

    Ok(())
}

#[test]
fn test_multiple_bids() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "multibid";
    let owner = suite.creator.clone();
    let bidder1 = Addr::unchecked(BIDDER1);
    let bidder2 = Addr::unchecked(BIDDER2);

    // Give bidders some funds
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bidder1.to_string(),
            amount: coins(BID_AMOUNT, "ubtsg"),
        },
    ))?;

    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bidder2.to_string(),
            amount: coins(BID_AMOUNT * 2, "ubtsg"),
        },
    ))?;

    suite.mint_and_list(token_id, &owner)?;
    suite.bid_with_funds(token_id, &bidder1, BID_AMOUNT)?;
    suite.bid_with_funds(token_id, &bidder2, BID_AMOUNT * 2)?;

    // Query highest bid
    let highest_bid: Option<bs721_account_marketplace::state::Bid> =
        suite.app.wrap().query_wasm_smart(
            suite.marketplace_addr.as_ref().unwrap(),
            &MarketQueryMsg::HighestBid {
                token_id: token_id.to_string(),
            },
        )?;

    assert!(highest_bid.is_some());
    assert_eq!(highest_bid.unwrap().amount, Uint256::from(BID_AMOUNT * 2));

    Ok(())
}

#[test]
fn test_ask_queries() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let owner = suite.creator.clone();

    suite.mint_and_list("ask1", &owner)?;
    suite.mint_and_list("ask2", &owner)?;

    // Query all asks
    let asks: Vec<bs721_account_marketplace::state::Ask> = suite.app.wrap().query_wasm_smart(
        suite.marketplace_addr.as_ref().unwrap(),
        &MarketQueryMsg::Asks {
            start_after: None,
            limit: None,
        },
    )?;

    assert!(asks.len() >= 2);

    // Query asks by seller
    let seller_asks: Vec<bs721_account_marketplace::state::Ask> =
        suite.app.wrap().query_wasm_smart(
            suite.marketplace_addr.as_ref().unwrap(),
            &MarketQueryMsg::AsksBySeller {
                seller: owner.to_string(),
                start_after: None,
                limit: None,
            },
        )?;

    assert!(seller_asks.len() >= 2);

    // Query ask count
    let ask_count: u64 = suite.app.wrap().query_wasm_smart(
        suite.marketplace_addr.as_ref().unwrap(),
        &MarketQueryMsg::AskCount {},
    )?;

    assert!(ask_count >= 2);

    Ok(())
}

#[test]
fn test_two_sales_cycle() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "cycle";
    let owner = suite.creator.clone();
    let bidder1 = Addr::unchecked(BIDDER1);
    let bidder2 = Addr::unchecked(BIDDER2);

    // Give bidders funds
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bidder1.to_string(),
            amount: coins(BID_AMOUNT, "ubtsg"),
        },
    ))?;

    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bidder2.to_string(),
            amount: coins(BID_AMOUNT, "ubtsg"),
        },
    ))?;

    // First sale cycle
    suite.mint_and_list(token_id, &owner)?;
    suite.bid_with_funds(token_id, &bidder1, BID_AMOUNT)?;
    suite.accept_bid(&bidder1, token_id)?;

    // Verify bidder1 now owns the token
    let current_owner = suite.get_owner(token_id)?;
    assert_eq!(current_owner, bidder1);

    // Second sale cycle - bidder1 sells to bidder2
    suite.bid_with_funds(token_id, &bidder2, BID_AMOUNT)?;

    // Approve marketplace for bidder1's token
    suite.app.execute_contract::<ExecuteFnType>(
        bidder1.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &bs721_account::msg::ExecuteMsg::Approve {
            spender: suite.marketplace_addr.as_ref().unwrap().to_string(),
            token_id: token_id.to_string(),
            expires: None,
        },
        &[],
    )?;

    // Accept bid as bidder1
    suite.app.execute_contract(
        bidder1.clone(),
        suite.marketplace_addr.as_ref().unwrap().clone(),
        &MarketExecuteMsg::AcceptBid {
            bidder: bidder2.to_string(),
            token_id: token_id.to_string(),
        },
        &[],
    )?;

    // Verify bidder2 now owns the token
    let final_owner = suite.get_owner(token_id)?;
    assert_eq!(final_owner, bidder2);

    Ok(())
}

#[test]
fn test_check_approvals() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "approval";
    let owner = suite.creator.clone();

    suite.mint_and_list(token_id, &owner)?;

    // Check that marketplace is approved
    let operators: bs721::OperatorsResponse = suite.app.wrap().query_wasm_smart(
        suite.account_addr.as_ref().unwrap(),
        &AccountQueryMsg::AllOperators {
            owner: owner.to_string(),
            include_expired: None,
            start_after: None,
            limit: None,
        },
    )?;

    assert!(operators.operators.len() >= 1);

    Ok(())
}

#[test]
fn test_pause_functionality() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "pause";
    let user = suite.creator.clone();

    // Pause minting
    suite.app.execute_contract(
        suite.admin.clone(),
        suite.minter_addr.as_ref().unwrap().clone(),
        &MinterExecuteMsg::Pause { pause: true },
        &[],
    )?;

    // Try to mint while paused - should fail
    let result = suite.mint_and_list(token_id, &user);
    assert!(result.is_err(), "Should fail when minting is paused");

    Ok(())
}

#[test]
fn test_marketplace_sudo_updates() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    // Run sudo message to update marketplace parameters
    suite.app.sudo(SudoMsg::Wasm(WasmSudo {
        contract_addr: suite.marketplace_addr.as_ref().unwrap().clone(),
        message: cosmwasm_std::to_json_binary(&MarketplaceSudoMsg::UpdateParams {
            trading_fee_bps: Some(1000u64),
            min_price: Some(Uint128::from(1000u128)),
            ask_interval: Some(1000),
        })?,
    }))?;

    // Verify parameters were updated
    let params: bs721_account_marketplace::state::SudoParams = suite.app.wrap().query_wasm_smart(
        suite.marketplace_addr.as_ref().unwrap(),
        &MarketQueryMsg::Params {},
    )?;

    assert_eq!(params.trading_fee_percent, Decimal::percent(10));
    assert_eq!(params.min_price, Uint128::from(1000u128));
    assert_eq!(params.ask_interval, 1000);

    Ok(())
}

#[test]
fn test_delegation_tiers() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.setup_staking()?;
    suite.instantiate_marketplace()?;
    suite.instantiate_minter()?;
    suite.setup_marketplace()?;

    let base_delegation = Uint128::new(2_100_000_000);

    // Test 3-character account (requires 5x delegation)
    let user3 = Addr::unchecked(USER3);
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: user3.to_string(),
            amount: coins(21_000_000_000, "ubtsg"),
        },
    ))?;

    suite.delegate_to_validator(&user3, (base_delegation * Uint128::new(5)).u128())?;
    suite
        .app
        .update_block(|block| block.time = block.time.plus_seconds(200));
    suite.mint_and_list("abc", &user3)?;

    // Test 4-character account (requires 3x delegation)
    let user4 = Addr::unchecked(USER4);
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: user4.to_string(),
            amount: coins(21_000_000_000, "ubtsg"),
        },
    ))?;

    suite.delegate_to_validator(&user4, (base_delegation * Uint128::new(3)).u128())?;
    suite.mint_and_list("abcd", &user4)?;

    // Test 5+ character account (requires 1x delegation)
    let user5 = Addr::unchecked(USER5);
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: user5.to_string(),
            amount: coins(21_000_000_000, "ubtsg"),
        },
    ))?;

    suite.delegate_to_validator(&user5, base_delegation.u128())?;
    suite.mint_and_list("abcde", &user5)?;

    Ok(())
}

#[test]
fn test_admin_ownership_transfer() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let new_admin = Addr::unchecked(NEW_ADMIN);

    // Transfer ownership
    suite.app.execute_contract(
        suite.admin.clone(),
        suite.minter_addr.as_ref().unwrap().clone(),
        &MinterExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: new_admin.to_string(),
            expiry: None,
        }),
        &[],
    )?;

    // Accept ownership
    suite.app.execute_contract(
        new_admin.clone(),
        suite.minter_addr.as_ref().unwrap().clone(),
        &MinterExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership),
        &[],
    )?;

    // Renounce ownership
    suite.app.execute_contract(
        new_admin.clone(),
        suite.minter_addr.as_ref().unwrap().clone(),
        &MinterExecuteMsg::UpdateOwnership(cw_ownable::Action::RenounceOwnership),
        &[],
    )?;

    // Verify no owner can update (should fail)
    let result = suite.app.execute_contract(
        suite.admin.clone(),
        suite.minter_addr.as_ref().unwrap().clone(),
        &MinterExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: new_admin.to_string(),
            expiry: None,
        }),
        &[],
    );

    assert!(result.is_err(), "Should fail without owner");

    Ok(())
}

#[test]
fn test_bids_pagination() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "paginate";
    let owner = suite.creator.clone();
    let bidder1 = Addr::unchecked(BIDDER1);
    let bidder2 = Addr::unchecked(BIDDER2);

    // Give bidders funds
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bidder1.to_string(),
            amount: coins(BID_AMOUNT * 3, "ubtsg"),
        },
    ))?;

    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bidder2.to_string(),
            amount: coins(BID_AMOUNT * 2, "ubtsg"),
        },
    ))?;

    suite.mint_and_list(token_id, &owner)?;
    suite.bid_with_funds(token_id, &bidder1, BID_AMOUNT)?;
    suite.bid_with_funds(token_id, &bidder2, BID_AMOUNT * 2)?;

    // Test pagination with offset
    let filter = BidOffset::new(
        Uint128::from(BID_AMOUNT),
        token_id.to_string(),
        bidder1.clone(),
    );

    let bids: Vec<Bid> = suite.app.wrap().query_wasm_smart(
        suite.marketplace_addr.as_ref().unwrap(),
        &MarketQueryMsg::BidsForSeller {
            seller: owner.to_string(),
            start_after: Some(filter),
            limit: None,
        },
    )?;

    // Should return remaining bids after the offset
    assert!(bids.len() == 0usize);

    Ok(())
}

// #[test]
// fn test_reverse_map_functionality() -> StdResult<()> {
//     let mut suite = BtsgTestSuite::new();
//     suite.default_setup()?;

//     let token_id = "revmap";
//     let user = suite.creator.clone();

//     suite.mint_and_list(token_id, &user)?;

//     // Associate address
//     suite.app.execute_contract::<ExecuteFnType>(
//         user.clone(),
//         suite.account_addr.as_ref().unwrap().clone(),
//         &AccountExecuteMsg::AssociateAddress {
//             account: token_id.to_string(),
//             address: Some(user.to_string()),
//         },
//         &[],
//     )?;

//     // Query reverse mapping
//     let account: String = suite.app.wrap().query_wasm_smart(
//         suite.account_addr.as_ref().unwrap(),
//         &AccountQueryMsg::Account {
//             address: user.to_string(),
//         },
//     )?;

//     assert_eq!(account, token_id);

//     Ok(())
// }

#[test]
fn test_twitter_verification() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "twitter";
    let user = suite.creator.clone();
    let verifier = Addr::unchecked(VERIFIER);

    suite.mint_and_list(token_id, &user)?;

    // Add twitter record
    let record = TextRecord {
        account: "twitter".to_string(),
        value: "handle".to_string(),
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

    // Verify record
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

    // Check if twitter is verified
    let is_verified: bool = suite.app.wrap().query_wasm_smart(
        suite.account_addr.as_ref().unwrap(),
        &AccountQueryMsg::IsTwitterVerified {
            account: token_id.to_string(),
        },
    )?;

    assert!(is_verified);

    Ok(())
}

#[test]
fn test_transfer_with_bids() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "transfer-bid";
    let owner = suite.creator.clone();
    let bidder = Addr::unchecked(BIDDER);
    let new_owner = Addr::unchecked(NEW_OWNER);

    // Give bidder funds
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bidder.to_string(),
            amount: coins(BID_AMOUNT, "ubtsg"),
        },
    ))?;

    suite.mint_and_list(token_id, &owner)?;
    suite.bid_with_funds(token_id, &bidder, BID_AMOUNT)?;

    // Transfer NFT
    suite.app.execute_contract::<ExecuteFnType>(
        owner.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::TransferNft {
            recipient: new_owner.to_string(),
            token_id: token_id.to_string(),
        },
        &[],
    )?;

    // Verify new owner
    let current_owner = suite.get_owner(token_id)?;
    assert_eq!(current_owner, new_owner);

    // New owner can accept bids
    suite.app.execute_contract::<ExecuteFnType>(
        new_owner.clone(),
        suite.account_addr.as_ref().unwrap().clone(),
        &AccountExecuteMsg::Approve {
            spender: suite.marketplace_addr.as_ref().unwrap().to_string(),
            token_id: token_id.to_string(),
            expires: None,
        },
        &[],
    )?;

    suite.app.execute_contract(
        new_owner.clone(),
        suite.marketplace_addr.as_ref().unwrap().clone(),
        &MarketExecuteMsg::AcceptBid {
            bidder: bidder.to_string(),
            token_id: token_id.to_string(),
        },
        &[],
    )?;

    Ok(())
}

// TODO: commented out as our test suite does not ovverride default address prefixes, specifically on app.store_code.
// #[test]
// fn test_transfer_with_reverse_map_cleanup() -> StdResult<()> {
//     let mut suite = BtsgTestSuite::new();
//     suite.default_setup()?;

//     let token_id = "cleanup";
//     let owner = suite.creator.clone();
//     let new_owner = Addr::unchecked(NEW_OWNER);

//     suite.mint_and_list(token_id, &owner)?;

//     // Associate address
//     suite.app.execute_contract::<ExecuteFnType>(
//         owner.clone(),
//         suite.account_addr.as_ref().unwrap().clone(),
//         &AccountExecuteMsg::AssociateAddress {
//             account: token_id.to_string(),
//             address: Some(owner.to_string()),
//         },
//         &[],
//     )?;

//     // Verify mapping exists
//     let account: String = suite.app.wrap().query_wasm_smart(
//         suite.account_addr.as_ref().unwrap(),
//         &AccountQueryMsg::Account {
//             address: owner.to_string(),
//         },
//     )?;
//     assert_eq!(account, token_id);

//     // Transfer NFT
//     suite.app.execute_contract::<ExecuteFnType>(
//         owner.clone(),
//         suite.account_addr.as_ref().unwrap().clone(),
//         &AccountExecuteMsg::TransferNft {
//             recipient: new_owner.to_string(),
//             token_id: token_id.to_string(),
//         },
//         &[],
//     )?;

//     // Verify reverse mapping is cleaned up
//     let result: Result<String, _> = suite.app.wrap().query_wasm_smart(
//         suite.account_addr.as_ref().unwrap(),
//         &AccountQueryMsg::Account {
//             address: owner.to_string(),
//         },
//     );

//     assert!(result.is_err(), "Reverse mapping should be cleaned up");

//     Ok(())
// }
