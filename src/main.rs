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


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = ProviderBuilder::new()
        .connect_http("https://lb.drpc.live/base/AhuxMhCqfkI8pF_0y4Fpi89GWcIMFIwR8ZsatuZZzRRv".parse()?);
    let block = provider.get_block_number().await?;
    println!("block number = {}", block);    
    Ok(())
}
