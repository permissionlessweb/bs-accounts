use clap::Parser;
use cw_orch::prelude::ChainInfoOwned;
use scripts::{assert_wallet_balance, networks::ping_grpc};
use tokio::runtime::Runtime;

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
    println!("Deploying Headstash Framework...");
    let bitsong_chain = scripts::networks::BITSONG_MAINNET.to_owned();

    if let Err(ref err) = manual_deploy(bitsong_chain.into()) {
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

fn manual_deploy(network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(assert_wallet_balance(vec![network.clone()]));

    let urls = network.grpc_urls.to_vec();
    for url in urls {
        rt.block_on(ping_grpc(&url))?;
    }

    Ok(())
}
