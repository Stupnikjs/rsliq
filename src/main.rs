#![allow(dead_code, unused_variables, unused_imports)]

use std::result;
use std::str::FromStr;
use crate::api::market::{self, fetch_all_market_by_chainid};
use crate::morpho::types::MarketParam;
use crate::connector::Connector; 
use crate::cache::{MarketStats, init_cache};
use crate::onchain::calls::{MarketStatsCall, market_call, oracle_call}; 
use alloy::providers::{ProviderBuilder, Provider};
use alloy::network::AnyNetwork;
use alloy_primitives::{Address, address};



mod morpho;
mod api;
mod cache; 
mod connector;
mod onchain;


use std::sync::Arc;
use tokio::time::{sleep, Duration};




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let drpc_mainnet = "https://lb.drpc.live/ethereum/AhuxMhCqfkI8pF_0y4Fpi89GWcIMFIwR8ZsatuZZzRRv"; 
    let connector = Arc::new(connector::new(drpc_mainnet)?);
    let markets = fetch_all_market_by_chainid(1).await?;
    let cache = Arc::new(cache::MarketCache::new(&markets));

    cache.api_refresh(1).await;

    for id in cache.ids() {
        let conn = Arc::clone(&connector);
        let cache = Arc::clone(&cache);
        tokio::spawn(async move {
            loop {
                if let Err(e) = cache.onchain_oracle_refresh(&conn, id).await {
                    println!("error on market {:?}: {}", id, e);
                }
                 // let interval = cache.get_refresh_interval(id).unwrap_or(12);
                // sleep(Duration::from_secs(interval)).await;
                sleep(Duration::from_secs(12)).await; // ~1 block Base
            }
        });
    }

    // garde le main en vie
    tokio::signal::ctrl_c().await?;
    Ok(())
}
