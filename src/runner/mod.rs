use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;

use crate::config::{Config, load_base_config};
use crate::connector::{self, Connector};
use crate::cache::{MarketCache, WAD};
use crate::api::market::fetch_all_market_by_chainid;


pub struct Runner {
    config: Config,
    cache: Arc<MarketCache>,
    connector: Arc<Connector>,
    wallet: Arc<Mutex<LocalWallet>>,
}

impl Runner {
    pub async fn new(chainid: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let config = match chainid {
            8453 => load_base_config(),
            _ => panic!("unsupported chain {}", chainid),
        };

        let cache = Arc::new(MarketCache::new(&[]));
        let connector = Arc::new(connector::build(&config.main_rpc, &config.ws_rpc).await?);
        let wallet = Arc::new(Mutex::new(LocalWallet::from_str(&config.private_key)?));

        Ok(Self { config, cache, connector, wallet })
    }

    pub async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let markets = fetch_all_market_by_chainid(self.config.chain_id).await?;
        println!("{} markets fetched", markets.len());
        self.cache = Arc::new(MarketCache::new(&markets));
        self.cache.api_refresh(self.config.chain_id).await;
        println!("{} markets watched", self.cache.ids().len());
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Task 1 : refresh API périodique
        let cache_api = Arc::clone(&self.cache);
        let chain_id = self.config.chain_id;
        tokio::spawn(async move {
            loop {
                cache_api.api_refresh(chain_id).await;
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });

        // Task 2 : une loop par market
        self.market_loop().await;

        Ok(())
    }

    async fn market_loop(&self) {
        for id in self.cache.ids() {
            let cache = Arc::clone(&self.cache);
            let connector = Arc::clone(&self.connector);
            let wallet = Arc::clone(&self.wallet);
            let mut count = 0u64;

            tokio::spawn(async move {
                loop {
                    let _ = cache.onchain_oracle_refresh(&connector, id).await;

                    if count % 10 == 0 {
                        cache.recompute_all_hf(id);
                        cache.sort_by_hf(id);
                    }
                    let lowest = cache.lowest_hf(id); 
                    if lowest.cached_hf < WAD {
                        if let Some(candidate) =  {
                            let mut w = wallet.lock().await;
                            let _ = liquidate(&mut w, &connector, candidate).await;
                        }
                    }

                    count += 1;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
        }
    }
}