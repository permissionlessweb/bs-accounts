use std::path::PathBuf;

// use abstract_interface::Abstract;
use crate::BtsgAccountMarketExecuteFns;
use anyhow::anyhow;
use bs721_account::interface::BtsgAccountCollection;
use bs721_account_marketplace::interface::BtsgAccountMarket;
use bs721_account_minter::interface::BtsgAccountMinter;
use btsg_account::{Metadata, CURRENT_BASE_DELEGATION, CURRENT_BASE_PRICE};
use cosmwasm_std::{coin, Uint128};

use cw_orch::prelude::*;
pub struct BtsgAccountSuite<Chain>
where
    Chain: cw_orch::prelude::CwEnv,
{
    pub account: BtsgAccountCollection<Chain, Metadata>,
    pub minter: BtsgAccountMinter<Chain>,
    pub market: BtsgAccountMarket<Chain>,
    // pub abs: Abstract<Chain>,
}

pub const BLS_PUBKEY: &str = "";

impl<Chain: CwEnv> BtsgAccountSuite<Chain> {
    pub fn new(chain: Chain) -> BtsgAccountSuite<Chain> {
        BtsgAccountSuite::<Chain> {
            account: BtsgAccountCollection::new("bs721_account", chain.clone()),
            minter: BtsgAccountMinter::new("bs721_account_minter", chain.clone()),
            market: BtsgAccountMarket::new("bs721_account_marketplace", chain.clone()),
        }
    }

    pub fn upload(&self) -> Result<(), CwOrchError> {
        let acc_code_id = self.account.upload()?.uploaded_code_id()?;
        let minter_code_id = self.minter.upload()?.uploaded_code_id()?;
        let market_code_id = self.market.upload()?.uploaded_code_id()?;

        println!("Account code ID: {}", acc_code_id);
        println!("Minter code ID: {}", minter_code_id);
        println!("Market code ID: {}", market_code_id);

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

        // Read and parse state.json
        let crate_path = env!("CARGO_MANIFEST_DIR");
        let file_path = PathBuf::from(crate_path)
            // State file of your deployment
            .join("state.json")
            .display()
            .to_string();
        let file = std::fs::File::open(&file_path)
            .map_err(|e| anyhow!(format!("Failed to open {}: {}", file_path, e)))?;
        let state: serde_json::Value = serde_json::from_reader(file)
            .map_err(|e| anyhow!(format!("Failed to parse {}: {}", file_path, e)))?;

        // parse json to extract the code-id & contracts
        let chain_id = chain.env_info().chain_id;
        let chain_data = state
            .get(&chain_id)
            .ok_or_else(|| anyhow!(format!("No data found for chain ID: {}", chain_id)))?;

        let code_ids = chain_data
            .get("code_ids")
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("Missing or invalid 'code_ids' in state.json".to_string()))?;

        let contracts = chain_data
            .get("default")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                anyhow!("Missing or invalid 'default' contracts in state.json".to_string())
            })?;

        // Define a local macro that works on any suite object with .id(), .set_code_id(), .set_address()
        macro_rules! set_from_state {
            ($obj:expr) => {{
                let key = $obj.id();
                let code_id = code_ids
                    .get(&key)
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow!(format!("Missing code ID for {}", key)))?;

                let address = contracts
                    .get(&key)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!(format!("Missing address for {}", key)))?;

                $obj.set_code_id(code_id);
                $obj.set_address(&Addr::unchecked(address.to_string()));
            }};
        }

        set_from_state!(suite.account);
        set_from_state!(suite.market);
        set_from_state!(suite.minter);

        Ok(suite)
    }

    fn deploy_on(chain: Chain, data: Self::DeployData) -> Result<Self, Self::Error> {
        // ########### Upload ##############
        let mut suite: BtsgAccountSuite<Chain> = BtsgAccountSuite::store_on(chain.clone())?;
        // suite.abs = abs;

        suite.market.instantiate(
            &btsg_account::market::MarketplaceInstantiateMsg {
                trading_fee_bps: 200,
                min_price: Uint128::from(55_500_000u64),
                ask_interval: 60,
                valid_bid_query_limit: 30,
                cooldown_timeframe: 60 * 60 * 24 * 14 as u64, // 14 days
                cooldown_cancel_fee: coin(500_000_000u128, "ubtsg"),
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
                    base_price: CURRENT_BASE_PRICE.into(),
                    base_delegation: CURRENT_BASE_DELEGATION.into(),
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
        let account = &Addr::unchecked(bs721_account);
        suite.account.set_default_address(&account);
        suite.account.set_address(&account);

        // Provide marketplace with collection and minter contracts.
        suite
            .market
            .setup(suite.account.address()?, suite.minter.address()?)?;

        Ok(suite)
    }
}
