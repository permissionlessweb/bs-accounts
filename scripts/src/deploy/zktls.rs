use bs721_account::entry::sudo;
use btsg_irl::{
    contract::{execute, instantiate, query},
    ExecuteMsg, InstantiateMsg, QueryMsg,
};
use cw_orch::{interface, prelude::*};

/// Uploadable trait for bs721_account & use with cw-orchestrator library
#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct BtsgZktls;

impl<Chain> Uploadable for BtsgZktls<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("btsg_zktls")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_sudo(sudo))
    }
}
