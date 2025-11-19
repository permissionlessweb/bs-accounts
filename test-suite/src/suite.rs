use anyhow::Result as AnyResult;

use cosmwasm_std::{
    coin, coins, Addr, Coin, Decimal, Decimal256, StakingMsg, StdResult, Uint128, Uint256,
};
use cw_multi_test::{
    App, AppBuilder, Contract, ContractWrapper, Executor, Module, StakingInfo, SudoMsg as CwSudoMsg,
};

use crate::constants::*;

pub const BASE_PRICE: u128 = 100_000_000;
pub const BASE_DELEGATION: u128 = 2_100_000_000;

pub struct BtsgTestSuite {
    pub app: App,
    pub admin: Addr,
    pub creator: Addr,
}

impl BtsgTestSuite {
    pub fn new() -> Self {
        let admin = Addr::unchecked(ADMIN);
        let creator = Addr::unchecked(CREATOR);
        let mut app = AppBuilder::new().build(|router, api, storage| {
            router
                .bank
                .init_balance(storage, &creator, coins(42000000000, "ubtsg"))
                .unwrap();
            router
                .bank
                .init_balance(storage, &admin, coins(21000000000, "ubtsg"))
                .unwrap();
        });

        // Store contracts

        Self {
            app,

            admin,
            creator,
        }
    }

    pub fn setup_staking(&mut self) -> StdResult<()> {
        let block_info = self.app.block_info();

        self.app.init_modules(|router, api, storage| {
            router.staking.setup(
                storage,
                StakingInfo {
                    bonded_denom: "ubtsg".into(),
                    unbonding_time: 69u64,
                    apr: Decimal256::from_ratio(69u128, 100u128),
                },
            )?;
            router.staking.add_validator(
                api,
                storage,
                &block_info,
                cosmwasm_std::Validator::create(
                    VALIDATOR_1.to_string(),
                    Decimal::from_ratio(1u128, 2u128),
                    Decimal::one(),
                    Decimal::one(),
                ),
            )
        })?;

        Ok(())
    }

    pub fn delegate_to_validator(&mut self, delegator: &Addr, amount: u128) -> StdResult<()> {
        let block_info = self.app.block_info();

        self.app.init_modules(|router, api, storage| {
            router.staking.execute(
                api,
                storage,
                router,
                &block_info,
                delegator.clone(),
                StakingMsg::Delegate {
                    validator: VALIDATOR_1.into(),
                    amount: coin(amount, "ubtsg"),
                },
            )
        })?;

        Ok(())
    }

    pub fn mint_and_list(&mut self, account: &str, user: &Addr) -> StdResult<()> {
        Ok(())
    }

    pub fn bid_with_funds(&mut self, account: &str, bidder: &Addr, amount: u128) -> StdResult<()> {
        let bid_amount = coins(amount, "ubtsg");

        Ok(())
    }

    pub fn accept_bid(&mut self, bidder: &Addr, token_id: &str) -> StdResult<()> {
        Ok(())
    }

    pub fn default_setup(&mut self) -> StdResult<()> {
        Ok(())
    }
}

// Contract wrappers
