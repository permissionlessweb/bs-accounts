use btsg_account::Metadata;

use cosmwasm_std::Uint128;
use cw_orch::prelude::*;

use crate::deploy::{account::*, market::BitsongAccountMarket, minter::BitsongAccountMinter};
use bs721_account_marketplace::msgs::ExecuteMsgFns as _;
pub struct BtsgAccountSuite<Chain> {
    pub account: BitsongAccountCollection<Chain, Metadata>,
    pub minter: BitsongAccountMinter<Chain>,
    pub market: BitsongAccountMarket<Chain>,
}

impl<Chain: CwEnv> BtsgAccountSuite<Chain> {
    pub fn new(chain: Chain) -> BtsgAccountSuite<Chain> {
        BtsgAccountSuite::<Chain> {
            account: BitsongAccountCollection::new("bs721_account", chain.clone()),
            minter: BitsongAccountMinter::new("bs721_account_minter", chain.clone()),
            market: BitsongAccountMarket::new("bs721_account_market", chain.clone()),
        }
    }

    pub fn upload(&self) -> Result<(), CwOrchError> {
        let acc = self.account.upload()?.uploaded_code_id()?;
        let minter = self.minter.upload()?.uploaded_code_id()?;
        let market = self.market.upload()?.uploaded_code_id()?;

        println!("account collection code-id: {}", acc);
        println!("account minter code-id: {}", minter);
        println!("account minter code-id: {}", market);
        Ok(())
    }
}

// Bitsong Accounts `Deploy` Suite
impl<Chain: CwEnv> cw_orch::contract::Deploy<Chain> for BtsgAccountSuite<Chain> {
    // We don't have a custom error type
    type Error = CwOrchError;
    type DeployData = Addr;

    fn store_on(chain: Chain) -> Result<Self, Self::Error> {
        let suite = BtsgAccountSuite::new(chain.clone());
        suite.upload()?;
        Ok(suite)
    }

    fn deployed_state_file_path() -> Option<String> {
        None
    }

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
        vec![Box::new(&mut self.account), Box::new(&mut self.minter)]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let suite = Self::new(chain.clone());
        Ok(suite)
    }

    fn deploy_on(chain: Chain, data: Self::DeployData) -> Result<Self, Self::Error> {
        // ########### Upload ##############
        let mut suite: BtsgAccountSuite<Chain> = BtsgAccountSuite::store_on(chain.clone())?;

        let bs721_account = suite
            .minter
            .instantiate(
                &bs721_account_minter::msg::InstantiateMsg {
                    admin: Some(data.to_string()),
                    verifier: None,
                    collection_code_id: suite.account.code_id()?,
                    min_account_length: 3u32,
                    max_account_length: 128u32,
                    base_price: 10u128.into(),
                    marketplace_addr: "todo!()".into(),
                },
                Some(&Addr::unchecked(data.clone())),
                None,
            )?
            .event_attr_value("wasm", "bs721_account_address")?;

        println!(
            "bs721-account minter contract: {}",
            suite.minter.addr_str()?
        );
        println!("bs721-account collection contract: {}", bs721_account);

        suite
            .account
            .set_default_address(&Addr::unchecked(bs721_account));

        suite.market.instantiate(
            &bs721_account_marketplace::msgs::InstantiateMsg {
                trading_fee_bps: 200,
                min_price: Uint128::from(5000000u64),
                ask_interval: 60,
                valid_bid_query_limit: 30,
            },
            Some(&Addr::unchecked(data)),
            None,
        )?;

        // Provide marketplace with collection and minter contracts.
        // suite
        //     .market
        //     .setup(suite.account.address()?, suite.minter.address()?)?;

        suite.market.execute(
            &bs721_account_marketplace::msgs::ExecuteMsg::Setup {
                minter: suite.minter.address()?.into(),
                collection: suite.account.address()?.into(),
            },
            None,
        )?;
        Ok(suite)
    }
}
