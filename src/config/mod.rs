use alloy::primitives::Address;
use crate::swap;
use std::env::var;
use std::str::FromStr;

pub struct Config {
    pub chain_id: u32,
    pub main_rpc: String,
    pub second_rpc: String,
    pub morpho_addr: Address,
    pub liquidator_addr: Address,
    pub dexes: Vec<Box<dyn swap::Dex>>,
}

pub fn load_base_config(chain_id: u32) -> Config {
    dotenvy::dotenv().ok();

    let main_rpc = var("BASE_HTTP_DRPC").expect("BASE_HTTP_DRPC not set");
    let morpho_addr = Address::from_str(
        &var("BASE_MORPHO_ADDR").expect("BASE_MORPHO_ADDR not set")
    ).expect("invalid BASE_MORPHO_ADDR");
    let liquidator_addr = Address::from_str(
        &var("BASE_LIQUIDATOR_ADDR").expect("BASE_LIQUIDATOR_ADDR not set")
    ).expect("invalid BASE_LIQUIDATOR_ADDR");

    Config {
        chain_id,
        main_rpc,
        second_rpc: String::new(),
        morpho_addr,
        liquidator_addr,
        dexes: vec![],
    }
}