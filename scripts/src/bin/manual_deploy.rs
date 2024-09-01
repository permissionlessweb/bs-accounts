use clap::Parser;
use cw_orch::{
    daemon::{DaemonBuilder, TxSender},
    prelude::*,
};
use scripts::{networks::ping_grpc, BtsgAccountSuite};
use tokio::runtime::Runtime;

// todo: move to .env file
pub const MNEMONIC: &str =
    "garage dial step tourist hint select patient eternal lesson raccoon shaft palace flee purpose vivid spend place year file life cliff winter race fox";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Network to deploy on: main, testnet, local
    #[clap(short, long)]
    network: String,
}

fn main() {
    // parse cargo command arguments for network type
    let args = Args::parse();
    // logs any errors
    env_logger::init();

    println!("Deploying Headstash Framework...");
    let bitsong_chain = match args.network.as_str() {
        "main" => scripts::networks::BITSONG_MAINNET.to_owned(),
        "testnet" => scripts::networks::BITSONG_TESTNET.to_owned(),
        "local" => scripts::networks::LOCAL_NETWORK1.to_owned(),
        _ => panic!("Invalid network"),
    };

    if let Err(ref err) = manual_deploy(bitsong_chain.into()) {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        ::std::process::exit(1);
    }
}

fn manual_deploy(network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    // rt.block_on(assert_wallet_balance(vec![network.clone()]));

    let urls = network.grpc_urls.to_vec();
    for url in urls {
        rt.block_on(ping_grpc(&url))?;
    }

    let chain = DaemonBuilder::new(network.clone())
        .handle(rt.handle())
        .mnemonic(MNEMONIC)
        .build()?;

    let _suite = BtsgAccountSuite::deploy_on(chain.clone(), chain.sender().address())?;
    // query account for connected wallet

    Ok(())
}
