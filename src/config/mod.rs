use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use crate::swap;
use crate::config; 
use std::env::var;
use std::str::FromStr;
use std::sync::Arc;

mod address; 


pub struct Config {
    pub chain_id: u32,
    pub main_rpc: String,
    pub second_rpc: String,
    pub ws_rpc: String,
    pub morpho_addr: Address,
    pub liquidator_addr: Address,
    pub dexes: Vec<Box<dyn swap::Dex>>,
    pub signer: Arc<PrivateKeySigner>, 
}


pub fn load_base_config() -> Result<Config, anyhow::Error> {
    dotenvy::dotenv().ok();
    Ok(Config {
        chain_id: 8453,
        main_rpc: var("BASE_HTTP_DRPC").expect("BASE_HTTP_DRPC not set"),
        second_rpc: String::new(),
        ws_rpc: var("BASE_WS_ALCH").expect("BASE_WS_ALCH not set"),
        morpho_addr: config::address::MORPHO_MAINNET,
        liquidator_addr: config::address::BASE_LIQUIDATOR_LAST,
        dexes: vec![],
        signer: Arc::new(PrivateKeySigner::from_str(&var("PRIV_K")?)?),
    })
}