use anyhow::Result as AnyResult;
use btsg_account::Metadata;
use cosmwasm_std::{
    coin, coins, Addr, Coin, Decimal, Decimal256, StakingMsg, StdResult, Uint128, Uint256,
};
use cw_multi_test::{
    App, AppBuilder, Contract, ContractWrapper, Executor, Module, StakingInfo, SudoMsg as CwSudoMsg,
};

use bs721_account::{
    msg::{ExecuteMsg as AccountExecuteMsg, InstantiateMsg as AccountInstantiateMsg},
    QueryMsg,
};
use bs721_account_marketplace::msgs::{
    ExecuteMsg as MarketExecuteMsg, InstantiateMsg as MarketInstantiateMsg,
    QueryMsg as MarketQueryMsg,
};
use bs721_account_minter::{
    commands::get_ascii_cost,
    msg::{
        ExecuteMsg as MinterExecuteMsg, InstantiateMsg as MinterInstantiateMsg,
        QueryMsg as MinterQueryMsg,
    },
};

use crate::constants::*;

pub const BASE_PRICE: u128 = 100_000_000;
pub const BASE_DELEGATION: u128 = 2_100_000_000;

pub struct BtsgTestSuite {
    pub app: App,
    pub account_code_id: u64,
    pub marketplace_code_id: u64,
    pub minter_code_id: u64,

    pub account_addr: Option<Addr>,
    pub marketplace_addr: Option<Addr>,
    pub minter_addr: Option<Addr>,

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
        let account_code_id = app.store_code(account_contract());
        let marketplace_code_id = app.store_code(marketplace_contract());
        let minter_code_id = app.store_code(minter_contract());

        Self {
            app,
            account_code_id,
            marketplace_code_id,
            minter_code_id,
            account_addr: None,
            marketplace_addr: None,
            minter_addr: None,
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

    pub fn instantiate_marketplace(&mut self) -> StdResult<()> {
        let marketplace_addr = self.app.instantiate_contract(
            self.marketplace_code_id,
            self.admin.clone(),
            &MarketInstantiateMsg {
                trading_fee_bps: 0u64,
                min_price: 100u128.into(),
                ask_interval: 30u64,
                valid_bid_query_limit: 100u32,
            },
            &[],
            "marketplace",
            None,
        )?;

        self.marketplace_addr = Some(marketplace_addr);
        Ok(())
    }

    pub fn instantiate_minter(&mut self) -> StdResult<()> {
        let marketplace_addr = self.marketplace_addr.as_ref().unwrap();

        let res = self.app.instantiate_contract(
            self.minter_code_id,
            self.creator.clone(),
            &MinterInstantiateMsg {
                admin: Some(self.admin.to_string()),
                verifier: Some(VERIFIER.to_string()),
                collection_code_id: self.account_code_id,
                marketplace_addr: marketplace_addr.to_string(),
                min_account_length: 3u32,
                max_account_length: 128u32,
                base_price: BASE_PRICE.into(),
                base_delegation: BASE_DELEGATION.into(),
                mint_start_delay: Some(200u64),
            },
            &[],
            "minter",
            None,
        )?;
        self.minter_addr = Some(res);

        // query contract config to grab account addres
        self.account_addr = Some(self.app.wrap().query_wasm_smart(
            self.minter_addr.as_ref().expect("FATAL"),
            &MinterQueryMsg::Collection {},
        )?);

        Ok(())
    }

    pub fn setup_marketplace(&mut self) -> StdResult<()> {
        let account_addr = self.account_addr.as_ref().unwrap();
        let minter_addr = self.minter_addr.as_ref().unwrap();
        let marketplace_addr = self.marketplace_addr.as_ref().unwrap();

        self.app.execute_contract(
            self.admin.clone(),
            marketplace_addr.clone(),
            &MarketExecuteMsg::Setup {
                collection: account_addr.to_string(),
                minter: minter_addr.to_string(),
            },
            &[],
        )?;

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
        let marketplace_addr = self.marketplace_addr.as_ref().unwrap();
        let account_addr = self.account_addr.as_ref().unwrap();

        // Approve marketplace to handle the NFT
        self.app
            .execute_contract::<bs721_account::msg::ExecuteMsg<Metadata>>(
                user.clone(),
                account_addr.clone(),
                &AccountExecuteMsg::ApproveAll {
                    operator: marketplace_addr.to_string(),
                    expires: None,
                },
                &[],
            )?;

        let amount: Uint256 = get_ascii_cost(account.len(), BASE_PRICE.into())?;

        let name_fee = vec![Coin::new(amount, "ubtsg")];

        self.app.execute_contract(
            user.clone(),
            self.minter_addr.as_ref().unwrap().clone(),
            &MinterExecuteMsg::MintAndList {
                account: account.to_string(),
            },
            &name_fee,
        )?;

        Ok(())
    }

    pub fn bid_with_funds(&mut self, account: &str, bidder: &Addr, amount: u128) -> StdResult<()> {
        let bid_amount = coins(amount, "ubtsg");
        let marketplace_addr = self.marketplace_addr.as_ref().unwrap();

        self.app.execute_contract(
            bidder.clone(),
            marketplace_addr.clone(),
            &MarketExecuteMsg::SetBid {
                token_id: account.into(),
            },
            &bid_amount,
        )?;

        Ok(())
    }

    pub fn accept_bid(&mut self, bidder: &Addr, token_id: &str) -> StdResult<()> {
        let marketplace_addr = self.marketplace_addr.as_ref().unwrap();

        self.app.execute_contract(
            self.creator.clone(),
            marketplace_addr.clone(),
            &MarketExecuteMsg::AcceptBid {
                bidder: bidder.to_string(),
                token_id: token_id.to_string(),
            },
            &[],
        )?;

        Ok(())
    }

    pub fn get_owner(&self, token_id: &str) -> StdResult<Addr> {
        let account_addr = self.account_addr.as_ref().unwrap();

        let res: bs721::OwnerOfResponse = self.app.wrap().query_wasm_smart(
            account_addr,
            &QueryMsg::OwnerOf {
                token_id: token_id.to_string(),
                include_expired: None,
            },
        )?;

        Ok(Addr::unchecked(res.owner))
    }

    pub fn default_setup(&mut self) -> StdResult<()> {
        let creator = self.creator.clone();
        self.setup_staking()?;
        self.instantiate_marketplace()?;
        self.instantiate_minter()?;
        self.setup_marketplace()?;
        self.delegate_to_validator(&creator, 10500000000)?;

        // Wait for mint start delay
        self.app
            .update_block(|block| block.time = block.time.plus_seconds(200));

        Ok(())
    }
}

// Contract wrappers
pub fn account_contract() -> Box<dyn Contract<cosmwasm_std::Empty>> {
    let contract = ContractWrapper::new(
        bs721_account::entry::execute,
        bs721_account::entry::instantiate,
        bs721_account::entry::query,
    )
    .with_sudo(bs721_account::entry::sudo);

    Box::new(contract)
}

pub fn marketplace_contract() -> Box<dyn Contract<cosmwasm_std::Empty>> {
    let contract = ContractWrapper::new(
        bs721_account_marketplace::contract::execute,
        bs721_account_marketplace::contract::instantiate,
        bs721_account_marketplace::contract::query,
    )
    .with_sudo(bs721_account_marketplace::contract::sudo);

    Box::new(contract)
}

pub fn minter_contract() -> Box<dyn Contract<cosmwasm_std::Empty>> {
    let contract = ContractWrapper::new(
        bs721_account_minter::contract::execute,
        bs721_account_minter::contract::instantiate,
        bs721_account_minter::contract::query,
    )
    .with_reply(bs721_account_minter::contract::reply)
    .with_sudo(bs721_account_minter::contract::sudo);

    Box::new(contract)
}
