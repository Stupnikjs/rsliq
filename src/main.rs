#![allow(dead_code, unused_variables, unused_imports)]

use std::result;

use crate::api::{fetch_all_market,fetch_all_positions, market::api_fetch_all_market_by_chainid, positions::position_item_to_borrow_pos}; 
use crate::morpho::types::MarketParam;
use crate::cache::{BorrowPosition, MarketCache}; 
use futures::stream::{self, StreamExt};


mod morpho;
mod api;
mod cache; 

#[tokio::main]
async fn main() -> anyhow::Result<()> {
 let cache = init_cache().await?;
 print!("{}", cache.ids().len()); 
    Ok(())
}





const CONCURRENCY: usize = 100;

async fn init_cache() -> anyhow::Result<MarketCache> {
    let chain_id = 8453u32;
    let markets = api_fetch_all_market_by_chainid(chain_id).await?;
    let cache = cache::MarketCache::new(&markets);
    println!("markets len {}", cache.ids().len()); 
    stream::iter(cache.ids())
        .map(|id| async move {
            (id, fetch_all_positions(id, chain_id).await)
        })
        .buffer_unordered(CONCURRENCY)
        .for_each(|(id, result)| {
            match result {
                Ok(items) => {
                    if items.is_empty() || items.len() < 5 {
                        cache.update(id.0, |m| m.canceled = true);          
                    } else {
                        let positions: Vec<_> = items
                        .into_iter()
                        .map(|item| position_item_to_borrow_pos(item, id))
                        .collect();
                    cache.update(id.0, |m| m.positions = positions);
                    }
                  
                } 
                Err(err) => {
                    eprintln!("Erreur fetch positions {:?}: {}", id, err); 
                    
                }
            }
            futures::future::ready(())
        })
        .await;

    Ok(cache)
}