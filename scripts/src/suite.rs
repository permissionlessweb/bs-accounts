// use abstract_interface::Abstract;
use btsg_account::Metadata;

use crate::deploy::{account::*, market::BtsgAccountMarket, minter::BtsgAccountMinter};
pub use bs721_account_marketplace::msgs::ExecuteMsgFns as _;
pub use bs721_account_minter::msg::ExecuteMsgFns as _;
pub use bs721_base::msg::ExecuteMsgFns as _;
pub use bs721_account::msg::ExecuteMsgFns as _;
use btsg_nft_scripts::base::Bs721Base;
use cosmwasm_std::Uint128;

use cw_orch::prelude::*;
pub struct BtsgAccountSuite<Chain>
where
    Chain: cw_orch::prelude::CwEnv,
{
    pub account: BtsgAccountCollection<Chain, Metadata>,
    pub minter: BtsgAccountMinter<Chain>,
    pub market: BtsgAccountMarket<Chain>,
    pub bs721base: Bs721Base<Chain, Empty, Empty>,
    // pub abs: Abstract<Chain>,
}

pub const BLS_PUBKEY: &str = "";

impl<Chain: CwEnv> BtsgAccountSuite<Chain> {
    pub fn new(chain: Chain) -> BtsgAccountSuite<Chain> {
        BtsgAccountSuite::<Chain> {
            account: BtsgAccountCollection::new("bs721_account", chain.clone()),
            minter: BtsgAccountMinter::new("bs721_account_minter", chain.clone()),
            market: BtsgAccountMarket::new("bs721_account_market", chain.clone()),
            bs721base: Bs721Base::new("bs721_base", chain.clone()),
            // abs: Abstract::new(chain.clone()),
        }
    }

    pub fn upload(&self) -> Result<(), CwOrchError> {
        let _acc = self.account.upload()?.uploaded_code_id()?;
        let _minter = self.minter.upload()?.uploaded_code_id()?;
        let _market = self.market.upload()?.uploaded_code_id()?;
        let _bs721base = self.bs721base.upload()?.uploaded_code_id()?;
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

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
        vec![Box::new(&mut self.account), Box::new(&mut self.minter)]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let suite = Self::new(chain.clone());
        Ok(suite)
    }

    fn deploy_on(chain: Chain, data: Self::DeployData) -> Result<Self, Self::Error> {
        // let abs: Abstract<Chain> = Abstract::deploy_on(chain.clone(), ())
        //     .map_err(|e| CwOrchError::from(anyhow::anyhow!(e)))?;
        // ########### Upload ##############
        let mut suite: BtsgAccountSuite<Chain> = BtsgAccountSuite::store_on(chain.clone())?;
        // suite.abs = abs;

        suite.market.instantiate(
            &bs721_account_marketplace::msgs::InstantiateMsg {
                trading_fee_bps: 200,
                min_price: Uint128::from(5000000u64),
                ask_interval: 60,
                valid_bid_query_limit: 30,
            },
            Some(&Addr::unchecked(data.to_string())),
            &[],
        )?;

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
                    base_delegation: 2100_000_000u128.into(),
                    marketplace_addr: suite.market.addr_str()?,
                    mint_start_delay: None,
                },
                Some(&Addr::unchecked(data.clone())),
                &[],
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

        // Provide marketplace with collection and minter contracts.
        suite
            .market
            .setup(suite.account.address()?, suite.minter.address()?)?;

        // instantiate wavs authenticator
        // suite.wavs.instantiate(
        //     &btsg_wavs::msg::InstantiateMsg {
        //         owner: None,
        //         wavs_operator_pubkeys: vec![BLS_PUBKEY.as_bytes().into()], // only one bls key, need to add more
        //     },
        //     None,
        //     &[],
        // )?;

        Ok(suite)
    }
}
