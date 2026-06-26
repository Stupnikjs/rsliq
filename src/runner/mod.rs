use std::sync::Arc;
use alloy::network::Ethereum;
use alloy::providers::Provider;
use alloy::pubsub::PubSubFrontend;
use alloy::rpc::types::Filter;
use futures_util::{Stream, StreamExt};
use alloy::primitives::{Address,keccak256, FixedBytes};
use alloy::eips::BlockNumberOrTag;
use alloy::rpc::types::{ Log};
use tokio::time::Sleep;


use crate::config::{Config, load_base_config};
use crate::connector::{self, Connector};
use crate::cache::MarketCache;
use crate::api::market::fetch_all_market_by_chainid;

pub struct Runner {
    config: Config,
    cache: Arc<MarketCache>,
    connector: Arc<Connector>,
} 

impl Runner{
    pub async fn new(chainid: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let config = match chainid {
            8453 => load_base_config(),
            _ => panic!("unsupported chain {}", chainid),
        };

        let cache = Arc::new(MarketCache::new(&[]));
        let connector = Arc::new(connector::build(&config.main_rpc.clone(), &config.ws_rpc.clone()).await?); 

        Ok(Self { config, cache, connector })
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let markets = fetch_all_market_by_chainid(self.config.chain_id).await?;
        print!("{} market fetched", markets.len()); 
        self.cache = Arc::new(MarketCache::new(&markets));
        self.cache.api_refresh(self.config.chain_id).await;

        print!("{} market watched ", self.cache.ids().len());

        // _ = self.subscribe().await?; 
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Task 1 : refresh périodique via API
        let cache_api = Arc::clone(&self.cache);
        let chain_id = self.config.chain_id;
        tokio::spawn(async move {
            loop {
                cache_api.api_refresh(chain_id).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });


        self.market_loop().await;
        let cache_ws = Arc::clone(&self.cache);

        Ok(())
    }
    
    pub async fn market_loop(&self) {
    for id in self.cache.ids() {
        let cache = Arc::clone(&self.cache);
        let connector = Arc::clone(&self.connector); // connector doit être Arc<Connector>
        let mut count = 0; 
        tokio::spawn(async move {
            loop {
                _ =  cache.onchain_oracle_refresh(&connector, id).await;
                
                if count % 10 == 0 {
                    cache.recompute_all_hf(id); 
                    cache.sort_by_hf(id); 
                }
                let interval = cache.refresh_interval(id);
                
                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
                count += 1; 
            }
        });
    }
}

/* 
  pub async fn subscribe(&self) -> Result<impl Stream<Item = Log>, Box<dyn std::error::Error>> {
    // self.connector.subscribe(self.config.morpho_addr, self.cache.process_log).await
    Ok()
}


    pub async fn market_spawn() 
    gère la détection du hf le tri 
    */


    }