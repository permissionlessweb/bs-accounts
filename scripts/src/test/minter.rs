use cw_orch::{anyhow, mock::MockBech32, prelude::*};

use crate::BtsgAccountSuite;
use crate::{
    Bs721AccountsQueryMsgFns, BtsgAccountExecuteFns, BtsgAccountMarketExecuteFns,
    BtsgAccountMarketQueryFns, TestOwnershipExecuteMsgFns, TestOwnershipInitMsg,
    TestOwnershipQueryMsgFns,
};
use bs721_account_minter::msg::{ExecuteMsgFns as _, QueryMsgFns as _};
use cosmwasm_std::Uint128;
use cosmwasm_std::{coins, to_json_binary, Decimal};
use cw_orch::mock::cw_multi_test::{SudoMsg, WasmSudo};

const BID_AMOUNT: u128 = 1_000_000_000;
#[test]
pub fn init() -> anyhow::Result<()> {
    // new mock Bech32 chain environment
    let mock = MockBech32::new("mock");
    // simulate deploying the test suite to the mock chain env.
    BtsgAccountSuite::deploy_on(mock.clone(), mock.sender)?;
    Ok(())
}

mod execute {

    use bs721_account_minter::ContractError;
    use btsg_account::market::{Ask, ExecuteMsg, PendingBid, QueryMsgFns};
    use btsg_account::DEPLOYMENT_DAO;
    use cosmwasm_std::{coin, Binary};

    use super::*;

    #[test]
    fn test_check_approvals() -> anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;

        let owner = mock.sender.clone();
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &owner)?;
        // check operators
        assert_eq!(
            suite
                .account
                .all_operators(owner, None, None, None)?
                .operators
                .len(),
            1
        );

        Ok(())
    }
    #[test]
    fn test_mint() -> anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let owner = mock.sender.clone();
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &owner)?;

        // check if account is listed in marketplace
        let res = suite.market.ask(token_id.to_string())?.unwrap();
        assert_eq!(res.token_id, token_id);

        // check if token minted
        let res = suite.account.num_tokens()?;
        assert_eq!(res.count, 1);

        assert_eq!(suite.owner_of(token_id.into())?, owner.to_string());

        Ok(())
    }
    #[test]
    fn test_bid() -> anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let owner = mock.sender.clone();
        let bidder = mock.addr_make("bidder");
        let token_id = "bandura";
        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &owner)?;
        suite.bid_w_funds(mock, token_id, bidder, BID_AMOUNT)?;
        Ok(())
    }

    #[test]
    fn test_accept_bid() -> anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let owner = mock.sender.clone();
        let bidder = mock.addr_make("bidder");
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &owner)?;
        suite.bid_w_funds(mock.clone(), token_id, bidder.clone(), BID_AMOUNT)?;

        // user (owner) starts off with 0 internet funny money
        assert_eq!(
            mock.balance(&owner.clone(), Some("ubtsg".into()))?[0].amount,
            Uint128::zero()
        );

        suite.market.accept_bid(bidder.clone(), token_id.into())?;

        mock.wait_seconds(60)?;
        suite
            .market
            .call_as(&owner)
            .finalize_bid(token_id.to_string())?;

        // check if bid is removed
        assert!(suite
            .market
            .bid(bidder.to_string(), token_id.into())?
            .is_none());
        // verify that the bidder is the new owner
        assert_eq!(suite.owner_of(token_id.into())?, bidder.to_string());
        // check if user got the bid amount
        assert_eq!(
            mock.balance(&owner, Some("ubtsg".into()))?[0].amount,
            Uint128::from(BID_AMOUNT)
        );
        // confirm that a new ask was created
        let res = suite.market.ask(token_id.to_string())?.unwrap();
        assert_eq!(res.seller, bidder);
        assert_eq!(res.token_id, token_id);
        Ok(())
    }
    #[test]

    fn test_two_sales_cycles() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let owner = mock.sender.clone();
        let bidder = mock.addr_make("bidder");
        let bidder2 = mock.addr_make("bidder2");
        let token_id = "bandura";
        let market = suite.market.address()?;
        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &owner)?;
        suite.bid_w_funds(mock.clone(), token_id, bidder.clone(), BID_AMOUNT)?;
        suite.market.accept_bid(bidder.clone(), token_id.into())?;
        mock.wait_seconds(60)?;
        suite
            .market
            .call_as(&owner)
            .finalize_bid(token_id.to_string())?;
        suite.bid_w_funds(mock.clone(), token_id, bidder2.clone(), BID_AMOUNT)?;
        suite
            .account
            .call_as(&bidder)
            .approve(market, token_id, None)?;
        suite
            .market
            .call_as(&bidder)
            .accept_bid(bidder2.clone(), token_id.into())?;
        mock.wait_seconds(60)?;
        suite
            .market
            .call_as(&bidder)
            .finalize_bid(token_id.to_string())?;

        Ok(())
    }
    #[test]
    fn test_reverse_map() -> anyhow::Result<()> {
        let token_id = "bandura";

        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        mock.wait_seconds(200)?;

        let admin2 = mock.addr_make("admin2");
        mock.add_balance(&admin2, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), admin2.clone(), 10000000000u128)?;

        suite.mint_and_list(mock.clone(), token_id, &admin2)?;

        // when no associated address, query should throw error
        suite
            .account
            .call_as(&admin2)
            .associated_address(token_id)
            .unwrap_err();

        // associate owner address with account account
        suite
            .account
            .call_as(&admin2)
            .associate_address(token_id, Some(admin2.to_string()))?;

        // query associated address should return user
        assert_eq!(suite.account.associated_address(token_id)?, admin2);

        // added to get around rate limiting
        mock.wait_seconds(60)?;
        // associate another
        let account2 = "exam";
        suite.mint_and_list(mock.clone(), account2, &admin2.clone())?;

        suite
            .account
            .call_as(&admin2)
            .associate_address(account2, Some(admin2.to_string()))?;

        assert_eq!(suite.account.account(admin2)?, account2.to_string());
        Ok(())
    }

    #[test]
    fn test_reverse_map_contract_address() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn test_reverse_map_not_contract_address_admin() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;

        let not_admin = mock.addr_make_with_balance("not-admin", coins(1000000000, "ubtsg"))?;
        mock.add_balance(&not_admin, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), not_admin.clone(), 10000000000u128)?;

        let minter = suite.minter.address()?;
        let token_id = "bandura";
        println!("not-admin: {:#?}", not_admin);
        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &not_admin)?;

        suite
            .account
            .associate_address(token_id, Some(minter.to_string()))
            .unwrap_err();
        Ok(())
    }

    #[test]
    fn test_reverse_map_not_owner() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let token_id = "bandura";
        let admin2 = mock.addr_make("admin2");
        // delegate
        mock.add_balance(&admin2, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), admin2.clone(), 10000000000u128)?;

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin2)?;

        suite
            .account
            .associate_address(token_id, Some(admin2.to_string()))
            .unwrap_err();
        Ok(())
    }
    #[test]
    fn test_pause() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let token_id = "bandura";
        let admin2 = mock.addr_make("admin2");
        // delegate
        mock.add_balance(&admin2, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), admin2.clone(), 10000000000u128)?;

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin2)?;

        // pause minting
        suite.minter.pause(true)?;

        // error trying to mint
        mock.wait_seconds(200)?;
        suite
            .mint_and_list(mock.clone(), token_id, &admin2)
            .unwrap_err();

        Ok(())
    }

    #[test]
    fn test_update_mkt_sudo() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let token_id = "bandura";
        let admin2 = mock.addr_make("admin2");
        // delegate
        mock.add_balance(&admin2, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), admin2.clone(), 10000000000u128)?;

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin2)?;

        // run sudo msg
        mock.app.borrow_mut().sudo(SudoMsg::Wasm(WasmSudo {
            contract_addr: suite.market.address()?,
            message: to_json_binary(&btsg_account::market::SudoMsg::UpdateParams {
                trading_fee_bps: Some(1000u64),
                min_price: Some(Uint128::from(1000u128)),
                ask_interval: Some(1000),
                cooldown_duration: Some(69),
                cooldown_cancel_fee: Some(coin(69u128, "jerets")),
            })?,
        }))?;

        // confirm updated params
        let res = suite.market.params()?;
        assert_eq!(res.trading_fee_percent, Decimal::percent(10));
        assert_eq!(res.min_price, Uint128::from(1000u128));
        assert_eq!(res.ask_interval, 1000);
        assert_eq!(res.cooldown_duration, 69);
        assert_eq!(res.cooldown_fee, coin(69u128, "jerets"));

        Ok(())
    }

    #[test]
    fn test_cooldown_period() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        mock.wait_seconds(200)?;

        let owner = mock.sender.clone();
        mock.add_balance(&owner, vec![coin(10000000000u128, "ubtsg")])?;
        let bidder = mock.addr_make("bidder");
        let account = "account";
        suite.mint_and_list(mock.clone(), account, &owner)?;
        suite.bid_w_funds(mock.clone(), account, bidder.clone(), BID_AMOUNT)?;
        let owner_balance_a = mock.query_balance(&owner, "ubtsg")?;
        let bidder_balance_a = mock.query_balance(&bidder, "ubtsg")?;
        suite.market.accept_bid(bidder.clone(), account.into())?;
        // no funds are transfered before cooldown is over
        let owner_balance_b = mock.query_balance(&owner, "ubtsg")?;
        let bidder_balance_b = mock.query_balance(&bidder, "ubtsg")?;
        mock.wait_seconds(10)?;
        assert_eq!(owner_balance_a, owner_balance_b);
        assert_eq!(bidder_balance_a, bidder_balance_b);
        assert_eq!(
            suite.market.cooldown(account.to_string())?,
            Some(PendingBid {
                ask: Ask {
                    token_id: account.to_string(),
                    id: 1,
                    seller: owner.clone(),
                },
                new_owner: bidder.clone(),
                amount: BID_AMOUNT.into(),
                unlock_time: mock.block_info()?.time.plus_seconds(50)
            })
        );
        assert_eq!(
            suite.account.burn(account).unwrap_err().root().to_string(),
            bs721_account::ContractError::AccountCannotBeTransfered {
                reason: "Account is in cooldown".to_string()
            }
            .to_string()
        );
        assert_eq!(
            suite
                .account
                .transfer_nft(bidder.clone(), account)
                .unwrap_err()
                .root()
                .to_string(),
            bs721_account::ContractError::AccountCannotBeTransfered {
                reason: "Account is in cooldown".to_string()
            }
            .to_string()
        );
        assert_eq!(
            suite
                .account
                .send_nft(suite.minter.address()?, Binary::default(), account)
                .unwrap_err()
                .root()
                .to_string(),
            bs721_account::ContractError::AccountCannotBeTransfered {
                reason: "Account is in cooldown".to_string()
            }
            .to_string()
        );
        mock.wait_seconds(50)?;
        suite
            .market
            .call_as(&owner)
            .finalize_bid(account.to_string())?;
        assert_eq!(suite.market.cooldown(account.to_string())?, None,);
        let owner_balance_c = mock.query_balance(&owner, "ubtsg")?;
        let bidder_balance_c = mock.query_balance(&bidder, "ubtsg")?;
        assert_eq!(owner_balance_c.u128(), owner_balance_b.u128() + BID_AMOUNT);
        assert_eq!(bidder_balance_b, bidder_balance_c);
        Ok(())
    }

    #[test]
    fn test_cancel_cooldown_period() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        mock.wait_seconds(200)?;
        let owner = mock.sender.clone();
        mock.add_balance(&owner, vec![coin(10000000000u128, "ubtsg")])?;
        let bidder = mock.addr_make("bidder");
        mock.add_balance(&owner, vec![coin(10000000000u128, "uthiol")])?;
        let account = "account";
        suite.mint_and_list(mock.clone(), account, &owner)?;
        suite.bid_w_funds(mock.clone(), account, bidder.clone(), BID_AMOUNT)?;
        let owner_balance_a = mock.query_balance(&owner, "ubtsg")?;
        let bidder_balance_a = mock.query_balance(&bidder, "ubtsg")?;
        suite.market.accept_bid(bidder.clone(), account.into())?;
        // no funds are transfered before cooldown is over
        let owner_balance_b = mock.query_balance(&owner, "ubtsg")?;
        let bidder_balance_b = mock.query_balance(&bidder, "ubtsg")?;
        mock.wait_seconds(10)?;
        assert_eq!(owner_balance_a, owner_balance_b);
        assert_eq!(bidder_balance_a, bidder_balance_b);
        assert_eq!(
            suite.market.cooldown(account.to_string())?,
            Some(PendingBid {
                ask: Ask {
                    token_id: account.to_string(),
                    id: 1,
                    seller: owner.clone(),
                },
                new_owner: bidder.clone(),
                amount: BID_AMOUNT.into(),
                unlock_time: mock.block_info()?.time.plus_seconds(50)
            })
        );
        mock.wait_seconds(49)?;
        assert_eq!(
            suite
                .market
                .call_as(&bidder)
                .cancel_cooldown(account.to_string())
                .unwrap_err()
                .root()
                .to_string(),
            ContractError::Unauthorized {}.to_string()
        );
        assert_eq!(
            suite
                .market
                .execute(
                    &ExecuteMsg::CancelCooldown {
                        token_id: account.to_string()
                    },
                    &vec![]
                )
                .unwrap_err()
                .root()
                .to_string(),
            "No funds sent"
        );
        assert_eq!(
            suite
                .market
                .execute(
                    &ExecuteMsg::CancelCooldown {
                        token_id: account.to_string()
                    },
                    &vec![coin(499_000_000, "ubtsg")]
                )
                .unwrap_err()
                .root()
                .to_string(),
            ContractError::IncorrectPayment {
                got: 499_000_000u128,
                expected: 500_000_000u128
            }
            .to_string()
        );
        assert_eq!(
            suite
                .market
                .execute(
                    &ExecuteMsg::CancelCooldown {
                        token_id: account.to_string()
                    },
                    &vec![coin(499_000_000, "uthiol")]
                )
                .unwrap_err()
                .root()
                .to_string(),
            "Must send reserve token 'ubtsg'"
        );
        suite.market.execute(
            &ExecuteMsg::CancelCooldown {
                token_id: account.to_string(),
            },
            &vec![coin(500_000_000, "ubtsg")],
        )?;
        let dd_balance = mock.query_balance(&Addr::unchecked(DEPLOYMENT_DAO), "ubtsg")?;
        let bidder_balance_c = mock.query_balance(&bidder, "ubtsg")?;
        assert_eq!(bidder_balance_c.u128(), BID_AMOUNT + 250_000_000u128);
        assert_eq!(dd_balance.u128(), 250_000_000u128);
        Ok(())
        // assert_eq!(suite.market.cooldown(account.to_string())?, None,);

        // assert_eq!(owner_balance_b, bidder_balance_c);
        // assert_ne!(bidder_balance_b, owner_balance_c);
    }

    #[test]
    fn test_mint_with_delegation_tiers() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        mock.wait_seconds(200)?;
        let user = mock.addr_make("user");
        mock.add_balance(&user, vec![coin(10000000000u128, "ubtsg")])?;

        let id_3 = "rep";
        let id_4 = "repe";
        let id_5 = "repea";
        let id_6 = "repeat";
        let base_delegation = Uint128::new(2100_000_000); // assuming this is the base delegation amount

        // Test with account length 3
        let user3 = mock.addr_make("user3");
        mock.add_balance(&user3, vec![coin(10500000000u128, "ubtsg")])?;
        suite.delegate_to_val(
            mock.clone(),
            user3.clone(),
            (base_delegation * Uint128::new(5u128)).into(),
        )?;
        suite.mint_and_list(mock.clone(), id_3, &user3)?;

        // Test with account length 4
        let user4 = mock.addr_make("user4abcd");
        mock.add_balance(&user4, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(
            mock.clone(),
            user4.clone(),
            (base_delegation * Uint128::new(3u128)).into(),
        )?;
        suite.mint_and_list(mock.clone(), id_4, &user4)?;

        // Test with account length 5 or more
        let user5 = mock.addr_make("user5abcde");
        mock.add_balance(&user5, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), user5.clone(), base_delegation.u128())?;
        suite.mint_and_list(mock.clone(), id_5, &user5)?;

        // Test with insufficient delegation for account length 3
        let user3_insufficient = mock.addr_make("user3_insufficient");
        mock.add_balance(&user3_insufficient, vec![coin(10000440u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), user3_insufficient.clone(), 10000440u128)?;
        assert_eq!(
            suite
                .mint_and_list(mock.clone(), id_6, &user3_insufficient)
                .unwrap_err()
                .source()
                .unwrap()
                .to_string(),
            ContractError::IncorrectDelegation {
                got: 10000440u128,
                expected: base_delegation.u128()
            }
            .to_string()
        );

        Ok(())
    }
}
mod admin {
    use super::*;

    #[test]
    fn test_update_admin() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin2 = mock.addr_make("admin2");
        // non-admin tries to set admin to None
        suite
            .minter
            .call_as(&admin2)
            .update_ownership(cw_ownable::Action::RenounceOwnership)
            .unwrap_err();
        // admin updates admin
        suite
            .minter
            .update_ownership(cw_ownable::Action::TransferOwnership {
                new_owner: admin2.to_string(),
                expiry: None,
            })?;
        // new admin updates to have no admin
        suite
            .minter
            .call_as(&admin2)
            .update_ownership(cw_ownable::Action::AcceptOwnership)?;
        suite
            .minter
            .call_as(&admin2)
            .update_ownership(cw_ownable::Action::RenounceOwnership)?;
        // cannot update without admin
        suite
            .minter
            .update_ownership(cw_ownable::Action::TransferOwnership {
                new_owner: admin2.to_string(),
                expiry: None,
            })
            .unwrap_err();
        Ok(())
    }
}
mod query {
    use btsg_account::market::{Bid, BidOffset};
    use cosmwasm_std::coin;

    use super::*;

    #[test]
    fn test_query_ask() -> anyhow::Result<()> {
        let token_id = "bandura";
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin2 = mock.addr_make("admin2");
        // delegate
        mock.add_balance(&admin2, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), admin2.clone(), 10000000000u128)?;

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin2)?;

        assert_eq!(
            suite.market.ask(token_id.into())?.unwrap().token_id,
            token_id
        );
        Ok(())
    }
    #[test]
    fn test_query_asks() -> anyhow::Result<()> {
        let token_id = "bandura";
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let admin2 = mock.addr_make("admin2");

        // delegate
        mock.add_balance(&admin2, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), admin2.clone(), 10000000000u128)?;

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin)?;
        suite.mint_and_list(mock.clone(), "hack", &admin2)?;

        assert_eq!(suite.market.asks(None, None)?[0].id, 1);

        Ok(())
    }
    #[test]
    fn test_query_asks_by_seller() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let admin2 = mock.addr_make("admin2");
        let token_id = "bandura";

        // delegate
        mock.add_balance(&admin2, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), admin2.clone(), 10000000000u128)?;

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin)?;
        suite.mint_and_list(mock.clone(), "hack", &admin2)?;

        assert_eq!(
            suite
                .market
                .asks_by_seller(admin.to_string(), None, None)?
                .len(),
            1
        );
        Ok(())
    }
    #[test]
    fn test_query_ask_count() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let admin2 = mock.addr_make("admin2");
        let token_id = "bandura";

        // delegate
        mock.add_balance(&admin2, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), admin2.clone(), 10000000000u128)?;

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin)?;
        suite.mint_and_list(mock.clone(), "hack", &admin2)?;

        assert_eq!(suite.market.ask_count()?, 2);
        Ok(())
    }
    #[test]
    fn test_query_top_bids() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let bidder1 = mock.addr_make("bidder1");
        let bidder2 = mock.addr_make("bidder2");
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin)?;

        suite.bid_w_funds(mock.clone(), token_id, bidder1.clone(), BID_AMOUNT)?;
        suite.bid_w_funds(mock.clone(), token_id, bidder2.clone(), BID_AMOUNT * 5)?;

        let res = suite.market.bids_for_seller(admin.clone(), None, None)?;
        assert_eq!(res.len(), 2);
        assert_eq!(res[1].amount.u128(), BID_AMOUNT);

        // test pagination
        let filter = BidOffset {
            price: Uint128::from(BID_AMOUNT),
            token_id: token_id.into(),
            bidder: bidder1.clone(),
        };
        let res: Vec<Bid> =
            suite
                .market
                .bids_for_seller(admin.clone(), None, Some(filter.clone()))?;

        // should be length 0 because there are no token_ids besides NAME.to_string()
        assert_eq!(res.len(), 0);

        // added to get around rate limiting
        mock.wait_seconds(60)?;

        // test pagination with multiple accounts and bids
        let account: &str = "jump";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), account, &admin)?;

        suite.bid_w_funds(mock.clone(), account, bidder1, BID_AMOUNT * 3)?;
        suite.bid_w_funds(mock.clone(), account, bidder2, BID_AMOUNT * 2)?;
        let res = suite.market.bids_for_seller(admin, None, Some(filter))?;
        // should be length 2 because there is token_id "jump" with 2 bids
        assert_eq!(res.len(), 2);

        Ok(())
    }
    #[test]
    fn test_query_bids_by_seller() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let bidder1 = mock.addr_make("bidder1");
        let bidder2 = mock.addr_make("bidder2");
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin)?;

        suite.bid_w_funds(mock.clone(), token_id, bidder1.clone(), BID_AMOUNT)?;
        suite.bid_w_funds(mock.clone(), token_id, bidder2.clone(), BID_AMOUNT * 5)?;

        let res = suite.market.bids_for_seller(admin.clone(), None, None)?;
        assert_eq!(res.len(), 2);
        assert_eq!(res[1].amount.u128(), BID_AMOUNT);

        // test pagination
        let res = suite.market.bids_for_seller(
            admin.clone(),
            None,
            Some(BidOffset::new(
                Uint128::from(BID_AMOUNT),
                token_id.to_string(),
                bidder1.clone(),
            )),
        )?;
        assert_eq!(res.len(), 0);

        // added to get around rate limiting
        mock.wait_seconds(60)?;

        let account = "jump";
        suite.mint_and_list(mock.clone(), account, &admin)?;
        suite.bid_w_funds(mock.clone(), account, bidder1.clone(), BID_AMOUNT * 3)?;
        suite.bid_w_funds(mock.clone(), account, bidder2, BID_AMOUNT * 2)?;
        // should be length 2 because there is token_id "jump" with 2 bids
        let res = suite.market.bids_for_seller(
            admin.clone(),
            None,
            Some(BidOffset::new(
                Uint128::from(BID_AMOUNT),
                token_id.to_string(),
                bidder1.clone(),
            )),
        )?;
        assert_eq!(res.len(), 2);

        Ok(())
    }
    #[test]
    fn test_query_highest_bid() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let bidder1 = mock.addr_make("bidder1");
        let bidder2 = mock.addr_make("bidder2");
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin)?;

        suite.bid_w_funds(mock.clone(), token_id, bidder1.clone(), BID_AMOUNT)?;
        suite.bid_w_funds(mock.clone(), token_id, bidder2.clone(), BID_AMOUNT * 5)?;

        assert_eq!(
            suite
                .market
                .highest_bid(token_id.to_string())?
                .unwrap()
                .amount
                .u128(),
            BID_AMOUNT * 5
        );

        Ok(())
    }
    #[test]
    fn test_query_account() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin)?;

        // fails with "user" string, has to be a bech32 address
        suite.account.account(token_id).unwrap_err();

        suite.mint_and_list(mock.clone(), "yoyo", &admin)?;

        suite
            .account
            .associate_address("yoyo", Some(admin.to_string()))?;

        assert_eq!(suite.account.account(admin)?, "yoyo".to_string());

        Ok(())
    }
    // #[test]
    // fn test_query_trading_start_time() -> anyhow::Result<()> {
    //     let mock = MockBech32::new("bitsong");
    //     let mut suite = BtsgAccountSuite::new(mock.clone());
    //     suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

    //     Ok(())
    // }
}
mod collection {
    use btsg_account::TextRecord;
    use cosmwasm_std::coin;

    use super::*;
    #[test]
    fn test_verify_twitter() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        mock.wait_seconds(200)?;

        let admin_user = mock.sender.clone();
        let verifier = mock.addr_make("verifier");
        let token_id = "bandura";

        suite.mint_and_list(mock, token_id, &admin_user)?;

        let account = "twitter";
        let value = "loaf0bred";

        suite
            .account
            .add_text_record(token_id, TextRecord::new(account, value))?;

        // query text record to see if verified is not set
        let res = suite.account.nft_info(token_id)?;
        assert_eq!(res.extension.records[0].account, account.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        suite
            .account
            .verify_text_record(token_id, account, true)
            .unwrap_err();

        suite
            .account
            .call_as(&verifier)
            .verify_text_record(token_id, account, true)?;

        // query text record to see if verified is set
        let res = suite.account.nft_info(token_id)?;

        assert_eq!(res.extension.records[0].account, account.to_string());
        assert_eq!(res.extension.records[0].verified, Some(true));

        Ok(())
    }
    #[test]
    fn test_verify_false() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        mock.wait_seconds(200)?;

        let admin_user = mock.sender.clone();
        let verifier = mock.addr_make("verifier");
        let token_id = "bandura";

        suite.mint_and_list(mock, token_id, &admin_user)?;

        let account = "twitter";
        let value = "loaf0bred";

        suite
            .account
            .add_text_record(token_id, TextRecord::new(account, value))?;

        suite
            .account
            .call_as(&verifier)
            .verify_text_record(token_id, account, false)?;

        // query text record to see if verified is not set
        let res = suite.account.nft_info(token_id)?;

        assert_eq!(res.extension.records[0].account, account.to_string());
        assert_eq!(res.extension.records[0].verified, Some(false));

        Ok(())
    }
    #[test]
    fn test_verified_text_record() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        mock.wait_seconds(200)?;

        let admin_user = mock.sender.clone();
        let token_id = "bandura";

        suite.mint_and_list(mock, token_id, &admin_user)?;

        let account = "twitter";
        let value = "loaf0bred";

        suite
            .account
            .add_text_record(token_id, TextRecord::new(account, value))?;

        // query text record to see if verified is not set
        let res = suite.account.nft_info(token_id)?;
        assert_eq!(res.extension.records[0].account, account.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // attempt update text record w verified value
        suite.account.update_text_record(
            token_id,
            TextRecord {
                account: token_id.into(),
                value: "some new value".to_string(),
                verified: Some(true),
            },
        )?;

        // query text record to see if verified is set
        let res = suite.account.nft_info(token_id)?;

        assert_eq!(res.extension.records[0].account, account.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // query image nft
        assert_eq!(suite.account.image_nft(token_id)?, None);
        Ok(())
    }
    #[test]
    fn test_transfer_nft() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin_user)?;

        suite
            .account
            .transfer_nft(mock.addr_make("new-addr"), token_id)?;

        Ok(())
    }
    #[test]
    fn test_send_nft() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin_user)?;

        suite
            .account
            .send_nft(mock.addr_make("new-addr"), to_json_binary("ini")?, token_id)?;
        Ok(())
    }
    #[test]
    fn test_transfer_nft_and_bid() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let bidder1 = mock.addr_make("bidder1");
        let market = suite.market.address()?;

        let user1 = mock.addr_make("user1");
        let admin_user = mock.sender.clone();
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin_user)?;

        suite.account.transfer_nft(user1.clone(), token_id)?;

        suite.bid_w_funds(mock.clone(), token_id, bidder1.clone(), BID_AMOUNT * 3)?;

        // user2 must approve the marketplace to transfer their account
        suite
            .account
            .call_as(&user1)
            .approve(market, token_id, None)?;
        // accept bid
        suite
            .market
            .call_as(&user1)
            .accept_bid(bidder1, token_id.into())?;

        Ok(())
    }
    #[test]
    fn test_transfer_nft_with_reverse_map() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let user = mock.addr_make("user");
        let user2 = mock.addr_make("user2");
        let token_id = "bandura";

        // delegate
        mock.add_balance(&user, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), user.clone(), 10000000000u128)?;

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &user)?;

        suite
            .account
            .call_as(&user)
            .associate_address(token_id, Some(user.to_string()))?;

        assert_eq!(suite.account.account(user.clone())?, token_id);

        suite
            .account
            .call_as(&user)
            .transfer_nft(user2.clone(), token_id)?;

        suite.account.account(user).unwrap_err();
        suite.account.account(user2).unwrap_err();

        Ok(())
    }
    // #[test]
    // fn test_burn_nft() -> anyhow::Result<()> {
    //     Ok(())
    // }
    // #[test]
    // fn test_burn_with_existing_bids() -> anyhow::Result<()> {
    //     Ok(())
    // }
    // #[test]
    // fn test_burn_nft_with_reverse_map() -> anyhow::Result<()> {
    //     Ok(())
    // }
    #[test]
    fn test_sudo_update() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let max_record_count = suite.account.params()?.max_record_count;
        let max_rev_key_count = suite.account.params()?.max_reverse_map_key_limit;

        // run sudo msg
        mock.app.borrow_mut().sudo(SudoMsg::Wasm(WasmSudo {
            contract_addr: suite.account.address()?,
            message: to_json_binary(&bs721_account::msg::SudoMsg::UpdateParams {
                max_record_count: max_record_count + 1,
                max_rev_map_count: max_rev_key_count + 4,
            })?,
        }))?;

        assert_eq!(
            suite.account.params()?.max_record_count,
            max_record_count + 1
        );
        assert_eq!(
            suite.account.params()?.max_reverse_map_key_limit,
            max_record_count + 4
        );

        Ok(())
    }
}
mod public_start_time {

    use btsg_account::minter::Config;
    use cosmwasm_std::coin;

    use super::*;

    #[test]
    fn test_mint_before_start() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();
        let token_id = "bandura";
        let user4 = mock.addr_make("user4");

        // delegate
        mock.add_balance(&user4, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), user4.clone(), 10000000000u128)?;

        suite
            .mint_and_list(mock.clone(), token_id, &admin_user)
            .unwrap_err();
        suite
            .mint_and_list(mock.clone(), token_id, &user4)
            .unwrap_err();
        Ok(())
    }

    #[test]
    fn test_update_start_time() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let res = suite.minter.config()?;
        assert_eq!(
            res.public_mint_start_time,
            mock.block_info()?.time.plus_seconds(200)
        );

        suite.minter.update_config(Config {
            public_mint_start_time: mock.block_info()?.time.plus_seconds(2),
        })?;

        let res = suite.minter.config()?;
        assert_eq!(
            res.public_mint_start_time,
            mock.block_info()?.time.plus_seconds(2)
        );

        Ok(())
    }
}

mod associate_address {

    use bs721_account::msg::InstantiateMsg;
    use cosmwasm_std::coin;

    use super::*;

    #[test]
    fn test_abstract_account_workflow() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();
        let cw721_id = suite.account.code_id()?;
        let token_id = "bandura";

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &admin_user)?;

        // set nft as ownership
        suite.test_owner.instantiate(
            &TestOwnershipInitMsg {
                ownership: abstract_std::objects::gov_type::GovernanceDetails::NFT {
                    collection_addr: suite.account.addr_str()?,
                    token_id: token_id.to_string(),
                },
            },
            None,
            &[],
        )?;

        // associate account to abstract account
        suite
            .account
            .update_abs_acc_support(token_id, Some(suite.test_owner.addr_str()?))?;

        // query the associated address and ensure its the same as the abstract account
        assert_eq!(
            suite.account.associated_address(token_id)?,
            suite.test_owner.address()?
        );

        Ok(())
    }
    #[test]
    fn test_transfer_to_eoa() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();

        let cw721_id = suite.account.code_id()?;
        let token_id = "bandura";

        let nft_addr = mock
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                Some(&admin_user),
                &[],
            )?
            .instantiated_contract_address()?;

        mock.wait_seconds(200)?;
        // mint and transfer to collection
        suite.mint_and_list(mock.clone(), token_id, &admin_user)?;
        suite.account.transfer_nft(nft_addr.clone(), token_id)?;
        assert_eq!(
            suite.account.owner_of(token_id, None)?.owner,
            nft_addr.to_string()
        );

        Ok(())
    }
    #[test]
    fn test_associate_with_a_contract_with_no_admin() -> anyhow::Result<()> {
        // For the purposes of this test, a collection contract with no admin needs to be instantiated (contract_with_no_admin)
        // This contract needs to have a creator that is itself a contract and this creator contract should have an admin (USER).
        // The admin (USER) of the creator contract will mint a account and associate the account with the collection contract that doesn't have an admin successfully.
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();

        let cw721_id = suite.account.code_id()?;

        let token_id = "bandura";
        // Instantiating the creator contract with an admin (USER)
        let creator_addr = mock
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                Some(&admin_user),
                &[],
            )?
            .instantiated_contract_address()?;

        // The creator contract instantiates the collection contract with no admin
        let collection_with_no_admin_addr = mock
            .call_as(&creator_addr)
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                None,
                &[],
            )?
            .instantiated_contract_address()?;

        mock.wait_seconds(200)?;
        // USER4 mints a account
        suite.mint_and_list(mock.clone(), token_id, &admin_user)?;

        // USER4 tries to associate the account with the collection contract that doesn't have an admin
        suite
            .account
            .call_as(&admin_user)
            .associate_address(token_id, Some(collection_with_no_admin_addr.to_string()))?;

        mock.wait_seconds(200)?;
        Ok(())
    }
    #[test]
    fn test_associate_with_a_contract_with_no_admin_fail() -> anyhow::Result<()> {
        // For the purposes of this test, a collection contract with no admin needs to be instantiated (contract_with_no_admin)
        // This contract needs to have a creator that is itself a contract and this creator contract should have an admin (USER).
        // An address other than the admin (USER) of the creator contract will mint a account, try to associate the account with the collection contract that doesn't have an admin and fail.
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.addr_make("admin-user");
        let user4 = mock.addr_make("user4");

        // delegate
        mock.add_balance(&user4, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), user4.clone(), 10000000000u128)?;

        let cw721_id = suite.account.code_id()?;

        let token_id = "bandura";
        // Instantiating the creator contract with an admin (USER)
        let creator_addr = mock
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                Some(&admin_user),
                &[],
            )?
            .instantiated_contract_address()?;

        // The creator contract instantiates the collection contract with no admin
        let collection_with_no_admin_addr = mock
            .call_as(&creator_addr)
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                None,
                &[],
            )?
            .instantiated_contract_address()?;

        mock.wait_seconds(200)?;
        // USER4 mints a account
        suite.mint_and_list(mock.clone(), token_id, &user4)?;

        // USER4 tries to associate the account with the collection contract that doesn't have an admin
        let err = suite
            .account
            .call_as(&user4)
            .associate_address(token_id, Some(collection_with_no_admin_addr.to_string()))
            .unwrap_err();

        assert_eq!(
            err.root().to_string(),
            bs721_account::ContractError::UnauthorizedCreatorOrAdmin {}.to_string()
        );
        Ok(())
    }
    #[test]
    fn test_associate_with_a_contract_with_an_admin_fail() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.addr_make("admin-user");
        let user4 = mock.addr_make("user4");

        // delegate
        mock.add_balance(&user4, vec![coin(10000000000u128, "ubtsg")])?;
        suite.delegate_to_val(mock.clone(), user4.clone(), 10000000000u128)?;

        let cw721_id = suite.account.code_id()?;

        let token_id = "bandura";
        // Instantiating the creator contract with an admin (USER)
        let contract = mock
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                Some(&admin_user),
                &[],
            )?
            .instantiated_contract_address()?;

        mock.wait_seconds(200)?;
        suite.mint_and_list(mock.clone(), token_id, &user4)?;

        let err = suite
            .account
            .call_as(&user4)
            .associate_address(token_id, Some(contract.to_string()))
            .unwrap_err();

        assert_eq!(
            err.root().to_string(),
            bs721_account::ContractError::UnauthorizedCreatorOrAdmin {}.to_string()
        );
        Ok(())
    }
}
