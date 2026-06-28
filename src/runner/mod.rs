use std::sync::Arc;
use std::str::FromStr;
use alloy::signers::local::PrivateKeySigner;
use tokio::time::Duration;

use crate::config::{Config, load_base_config};
use crate::connector::{self, Connector};
use crate::cache::{MarketCache, WAD};
use crate::api::market::fetch_all_market_by_chainid;
use crate::liquidate;
use crate::swap::routes; 
use crate::swap::routes::RouteCache;

pub struct Runner {
    config: Config,
    cache: Arc<MarketCache>,
    connector: Arc<Connector>,
    routes: Arc<RouteCache>,
}

impl Runner {
    pub async fn new(chainid: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let config = match chainid {
            8453 => load_base_config()?,
            _ => panic!("unsupported chain {}", chainid),
        };

        let cache = Arc::new(MarketCache::new(&[]));
        let connector = Arc::new(connector::build(&config.main_rpc, &config.ws_rpc).await?);
        let routes = Arc::new(routes::new(vec![])); 
        Ok(Self { config, cache, connector, routes })
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let markets = fetch_all_market_by_chainid(self.config.chain_id).await?;
        
        self.cache = Arc::new(MarketCache::new(&markets));
        self.cache.api_refresh(self.config.chain_id).await;
        println!("{} markets watched", self.cache.ids().len());
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cache_api = Arc::clone(&self.cache);
        let chain_id = self.config.chain_id;
       

        self.market_loop().await;
         tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1800)).await;
                cache_api.api_refresh(chain_id).await;
                
            }
        });
        Ok(())
    }

    async fn market_loop(&self) {
        for id in self.cache.ids() {
            let cache = Arc::clone(&self.cache);
            let morpho_addr = self.config.morpho_addr.clone(); 
            let connector = Arc::clone(&self.connector);
            let mut count = 0u64;

            tokio::spawn(async move {
                loop {
                    
                    let _ = cache.onchain_oracle_refresh(&connector, id).await;
                    cache.log_market(id);
                    cache.recompute_all_hf(id);

                    if count % 10 == 0 {
                        let _ = cache.onchain_market_refresh(&connector, morpho_addr, id).await;
                        cache.sort_by_hf(id);
                    }

                    let (lowest, interval) = cache.lowest_hf_and_interval(id);

                    if let Some(pos) = lowest {
                        if pos.cached_hf.map_or(false, |hf| hf < WAD) {
                            let _ = liquidate::liquidate().await;
                        }
                    }

                    count += 1;
                    tokio::time::sleep(Duration::from_secs(interval)).await;
                }
            });
        }
    }
}