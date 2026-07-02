use std::mem::swap;
use std::sync::{Arc, RwLock};

use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::Log;
use tokio::time::Duration;

use config::{Config, load_base_config};
use crate::connector::{self, Connector};
use crate::cache::{MarketCache};
use crate::api::market::fetch_all_market_by_chainid;
use crate::morpho::utils::WAD;
use crate::swap::routes; 
use crate::swap::routes::RouteCache;


mod routine; 
mod liquidate;
mod config;

pub struct Runner {
    config: Config,
    cache: Arc<MarketCache>,
    connector: Arc<Connector>,
    route_cache: Arc<RwLock<RouteCache>>,
}

impl Runner {
    pub async fn new(chainid: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let config = match chainid {
            8453 => load_base_config()?,
            _ => panic!("unsupported chain {}", chainid),
        };

        let cache = Arc::new(MarketCache::new(&[]));
        let conn = connector::build(&config.main_rpc, &config.ws_rpc, config.signer.clone(), chainid, 200).await?;
        let connector = Arc::new(conn); 
        let route_cache = Arc::new(RwLock::new(RouteCache::new()));
        Ok(Self { config, cache, connector, route_cache })
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let markets = fetch_all_market_by_chainid(self.config.chain_id).await?;
        self.cache = Arc::new(MarketCache::new(&markets));
        self.cache.api_refresh(self.config.chain_id).await;

        // geting oracle prices 
        for market_id in self.cache.ids() {
            let _ = self.cache.onchain_oracle_refresh(self.connector.as_ref(), market_id).await;
            let _ = self.cache.onchain_market_refresh(self.connector.as_ref(), self.config.morpho_addr, market_id).await;
            self.cache.recompute_all_hf(market_id);  
            
        }
        println!("init done"); 

        Ok(())
    }

    pub async fn run(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
    let sub_handle = {
        let this = Arc::clone(&self);
        tokio::spawn(async move {
            let cache = this.cache.clone();
            if let Err(e) = this
                .connector
                .subscribe(this.config.morpho_addr, move |log| {
                    cache.process_log(&log);
                })
                .await
            {
                eprintln!("subscribe task failed: {e}");
            }
        })
    };

    let refresh_handle = {
        let this = Arc::clone(&self);
        tokio::spawn(async move {
            this.api_refresh_loop(3600).await;
        })
    };

    let market_handle = {
        let this = Arc::clone(&self);
        tokio::spawn(async move {
            print!("spawning markets"); 
            this.market_loop().await;
        })
    };

    let _ = tokio::join!(sub_handle, refresh_handle, market_handle);
    Ok(())
}


    
}