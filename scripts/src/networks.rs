use cw_orch::environment::{ChainKind, NetworkInfo};
//////////////// SUPPORTED NETWORK CONFIGS ////////////////
/// Add more chains in SUPPORTED_CHAINS to include in account framework instance.
use cw_orch::prelude::{networks::OSMOSIS_1, *};
/// Cw-orch imports
use reqwest::Url;
use std::net::TcpStream;

pub const SUPPORTED_CHAINS: &[ChainInfo] = &[BITSONG_MAINNET, OSMOSIS_1];
pub const BITSONG_SUPPORTED_NETWORKS: &[ChainInfo] = SUPPORTED_CHAINS;
pub const GAS_TO_DEPLOY: u64 = 60_000_000;


/// A helper function to retrieve a [`ChainInfo`] struct for a given chain-id.
/// supported chains are defined by the `SUPPORTED_CHAINS` variable
pub fn bitsong_parse_networks(net_id: &str) -> Result<ChainInfo, String> {
    BITSONG_SUPPORTED_NETWORKS
        .iter()
        .find(|net| net.chain_id == net_id)
        .cloned()
        .ok_or(format!("Network not found: {}", net_id))
}

/// Bitsong: <https://github.com/cosmos/chain-registry/blob/master/bitsong/chain.json>
pub const BITSONG_NETWORK: NetworkInfo = NetworkInfo {
    chain_name: "Bitsong",
    pub_address_prefix: "bitsong",
    coin_type: 639u32,
};

pub const BITSONG_MAINNET: ChainInfo = ChainInfo {
    kind: ChainKind::Mainnet,
    chain_id: "bitsong-2b",
    gas_denom: "ubtsg",
    gas_price: 0.025,
    grpc_urls: &["http://bitsong-grpc.polkachu.com:16090"],
    network_info: BITSONG_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

pub const BITSONG_TESTNET: ChainInfo = ChainInfo {
    kind: ChainKind::Testnet,
    chain_id: "bobnet",
    gas_denom: "ubtsg",
    gas_price: 0.025,
    grpc_urls: &["http://"],
    network_info: BITSONG_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

// Localnet
const LOCAL_NET: NetworkInfo = NetworkInfo {
    chain_name: "Local Network",
    pub_address_prefix: "mock",
    coin_type: 114u32,
};

pub async fn ping_grpc(url_str: &str) -> anyhow::Result<()> {
    let parsed_url = Url::parse(url_str)?;

    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("No host in url"))?;

    let port = parsed_url.port_or_known_default().ok_or_else(|| {
        anyhow::anyhow!(
            "No port in url, and no default for scheme {:?}",
            parsed_url.scheme()
        )
    })?;
    let socket_addr = format!("{}:{}", host, port);

    let _ = TcpStream::connect(socket_addr);
    Ok(())
}
