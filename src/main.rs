#![allow(dead_code, unused_variables, unused_imports)]

use std::result;
use std::str::FromStr;
use crate::api::market::{self, fetch_all_market_by_chainid};
use crate::morpho::types::MarketParam;
use crate::connector::Connector; 
use crate::cache::{MarketStats, init_cache};
use crate::onchain::calls::{market_call, MarketStatsCall}; 
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
    let markets = fetch_all_market_by_chainid(1).await?;
    println!("len markets {}", markets.len()); 
    for m in markets.iter().take(3) {
    let market_stat = market_call_wrap(&connector, &m.id.0).await? ; 
    println!("{:?}", market_stat) ;
}
    Ok(())
}



async fn market_call_wrap(connector:&Connector<impl Provider>, market_id_bytes: &[u8]) ->  Result<MarketStatsCall, Box<dyn std::error::Error>> {
    let morpho = address!("0xBBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb");
        
    let market = market_call(connector, morpho, &market_id_bytes).await
        .map_err(|e| format!("fetch_market error: {}", e))?;
    Ok(market)
}