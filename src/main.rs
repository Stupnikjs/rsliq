#![allow(dead_code, unused_variables, unused_imports)]

use std::result;


use crate::morpho::types::MarketParam;
use crate::cache::{init_cache}; 
use alloy::providers::{ProviderBuilder, Provider};
use alloy::network::AnyNetwork;



mod morpho;
mod api;
mod cache; 
mod connector;
mod onchain;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let drpc_mainnet = "https://lb.drpc.live/ethereum/AhuxMhCqfkI8pF_0y4Fpi89GWcIMFIwR8ZsatuZZzRRv"; 
    
    let connector = connector::new(drpc_mainnet)?; 
    
    let weth: alloy::primitives::Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;
    let data: alloy::primitives::Bytes = "0x06fdde03".parse()?;

    let result = connector.call_raw(weth, data).await?;
    println!("Raw result: {:#x}", result);

    Ok(())
}
