use std::env;

use clap::Parser;
use cosmwasm_std::Addr;
use cw_orch::{
    daemon::{DaemonBuilder, TxSender},
    prelude::{ChainInfoOwned, Deploy},
};
use scripts::{assert_wallet_balance, networks::ping_grpc, BtsgAccountSuite};
use tokio::runtime::Runtime;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Network to deploy on: main, testnet, local
    #[clap(short, long)]
    network: String,
}

fn main() {
    let args = Args::parse();
    
    println!("Deploying Headstash Framework As Governance Module...",);
    let bitsong_chain = match args.network.as_str() {
        "main" => scripts::networks::BITSONG_MAINNET.to_owned(),
        "testnet" => scripts::networks::BITSONG_TESTNET.to_owned(),
        "local" => scripts::networks::LOCAL_NETWORK1.to_owned(),
        _ => panic!("Invalid network"),
    };

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

    let mnemonic = env::var("MNEMONIC")?;
    let gov_module = env::var("GOV_MODULE").expect("GOV_MODULE must be set");

    rt.block_on(assert_wallet_balance(vec![network.clone()]));

    let urls = network.grpc_urls.to_vec();
    for url in urls {
        rt.block_on(ping_grpc(&url))?;
    }

    let mut chain = DaemonBuilder::new(network.clone())
        .handle(rt.handle())
        .mnemonic(std::env::var(mnemonic)?)
        .build()?;
    // send message under authorization of governance module
    chain.authz_granter(gov_module.clone());
    BtsgAccountSuite::deploy_on(chain.clone(), Addr::unchecked(gov_module))?;

    Ok(())
}
