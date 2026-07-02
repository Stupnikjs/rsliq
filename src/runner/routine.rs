use crate::runner::{Runner, liquidate}; 
use std::fs;
use std::sync::Arc;
use std::str::FromStr;

use tokio::time::Duration;
use crate::cache::{MarketCache, logs::write_market_log};
use crate::swap::quoter::UniswapV3;
use crate::morpho::utils::WAD;


impl Runner  {
    pub async fn api_refresh_loop(&self, sec: u64) {
        let cache_api = Arc::clone(&self.cache);
        let chain_id = self.config.chain_id;
         tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
                cache_api.api_refresh(chain_id).await;
                
            }
        });
    }

    pub async fn market_loop(&self) {
        for id in self.cache.ids() {
            let cache = Arc::clone(&self.cache);
            let morpho_addr = self.config.morpho_addr.clone(); 
            let connector = Arc::clone(&self.connector);
            let route_cache = Arc::clone(&self.route_cache); 
            let mut count = 0u64;
            let liquidator_addr = self.config.liquidator_addr.clone(); 
            tokio::spawn(async move {
                loop { 
                    let _ = cache.onchain_oracle_refresh(&connector, id).await;
                    
                    cache.recompute_all_hf(id);

                    if count % 10 == 0 {
                        let _ = cache.onchain_market_refresh(&connector, morpho_addr, id).await;
                        cache.sort_by_hf(id);
                    }
                    let mparam = cache.get_market_param_by_id(id).expect("cat find mparam");
                    let (lowest, interval) = cache.lowest_hf_and_interval(id, mparam.is_eth_correlated() || mparam.is_btc_correlated());
                    if let (Some(pos), 0) = (lowest, interval) {
                        let route = route_cache.read().unwrap().get_edge(&pos.market_id).cloned();
                
                        if let Some(route) = route {
                            liquidate::liquidate(&connector, pos, route, mparam, liquidator_addr).await;
                        }
                }

                    let snap = cache.snapshot(id).expect("snap not found");
                    let _ = write_market_log(&snap, "data/logs");
                    count += 1;
                    tokio::time::sleep(Duration::from_secs(interval)).await;
                }
            });
        }
    }


    pub async fn quote_market(&self) -> Result<(), Box<dyn std::error::Error>> {
        let route_cache = Arc::clone(&self.route_cache);
        println!("{} markets watched", self.cache.ids().len());
        for id in self.cache.ids() {
            let _ = self.cache.onchain_oracle_refresh(&self.connector, id).await; 
            let param = self.cache.get_market_param_by_id(id).expect("error in runner init get market param"); 
            let swaper = UniswapV3::new(
                self.config.dexes[0].quoter, 
                self.config.dexes[0].router, 
                1800, 
                String::from_str("UNIV3")?); 

            let snap = self.cache.snapshot(id).expect("snap failed in quote init"); 
            let edge = swaper.best_amount_in(
                &self.connector, 
                param.collateral_token, 
                param.loan_token, 
                snap.stats.max_collateral_pos, 
                snap.stats.oracle_price, 
                param.max_slippage()).await; 
            
            let Some(edge) = edge else {
            self.cache.update(id, |m| m.canceled = true);
            continue;
            };
            let mut route_cache = self.route_cache.write().unwrap(); 
            route_cache.edges.push(edge);
        }
         Ok(())
    }
    pub fn load_quote(&self) -> Result<(), Box<dyn std::error::Error>> {
        let edges_json = fs::read_to_string("data/edges.json")?;
        let edges = serde_json::from_str::<Vec<crate::swap::PoolEdge>>(&edges_json)?;
        let mut route_cache = self.route_cache.write().unwrap(); 
        for edge in edges {
            route_cache.edges.push(edge);
        }
         Ok(())
    }
}
