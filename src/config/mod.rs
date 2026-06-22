use alloy::primitives::Address;
use crate::swap;
use crate::config; 
use std::env::var;
use std::str::FromStr;

mod address; 


pub struct Config {
    pub chain_id: u32,
    pub main_rpc: String,
    pub second_rpc: String,
    pub morpho_addr: Address,
    pub liquidator_addr: Address,
    pub dexes: Vec<Box<dyn swap::Dex>>,
}


pub fn load_base_config() -> Config {
    dotenvy::dotenv().ok(); 
    let main_rpc = var("BASE_HTTP_DRPC").expect("BASE_HTTP_DRPC not set");
    Config {
        chain_id: 8543,
        main_rpc: main_rpc,
        second_rpc: String::new(),
        morpho_addr: config::address::MORPHO_MAINNET,
        liquidator_addr: config::address::BASE_LIQUIDATOR_LAST,
        dexes: vec![],
    }
}
