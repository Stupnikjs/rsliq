#![allow(dead_code, unused_variables, unused_imports)]

use std::result;
use std::str::FromStr;
use crate::api::market::{self, fetch_all_market_by_chainid};
use crate::morpho::types::MarketParam;
use crate::connector::Connector; 
use crate::cache::{MarketStats};
use crate::onchain::calls::{MarketStatsCall, market_call, oracle_call}; 
use alloy::providers::{ProviderBuilder, Provider};
use alloy::network::AnyNetwork;
use alloy_primitives::{Address, address};



mod morpho;
mod api;
mod cache; 
mod connector;
mod onchain;
mod config;
pub mod swap;


use std::sync::Arc;
use tokio::time::{sleep, Duration};



/*
Runner struct {
    conf config 
    cache MarketCache
    conn Connector
}



dans /runner/ 

config = load_base_config()
build runner.new(config) // pass config 

runner.init() 
- api call 
- onchain call 
- quote 

runner.lauch()
- spwan api refresh 
- spawn market refresh 

*/


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf; 
    let chain = std::env::args().nth(1).unwrap_or_else(|| {
    eprintln!("usage: rsliq <chain>");
    std::process::exit(1);
    });
    match chain.as_str() {
    "base"    => { conf = load_base_config() }
    _         => panic!("unknown chain: {}", chain),
    }
    let runner = Runner::new(conf); 
    runner.init(); 
    runner.run();
    let connector = Arc::new(connector::new(drpc_mainnet)?);
    let markets = fetch_all_market_by_chainid(1).await?;
    let cache = Arc::new(cache::MarketCache::new(&markets));

    cache.api_refresh(1).await;
    api_refresh_spawn(&cache)
    for id in cache.ids() {

        let conn = Arc::clone(&connector);
        let cache = Arc::clone(&cache);

        // en plus il faudrais l'api routine refresh 
        // 
        tokio::spawn(async move {
            let mut counter = 0; 
            loop {
                if counter % 10 == 0 {
                     if let Err(e) = cache.onchain_market_refresh(&conn, morpho_addr).await {
                    continue; 
                }
                
                }
                if let Err(e) = cache.onchain_oracle_refresh(&conn, id).await {
                    continue; 
                }
                

                // let interval = cache.get_refresh_interval(id).unwrap_or(12);
                // sleep(Duration::from_secs(interval)).await;
                  
                // interval = cache.check_liquidation 
                sleep(Duration::from_secs(12)).await; // ~1 block Base
                counter+=1; 
            }
        });
    }

    // garde le main en vie
    tokio::signal::ctrl_c().await?;
    Ok(())
}
