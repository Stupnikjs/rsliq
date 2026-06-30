use std::mem::swap;
use std::sync::{Arc, RwLock};

use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::Log;
use tokio::time::Duration;

use crate::config::{Config, load_base_config};
use crate::connector::{self, Connector};
use crate::cache::{MarketCache, WAD};
use crate::api::market::fetch_all_market_by_chainid;
use crate::liquidate;
use crate::swap::routes; 
use crate::swap::routes::RouteCache;


mod routine; 

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
        let conn = connector::build(&config.main_rpc, &config.ws_rpc, config.signer.clone()).await?;
        let connector = Arc::new(conn); 
        let route_cache = Arc::new(RwLock::new(RouteCache::new()));
        Ok(Self { config, cache, connector, route_cache })
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let markets = fetch_all_market_by_chainid(self.config.chain_id).await?;
        self.cache = Arc::new(MarketCache::new(&markets));
        self.cache.api_refresh(self.config.chain_id).await;
        self.quote_market().await?;  
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.connector.subscribe(self.config.morpho_addr, |log| {
        self.cache.process_log(&log);
        }).await?; 
        self.api_refresh_loop(3600).await; 
        self.market_loop().await; 
        Ok(())
    }


    
}