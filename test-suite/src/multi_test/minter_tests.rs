use crate::suite::BtsgTestSuite;
use crate::{constants::*, BASE_PRICE};
use bs721_account_minter::{
    commands::get_ascii_cost,
    msg::{ExecuteMsg as MinterExecuteMsg, QueryMsg as MinterQueryMsg},
};
use cosmwasm_std::{coins, Addr, StdResult, Uint256};
use cw_multi_test::Executor;

#[test]
fn test_basic_minting() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let token_id = "basic";
    let user = suite.creator.clone();

    suite.mint_and_list(token_id, &user)?;

    // Verify token count increased
    let token_count: bs721::NumTokensResponse = suite.app.wrap().query_wasm_smart(
        suite.account_addr.as_ref().unwrap(),
        &bs721_account::QueryMsg::NumTokens {},
    )?;

    assert!(token_count.count >= 1);

    // Verify ownership
    let owner = suite.get_owner(token_id)?;
    assert_eq!(owner, user);

    Ok(())
}

#[test]
fn test_pricing_tiers() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let user = suite.creator.clone();

    // Test different length accounts have different pricing
    let accounts: Vec<(&str)> = vec![
        // ("ab", 100_000_000u128.into()),     // 2 char - base price
        ("abc"),   // 3 char - 100x base price
        ("abcd"),  // 4 char - 10x base price
        ("abcde"), // 5+ char - base price
    ];

    for (account) in accounts {
        let expected_price = get_ascii_cost(account.len(), BASE_PRICE.into())?;
        let user_balance_before = suite.app.wrap().query_balance(&user, "ubtsg")?;

        suite.mint_and_list(account, &user)?;

        let user_balance_after = suite.app.wrap().query_balance(&user, "ubtsg")?;
        let cost = user_balance_before.amount - user_balance_after.amount;

        assert_eq!(
            cost, expected_price,
            "Incorrect pricing for account: {}",
            account
        );
    }

    Ok(())
}

#[test]
fn test_delegation_requirements() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.setup_staking()?;
    suite.instantiate_marketplace()?;
    suite.instantiate_minter()?;
    suite.setup_marketplace()?;

    let token_id = "delegate";
    let user = Addr::unchecked(DELEGATE);

    // Give user funds but don't delegate enough
    suite.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: user.to_string(),
            amount: coins(21_000_000_000, "ubtsg"),
        },
    ))?;

    // Delegate insufficient amount
    suite.delegate_to_validator(&user, 1_000_000_000)?; // Less than required 2.1B

    // Wait for mint start delay
    suite
        .app
        .update_block(|block| block.time = block.time.plus_seconds(200));

    // Try to mint - should fail due to insufficient delegation
    let result = suite.mint_and_list(token_id, &user);
    assert!(result.is_err(), "Should fail with insufficient delegation");

    Ok(())
}

#[test]
fn test_mint_before_start_time() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    let creator = suite.creator.clone();
    suite.setup_staking()?;
    suite.instantiate_marketplace()?;
    suite.instantiate_minter()?;
    suite.setup_marketplace()?;
    suite.delegate_to_validator(&creator, 10_500_000_000)?;

    let token_id = "early";
    let user = suite.creator.clone();

    // Don't advance time - should be before mint start
    let result = suite.mint_and_list(token_id, &user);
    assert!(result.is_err(), "Should fail before mint start time");

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

    // Unpause and try again
    suite.app.execute_contract(
        suite.admin.clone(),
        suite.minter_addr.as_ref().unwrap().clone(),
        &MinterExecuteMsg::Pause { pause: false },
        &[],
    )?;

    // Should work now
    suite.mint_and_list(token_id, &user)?;
    let owner = suite.get_owner(token_id)?;
    assert_eq!(owner, user);

    Ok(())
}

#[test]
fn test_admin_functions() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    let new_admin = Addr::unchecked(NEW_ADMIN);

    // Update ownership
    suite.app.execute_contract(
        suite.admin.clone(),
        suite.minter_addr.as_ref().unwrap().clone(),
        &MinterExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: new_admin.to_string(),
            expiry: None,
        }),
        &[],
    )?;

    // Accept ownership as new admin
    suite.app.execute_contract(
        new_admin.clone(),
        suite.minter_addr.as_ref().unwrap().clone(),
        &MinterExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership),
        &[],
    )?;

    // Verify ownership changed
    let ownership: cw_ownable::Ownership<Addr> = suite.app.wrap().query_wasm_smart(
        suite.minter_addr.as_ref().unwrap(),
        &MinterQueryMsg::Ownership {},
    )?;

    if let Some(owner) = ownership.owner {
        assert_eq!(owner, new_admin);
    } else {
        panic!("No owner found");
    }

    Ok(())
}

#[test]
fn test_config_queries() -> StdResult<()> {
    let mut suite = BtsgTestSuite::new();
    suite.default_setup()?;

    // Query config
    let config: btsg_account::minter::SudoParams = suite.app.wrap().query_wasm_smart(
        suite.minter_addr.as_ref().unwrap(),
        &MinterQueryMsg::Params {},
    )?;

    assert_eq!(config.base_price, Uint256::from(100_000_000u128));
    assert_eq!(config.base_delegation, Uint256::from(2_100_000_000u128));
    assert_eq!(config.min_account_length, 3u32);
    assert_eq!(config.max_account_length, 128u32);

    Ok(())
}
