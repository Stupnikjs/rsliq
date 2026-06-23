use std::sync::Arc;
use alloy::providers::Provider;
use alloy::pubsub::PubSubFrontend;
use futures::StreamExt;

use crate::config::{Config, load_base_config};
use crate::connector::{EthCaller, WsSubscriber};
use crate::cache::MarketCache;
use crate::api::market::fetch_all_market_by_chainid;

pub struct Runner<P, W> {
    config: Config,
    cache: Arc<MarketCache>,
    caller: EthCaller<P>,
    subscriber: WsSubscriber<W>,
}

impl<P: Provider, W: Provider<PubSubFrontend>> Runner<P, W> {
    pub fn new(chainid: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let config = match chainid {
            8453 => load_base_config(),
            _ => panic!("unsupported chain {}", chainid),
        };

        let cache = Arc::new(MarketCache::new(&[]));
        let caller = EthCaller::new(&config.http_rpc)?;
        let subscriber = WsSubscriber::new(&config.ws_rpc)?;

        Ok(Self { config, cache, caller, subscriber })
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let markets = fetch_all_market_by_chainid(self.config.chain_id).await?;
        self.cache = Arc::new(MarketCache::new(&markets));
        self.cache.api_refresh(self.config.chain_id).await;
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
        self.subscriber.subscribe_morpho_events(cache_ws).await?;

        Ok(())
    }
}