use cw_orch::{interface, prelude::*};

use crate::contract::{execute, instantiate, query, sudo};
use btsg_account::market::{ExecuteMsg, MarketplaceInstantiateMsg, QueryMsg};

/// Uploadable trait for bs721_account_minter & use with cw-orchestrator library
#[interface(MarketplaceInstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct BtsgAccountMarket;

impl<Chain> Uploadable for BtsgAccountMarket<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("bs721_account_marketplace")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_sudo(sudo))
    }
}
