use crate::BtsgAccountMarketExecuteFns;
use abstract_interface::{AbstractIbc, AccountI, AnsHost, ModuleFactory, Registry};
use abstract_std::{native_addrs, ACCOUNT, ANS_HOST, MODULE_FACTORY, REGISTRY};
use account_registry_middleware::interface::AccountRegistryMiddleware;
use account_registry_middleware::ExecuteMsgFns;
use anyhow::anyhow;
use bs721_account::interface::BtsgAccountCollection;
use bs721_account_marketplace::interface::BtsgAccountMarket;
use bs721_account_minter::interface::BtsgAccountMinter;
use btsg_account::{
    Metadata, CURRENT_BASE_DELEGATION, CURRENT_BASE_PRICE, CURRENT_COOLDOWN_FEE,
    CURRENT_MINIMUM_BID_PRICE,
};
use cosmwasm_std::{
    coin, instantiate2_address, Binary, CanonicalAddr, Instantiate2AddressError, Uint128,
};
use cw_blob::interface::{CwBlob, DeterministicInstantiation};
use ownership_verifier::interface::TestingOwnershipVerifier;

use cw_orch::prelude::*;
pub struct BtsgAccountSuite<Chain>
where
    Chain: cw_orch::prelude::CwEnv,
{
    pub ans_host: AnsHost<Chain>,
    pub registry: Registry<Chain>,
    pub module_factory: ModuleFactory<Chain>,
    pub ibc: AbstractIbc<Chain>,
    // btsg-account
    pub nft: BtsgAccountCollection<Chain, Metadata>,
    pub middleware: AccountRegistryMiddleware<Chain>,
    pub minter: BtsgAccountMinter<Chain>,
    pub market: BtsgAccountMarket<Chain>,
    pub(crate) test_owner: TestingOwnershipVerifier<Chain>,
    pub(crate) account: AccountI<Chain>,
    pub(crate) blob: CwBlob<Chain>,
}

pub const BLS_PUBKEY: &str = "";
pub const CW_BLOB: &str = "cw:blob";

impl<Chain: CwEnv> BtsgAccountSuite<Chain> {
    pub fn new(chain: Chain) -> BtsgAccountSuite<Chain> {
        BtsgAccountSuite::<Chain> {
            middleware: AccountRegistryMiddleware::new("registry_middleware", chain.clone()),
            nft: BtsgAccountCollection::new("bs721_account", chain.clone()),
            minter: BtsgAccountMinter::new("bs721_account_minter", chain.clone()),
            market: BtsgAccountMarket::new("bs721_account_marketplace", chain.clone()),
            test_owner: TestingOwnershipVerifier::new("ownership_verifier", chain.clone()),
            ans_host: AnsHost::new(ANS_HOST, chain.clone()),
            registry: Registry::new(REGISTRY, chain.clone()),
            module_factory: ModuleFactory::new(MODULE_FACTORY, chain.clone()),
            ibc: AbstractIbc::new(&chain),
            account: AccountI::new(ACCOUNT, chain.clone()),
            blob: CwBlob::new(CW_BLOB, chain.clone()),
        }
    }

    pub fn upload(&self) -> Result<(), CwOrchError> {
        let acc_code_id = self.nft.upload()?.uploaded_code_id()?;
        let minter_code_id = self.minter.upload()?.uploaded_code_id()?;
        let market_code_id = self.market.upload()?.uploaded_code_id()?;

        // let middleware_code_id = self.middleware.upload()?.uploaded_code_id()?;
        // self.blob.upload_if_needed()?;
        // self.ans_host.upload()?;
        // self.registry.upload()?;
        // self.module_factory.upload()?;
        // self.account.upload()?;
        // self.account.upload()?;
        // self.ibc
        //     .upload()
        //     .map_err(|e| CwOrchError::AnyError(anyhow!(e.to_string())))?;
        // println!("Middleware code ID: {}", middleware_code_id);

        println!("{}: {}", self.nft.id(), acc_code_id);
        println!("{}: {}", self.minter.id(), minter_code_id);
        println!("{}: {}", self.market.id(), market_code_id);

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
        vec![
            Box::new(&mut self.nft),
            Box::new(&mut self.minter),
            Box::new(&mut self.market),
        ]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let suite = Self::new(chain.clone());

        Ok(suite)
    }

    fn deploy_on(chain: Chain, data: Self::DeployData) -> Result<Self, Self::Error> {
        // ########### Upload ##############
        let mut suite: BtsgAccountSuite<Chain> = BtsgAccountSuite::store_on(chain.clone())?;

        // // // // // // // // // // // // // // // // // //
        //  BTSG ACCOUNT TOKENS
        // // // // // // // // // // // // // // // // // //
        suite.market.instantiate(
            &btsg_account::market::MarketplaceInstantiateMsg {
                trading_fee_bps: 200,
                min_price: Uint128::from(CURRENT_MINIMUM_BID_PRICE),
                ask_interval: 60,
                valid_bid_query_limit: 30,
                cooldown_timeframe: 60 * 60 * 24 * 14 as u64, // 14 days
                cooldown_cancel_fee: coin(CURRENT_COOLDOWN_FEE.into(), "ubtsg"),
                hooks_admin: None,
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
                    collection_code_id: suite.nft.code_id()?,
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

        println!("minter contract: {}", suite.minter.addr_str()?);
        println!("collection contract: {}", bs721_account);
        let account = &Addr::unchecked(bs721_account);
        suite.nft.set_default_address(&account);
        suite.nft.set_address(&account);

        // Provide marketplace with collection and minter contracts.
        suite
            .market
            .setup(suite.nft.address()?, suite.minter.address()?)?;

        // suite.middleware.instantiate(
        //     &account_registry_middleware::InstantiateMsg {
        //         market: suite.market.addr_str()?,
        //         collection: suite.minter.addr_str()?,
        //         account_code_id: suite.account.code_id()?,
        //     },
        //     Some(&Addr::unchecked(data.clone())),
        //     &[],
        // )?;

        // // // // // // // // // // // // // // // // // //
        //  ABSTRACT ACCOUNTS
        // // // // // // // // // // // // // // // // // //
        // let admin = chain.sender_addr().to_string();
        // let creator_account_id: cosmrs::AccountId = admin.as_str().parse().unwrap();
        // let canon_creator = CanonicalAddr::from(creator_account_id.to_bytes());
        // let blob_code_id = suite.blob.code_id()?;
        // let expected_addr = |salt: &[u8]| -> Result<CanonicalAddr, Instantiate2AddressError> {
        //     instantiate2_address(&cw_blob::CHECKSUM, &canon_creator, salt)
        // };
        // suite.ans_host.deterministic_instantiate(
        //     &abstract_std::ans_host::MigrateMsg::Instantiate(
        //         abstract_std::ans_host::InstantiateMsg {
        //             admin: admin.to_string(),
        //         },
        //     ),
        //     blob_code_id,
        //     expected_addr(native_addrs::ANS_HOST_SALT)?,
        //     Binary::from(native_addrs::ANS_HOST_SALT),
        // )?;

        // suite.registry.deterministic_instantiate(
        //     &abstract_std::registry::MigrateMsg::Instantiate(
        //         abstract_std::registry::InstantiateMsg {
        //             admin: suite.middleware.addr_str()?,
        //             security_enabled: Some(true),
        //             namespace_registration_fee: None,
        //         },
        //     ),
        //     blob_code_id,
        //     expected_addr(native_addrs::REGISTRY_SALT)?,
        //     Binary::from(native_addrs::REGISTRY_SALT),
        // )?;

        // suite.module_factory.deterministic_instantiate(
        //     &abstract_std::module_factory::MigrateMsg::Instantiate(
        //         abstract_std::module_factory::InstantiateMsg {
        //             admin: admin.to_string(),
        //         },
        //     ),
        //     blob_code_id,
        //     expected_addr(native_addrs::MODULE_FACTORY_SALT)?,
        //     Binary::from(native_addrs::MODULE_FACTORY_SALT),
        // )?;
        // // We also instantiate ibc contracts
        // suite.ibc.instantiate(&Addr::unchecked(admin.clone()))?;
        // suite
        //     .registry
        //     .register_base(&suite.account)
        //     .map_err(|e| CwOrchError::AnyError(anyhow!(e.to_string())))?;
        // suite
        //     .registry
        //     .approve_any_abstract_modules()
        //     .map_err(|e| CwOrchError::AnyError(anyhow!(e.to_string())))?;

        // suite
        //     .middleware
        //     .update_config(None, None, None, Some(suite.registry.addr_str()?))?;

        Ok(suite)
    }
}
