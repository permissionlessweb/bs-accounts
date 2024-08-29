use clap::Parser;
use cosmwasm_std::Addr;
use cw_orch::{
    daemon::{DaemonBuilder, TxSender},
    prelude::{ChainInfoOwned, Deploy},
};
use scripts::{assert_wallet_balance, networks::ping_grpc, BtsgAccountSuite};
use tokio::runtime::Runtime;

// todo: move to .env file
pub const MNEMONIC: &str = "";
pub const GOV_MODULE: &str = "";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Network Id to deploy on
    #[clap(short, long)]
    network_id: String,
    #[arg(long)]
    chain_id: String,
}

fn main() {
    println!("Deploying Headstash Framework As Governance Module...",);
    let bitsong_chain = scripts::networks::BITSONG_MAINNET.to_owned();

    if let Err(ref err) = deploy_as_gov(bitsong_chain.into()) {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        // The backtrace is not always generated. Try to run this example
        // with `$env:RUST_BACKTRACE=1`.
        //    if let Some(backtrace) = e.backtrace() {
        //        log::debug!("backtrace: {:?}", backtrace);
        //    }

        ::std::process::exit(1);
    }
}

fn deploy_as_gov(network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(assert_wallet_balance(vec![network.clone()]));

    let urls = network.grpc_urls.to_vec();
    for url in urls {
        rt.block_on(ping_grpc(&url))?;
    }

    let mut chain = DaemonBuilder::new(network.clone())
        .handle(rt.handle())
        .mnemonic(std::env::var(MNEMONIC)?)
        .build()?;
    // send message under authorization of governance module
    chain.authz_granter(GOV_MODULE);
    BtsgAccountSuite::deploy_on(chain.clone(), Addr::unchecked(GOV_MODULE))?;

    Ok(())
}
