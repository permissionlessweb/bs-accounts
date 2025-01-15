pub mod account;
pub mod market;
pub mod minter;

use bs721_account::msg::{Bs721AccountsQueryMsgFns as _, ExecuteMsgFns as _};
use bs721_account_marketplace::msgs::{
    ExecuteMsgFns as _, InstantiateMsg as AccountMarketInitMsg, QueryMsgFns,
};
use bs721_account_minter::msg::InstantiateMsg as AccountMinterInitMsg;
use cosmwasm_std::{coins, Uint128};
use cw_orch::{anyhow, prelude::*};

const BASE_PRICE: u128 = 100_000_000;

use serde::{Deserialize, Serialize};

use crate::{
    networks::{GAS_TO_DEPLOY, SUPPORTED_CHAINS},
    BtsgAccountSuite,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentStatus {
    pub chain_ids: Vec<String>,
    pub success: bool,
}

/// MockBech32 implementation for the Bitsong Account Suite.
impl BtsgAccountSuite<MockBech32> {
    /// Creates intitial suite for testing
    pub fn default_setup(
        &mut self,
        mock: MockBech32,
        creator: Option<Addr>,
        admin: Option<Addr>,
    ) -> anyhow::Result<()> {
        let admin2 = mock.addr_make("admin2");
        // a. uploads all contracts
        self.upload()?;
        // b. instantiates marketplace
        self.market.instantiate(
            &AccountMarketInitMsg {
                trading_fee_bps: 0u64,
                min_price: 100u128.into(),
                ask_interval: 30u64,
                valid_bid_query_limit: 100u32,
                // operator: mock.sender_addr().to_string(),
            },
            None,
            &[],
        )?;
        // Account Minter
        // On instantitate, bs721-account contract is created by minter contract.
        // We grab this contract addr from response events, and set address in internal test suite state.
        let bs721_account = self
            .minter
            .call_as(&creator.clone().unwrap_or_else(|| admin2.clone()))
            .instantiate(
                &AccountMinterInitMsg {
                    admin: admin.clone().map(|a| a.to_string()),
                    verifier: Some(mock.addr_make("verifier").to_string()),
                    collection_code_id: self.account.code_id()?,
                    marketplace_addr: self.market.addr_str()?,
                    min_account_length: 3u32,
                    max_account_length: 128u32,
                    base_price: BASE_PRICE.into(),
                },
                None,
                &[],
            )?
            .event_attr_value("wasm", "bs721_account_address")?;

        self.account
            .set_default_address(&Addr::unchecked(bs721_account));

        // Provide marketplace with collection and minter contracts.
        self.market
            .setup(self.account.address()?, self.minter.address()?)?;
        // println!("TOKEN:   {:#?}", self.account.addr_str()?);
        // println!("MARKET:  {:#?}", self.market.addr_str()?);
        // println!("MINTER:  {:#?}", self.minter.addr_str()?);
        // println!("SENDER:  {:#?}", mock.sender_addr().to_string());
        // println!("ADMIN2:  {:#?}", admin2.to_string());
        // println!("ADMIN:   {:#?}", admin);
        // println!("CREATOR: {:#?}", creator);

        Ok(())
    }
    /// mint and list an account token.
    pub fn mint_and_list(
        &mut self,
        mock: MockBech32,
        account: &str,
        user: &Addr,
    ) -> anyhow::Result<()> {
        // set approval for user, for all tokens
        // approve_all is needed because we don't know the token_id before-hand
        let market = self.market.address()?;
        self.account.call_as(user).approve_all(market, None)?;

        let amount: Uint128 = (match account.to_string().as_str().len() {
            0..=2 => BASE_PRICE,
            3 => BASE_PRICE * 100,
            4 => BASE_PRICE * 10,
            _ => BASE_PRICE,
        })
        .into();
        let name_fee = coins(amount.u128(), "ubtsg");
        // give user some funds
        if Uint128::from(BASE_PRICE) > Uint128::from(0u128) {
            mock.add_balance(&user.clone(), name_fee.clone())?;
        };
        // call as user to mint and list the account name, with account fees
        self.minter.call_as(user).execute(
            &bs721_account_minter::msg::ExecuteMsg::MintAndList {
                account: account.to_string(),
            },
            &name_fee,
        )?;
        Ok(())
    }

    pub fn owner_of(&self, id: String) -> anyhow::Result<String> {
        let res = self.account.owner_of(id, None)?;
        Ok(res.owner)
    }

    pub fn bid_w_funds(
        &self,
        mock: MockBech32,
        account: &str,
        bidder: Addr,
        amount: u128,
    ) -> anyhow::Result<()> {
        // give bidder some funds
        let bid_amnt = coins(amount, "ubtsg");
        mock.add_balance(&bidder, bid_amnt.clone())?;

        self.market.call_as(&bidder).execute(
            &bs721_account_marketplace::msgs::ExecuteMsg::SetBid {
                token_id: account.into(),
            },
            &bid_amnt,
        )?;

        // query if bid exists
        let res = self
            .market
            .bid(bidder.to_string(), account.into())?
            .unwrap();
        assert_eq!(res.token_id, account.to_string());
        assert_eq!(res.bidder, bidder);
        assert_eq!(res.amount, Uint128::from(amount));
        Ok(())
    }
}

pub async fn assert_wallet_balance(mut chains: Vec<ChainInfoOwned>) -> Vec<ChainInfoOwned> {
    if chains.is_empty() {
        chains = SUPPORTED_CHAINS.iter().cloned().map(Into::into).collect();
    }
    // check that the wallet has enough gas on all the chains we want to support
    for chain_info in &chains {
        let chain = DaemonAsyncBuilder::new(chain_info.clone())
            .build()
            .await
            .unwrap();

        let gas_denom = chain.state().chain_data.gas_denom.clone();
        let gas_price = chain.state().chain_data.gas_price;
        let fee = (GAS_TO_DEPLOY as f64 * gas_price) as u128;
        let bank = queriers::Bank::new_async(chain.channel());
        let balance = bank
            ._balance(&chain.sender_addr(), Some(gas_denom.clone()))
            .await
            .unwrap()
            .clone()[0]
            .clone();

        log::debug!(
            "Checking balance {} on chain {}, address {}. Expecting {}{}",
            balance.amount,
            chain_info.chain_id,
            chain.sender_addr(),
            fee,
            gas_denom
        );
        if fee > balance.amount.u128() {
            panic!("Not enough funds on chain {} to deploy the contract. Needed: {}{} but only have: {}{}", chain_info.chain_id, fee, gas_denom, balance.amount, gas_denom);
        }
        // check if we have enough funds
    }

    chains
}
