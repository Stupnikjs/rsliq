#![allow(dead_code, unused_variables, unused_imports)]

use std::result;
use std::str::FromStr;
use crate::api::market;
use crate::morpho::types::MarketParam;
use crate::cache::{init_cache};
use crate::onchain::market::fetch_market; 
use alloy::providers::{ProviderBuilder, Provider};
use alloy::network::AnyNetwork;
use alloy_primitives::{Address, address};



mod morpho;
mod api;
mod cache; 
mod connector;
mod onchain;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let drpc_mainnet = "https://lb.drpc.live/ethereum/AhuxMhCqfkI8pF_0y4Fpi89GWcIMFIwR8ZsatuZZzRRv"; 
    let connector = connector::new(drpc_mainnet)?; 
    let morpho = address!("BBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCc");
    let market_id_hex = "eeabdcb98e9f7ec216d259a2c026bbb701971efae0b44eec79a86053f9b128b6";
    let bytes = hex::decode(market_id_hex)?;
   
    let mut market_id_bytes = [0u8; 32];
    market_id_bytes.copy_from_slice(&bytes);
        
    let market = fetch_market(&connector, morpho, &market_id_bytes).await
        .map_err(|e| format!("fetch_market error: {}", e))?;
    
    Ok(())
}