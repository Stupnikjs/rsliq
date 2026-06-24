use std::sync::Arc;
use alloy::providers::Provider;
use alloy::pubsub::PubSubFrontend;
use alloy::rpc::types::Filter;
use futures_util::{Stream, StreamExt};
use alloy::primitives::{Address,keccak256};
use alloy::eips::BlockNumberOrTag;
use alloy::rpc::types::{Filter, Log};


use crate::config::{Config, load_base_config};
use crate::connector::{self, Connector};
use crate::cache::MarketCache;
use crate::api::market::fetch_all_market_by_chainid;

pub struct Runner {
    config: Config,
    cache: Arc<MarketCache>,
    connector: Connector,
} 

impl Runner {
    pub async fn new(chainid: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let config = match chainid {
            8453 => load_base_config(),
            _ => panic!("unsupported chain {}", chainid),
        };

        let cache = Arc::new(MarketCache::new(&[]));
        let connector = connector::new(config.main_rpc.clone(), config.ws_rpc.clone()).await?; 

        Ok(Self { config, cache, connector })
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let markets = fetch_all_market_by_chainid(self.config.chain_id).await?;
        self.cache = Arc::new(MarketCache::new(&markets));
        self.cache.api_refresh(self.config.chain_id).await;
        // full onchain refresh
        // full quote 

        _ = self.subscribe().await?; 
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

        // Task 2 : écoute les events Morpho et met à jour le cache
        let cache_ws = Arc::clone(&self.cache);

        Ok(())
    }


  pub async fn subscribe(&self) -> Result<impl Stream<Item = Log>, Box<dyn std::error::Error>> {
    let filter = Filter::new()
        .address(self.config.morpho_addr)
        .from_block(BlockNumberOrTag::Latest)
        .events([
            "Supply(bytes32,address,address,uint256,uint256)",
            "Borrow(bytes32,address,address,address,uint256,uint256)",
            "Repay(bytes32,address,address,uint256,uint256)",
            "Liquidate(bytes32,address,address,uint256,uint256,uint256,uint256,uint256)",
            "AccrueInterest(bytes32,uint256,uint256,uint256)",
        ]);
    self.connector.subscribe_logs(filter).await
}

pub async fn listen(&self, mut stream: impl Stream<Item = Log> + Unpin) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(log) = stream.next().await {
        let topic0 = log.topics()[0];
        match topic0 {
            x if x == keccak256("Supply(bytes32,address,address,uint256,uint256)")
              || x == keccak256("Borrow(bytes32,address,address,address,uint256,uint256)")
              || x == keccak256("Repay(bytes32,address,address,uint256,uint256)") => {
                let on_behalf = Address::from_slice(&log.topics()[2].as_slice()[12..]);
                self.cache.update_position(on_behalf);
            }
            x if x == keccak256("Liquidate(bytes32,address,address,uint256,uint256,uint256,uint256,uint256)") => {
                let borrower = Address::from_slice(&log.topics()[2].as_slice()[12..]);
                self.cache.invalidate(borrower);
            }
            x if x == keccak256("AccrueInterest(bytes32,uint256,uint256,uint256)") => {
                self.cache.accrue_interest();
            }
            _ => {}
        }
    }
    Ok(())
}
}