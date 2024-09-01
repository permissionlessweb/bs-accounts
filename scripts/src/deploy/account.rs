use bs721_account_marketplace::msg::ExecuteMsgFns as _;
use btsg_account::Metadata;
use btsg_cw_orch::*;
use cosmwasm_std::Decimal;
use cw_orch::prelude::*;

// Bitsong Accounts Collection Framework Suite.
pub struct BtsgAccountSuite<Chain> {
    pub account: BitsongAccountCollection<Chain, Metadata>,
    pub market: BitsongAccountMarketplace<Chain>,
    pub minter: BitsongAccountMinter<Chain>,
}

impl<Chain: CwEnv> BtsgAccountSuite<Chain> {
    pub fn new(chain: Chain) -> BtsgAccountSuite<Chain> {
        BtsgAccountSuite::<Chain> {
            account: BitsongAccountCollection::new("bs721_account", chain.clone()),
            market: BitsongAccountMarketplace::new("bs721_account_market", chain.clone()),
            minter: BitsongAccountMinter::new("bs721_account_minter", chain.clone()),
        }
    }

    pub fn upload(&self) -> Result<(), CwOrchError> {
        let acc = self.account.upload()?.uploaded_code_id()?;
        let mark = self.market.upload()?.uploaded_code_id()?;
        let minter = self.minter.upload()?.uploaded_code_id()?;

        println!("account collection code-id: {}", acc);
        println!("account market code-id: {}", mark);
        println!("account minter code-id: {}", minter);
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
        vec![
            Box::new(&mut self.account),
            Box::new(&mut self.market),
            Box::new(&mut self.minter),
        ]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let suite = Self::new(chain.clone());
        Ok(suite)
    }

    fn deploy_on(chain: Chain, data: Self::DeployData) -> Result<Self, Self::Error> {
        // ########### Upload ##############
        let mut suite: BtsgAccountSuite<Chain> = BtsgAccountSuite::store_on(chain.clone())?;

        // ########## Instantiate #############
        // account marketplace
        let market = suite
            .market
            .instantiate(
                &bs721_account_marketplace::msg::InstantiateMsg {
                    trading_fee_bps: 100u64,
                    min_price: 100u128.into(),
                    ask_interval: 30u64,
                    max_renewals_per_block: 10u32,
                    valid_bid_query_limit: 100u32,
                    renew_window: 1000u64,
                    renewal_bid_percentage: Decimal::one(),
                    operator: chain.sender_addr().to_string(),
                },
                None,
                None,
            )?
            .instantiated_contract_address()?;

        println!("bs721-account marketplace contract: {}", market);
        // Account Minter
        // On instantitate, bs721-account contract is created by minter contract.
        // We grab this contract addr from response events, and set address in internal test suite state.
        let bs721_account = suite
            .minter
            .instantiate(
                &bs721_account_minter::msg::InstantiateMsg {
                    admin: Some(data.to_string()),
                    verifier: None,
                    collection_code_id: suite.account.code_id()?,
                    marketplace_addr: suite.market.addr_str()?,
                    min_account_length: 3u32,
                    max_account_length: 128u32,
                    base_price: 10u128.into(),
                },
                Some(&Addr::unchecked(data)),
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

        // Provide marketplace with collection and minter contracts.
        suite
            .market
            .setup(suite.account.address()?, suite.minter.address()?)?;
        Ok(suite)
    }
}
