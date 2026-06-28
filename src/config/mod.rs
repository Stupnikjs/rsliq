use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use crate::swap;
use crate::config; 
use std::env::var;
use std::str::FromStr;
use std::sync::Arc;

mod address; 


pub struct DexConfig {
    pub quoter: Address,
    pub router: Address,
    pub name: DexesName,
}

pub enum DexesName {
    UniswapV3,
    Pankake, 
    Aerodrome,  
}
pub struct Config {
    pub chain_id: u32,
    pub main_rpc: String,
    pub second_rpc: String,
    pub ws_rpc: String,
    pub morpho_addr: Address,
    pub liquidator_addr: Address,
    pub dexes: Vec<DexConfig>,
    pub signer: Arc<PrivateKeySigner>, 
}


pub fn new_dex_config(quoter: Address, router: Address, name: DexesName) -> DexConfig {
    DexConfig {
        quoter,
        router,
        name: name,
    }
}
pub fn load_base_config() -> Result<Config, anyhow::Error> {
    dotenvy::dotenv().ok();
    Ok(Config {
        chain_id: 8453,
        main_rpc:String::from_str("https://lb.drpc.live/base/AhuxMhCqfkI8pF_0y4Fpi89GWcIMFIwR8ZsatuZZzRRv")?,  // var("BASE_HTTP_DRPC").expect("BASE_HTTP_DRPC not set") ,
        second_rpc: String::new(),
        ws_rpc: String::from_str("wss://lb.drpc.live/base/AhuxMhCqfkI8pF_0y4Fpi89GWcIMFIwR8ZsatuZZzRRv")?,
        morpho_addr: config::address::MORPHO_MAINNET,
        liquidator_addr: config::address::BASE_LIQUIDATOR_LAST,
        dexes: vec![new_dex_config(address::BASE_UNISWAP_QUOTER_V2, address::BASE_UNISWAP_V3_ROUTER, DexesName::UniswapV3)] ,
        signer: Arc::new(PrivateKeySigner::from_str("ca9a3a3d4026e6228713e683a9c45ef65a538b2f9336813bd597f5effa38668d")?)
        // signer: Arc::new(PrivateKeySigner::from_str(&var("PRIV_K")?)?),
    })
}