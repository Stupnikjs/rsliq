use std::result;
use std::str::FromStr;
use std::sync::Arc;

use alloy::providers::Provider;
use alloy::network::AnyNetwork;
use alloy_primitives::{Address, address};


use crate::config::{self, Config, load_base_config};
use crate::connector::{EthCaller, WsSubscriber};
use crate::connector; 
use crate::cache::MarketCache;
use crate::api::market::{self, fetch_all_market_by_chainid};
use crate::morpho::types::MarketParam;
use crate::cache::{MarketStats};
use crate::onchain::calls::{MarketStatsCall, market_call, oracle_call}; 



pub struct Runner<P, W> {
    config: Config,
    cache: Arc<MarketCache>,
    caller: EthCaller<P>,
    subscriber: WsSubscriber<W>,
}

impl<P: Provider> Runner<P> {
    pub fn new(chainid: u64) -> Result<Runner<impl Provider>, Box<dyn std::error::Error>> {
        let conf = match chainid {
            8453 => load_base_config(),
            _ => panic!("unsupported chain {}", chainid),
        };
       let main_rpc = conf.main_rpc.clone(); 
       let cache = Arc::new(MarketCache::new(&[]));
        let connector = connector::new(main_rpc)?;

        Ok(Runner { 
            config: conf, 
            cache: cache, 
            connector: connector })
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let markets = fetch_all_market_by_chainid(self.config.chain_id).await?;
        self.cache = Arc::new(MarketCache::new(&markets));
        self.cache.api_refresh(1).await;
        Ok(())
    }

    pub async fn run(&self) {
        let cache = Arc::clone(&self.cache);
        tokio::spawn(async move {
            cache.api_refresh(self.config.chain_id).await;
        });
    }
}

