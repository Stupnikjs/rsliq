#![allow(dead_code, unused_variables, unused_imports)]
mod refresh;
pub mod positions;
mod events;


use alloy_primitives::{Address, U256, FixedBytes};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::morpho::types::MarketParam;
use crate::api::market::{fetch_all_market_by_chainid}; 
use crate::api::pos::{fetch_all_positions, position_item_to_borrow_pos}; 
use positions::BorrowPosition;
use futures::stream::{self, StreamExt};

pub type MarketId = FixedBytes<32>;


#[derive(Default, Clone)]
pub struct MarketStats {
    pub total_borrow_assets: U256,
    pub total_borrow_shares: U256,
    pub borrow_rate: U256,
    pub max_collateral_pos: U256,
    pub oracle_price: U256,
}

pub struct Market {
    pub params: Arc<MarketParam>,
    pub canceled: bool,
    pub stats: MarketStats,
    pub active_index: usize,
    pub positions: Vec<BorrowPosition>, // trié par HF asc
}

pub struct MarketSnapshot {
    pub id: MarketId,
    pub params: Arc<MarketParam>,
    pub stats: MarketStats,
    pub positions: Vec<BorrowPosition>,
}

pub struct MarketCache {
    markets: RwLock<HashMap<MarketId, Arc<RwLock<Market>>>>,
}


impl MarketCache {
    pub fn new(markets: &[MarketParam]) -> Self {
        let map: HashMap<MarketId, Arc<RwLock<Market>>> = markets
            .iter()
            .map(|m| {
                // On copie et transfère les vrais paramètres ici !
                let market = Market {
                    params: Arc::new(m.clone()),
                    canceled: false,
                    stats: MarketStats::default(), // Les stats globales (volumes) seront mis à jour par l'indexeur
                    active_index: 0,
                    positions: Vec::new(), // Le tableau de positions démarre vide
                };
                
                let id_bytes: MarketId = m.id.into(); 
                (id_bytes, Arc::new(RwLock::new(market)))
            })
            .collect();

        Self { markets: RwLock::new(map) }
    }

    pub fn ids(&self) -> Vec<FixedBytes<32>> {
        self.markets
            .read()
            .unwrap()
            .iter()
            .filter(|(_, market)| !market.read().unwrap().canceled)
            // On prend le premier élément du tuple (la clé), et on ignore le second avec `_`
            .map(|(&id_bytes, _market)| FixedBytes::from(id_bytes)) 
            .collect()
    }

    pub fn get_market_param_by_id(&self, id: MarketId) -> Option<MarketParam> {
    self.markets
        .read()
        .unwrap()
        .get(&id)
        .map(|m| (*m.read().unwrap().params).clone())
} 

    pub fn update<F, R>(&self, id: MarketId, f: F) -> Option<R>
where
    F: FnOnce(&mut Market) -> R,
{
    // 1. Lock la HashMap juste le temps de récupérer l'Arc, puis relâche immédiatement
    let market_arc = {
        let guard = self.markets.read().unwrap();
        guard.get(&id)?.clone()
    };

    // 2. Lock le Market séparément — gère le poisoning sans paniquer
    let mut market = match market_arc.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(), // récupère quand même les données
    };

    Some(f(&mut market))
}

    pub fn snapshot(&self, id: MarketId) -> Option<MarketSnapshot>
    where
        BorrowPosition: Clone,
    {
        let guard = self.markets.read().unwrap();
        let market = guard.get(&id)?.read().unwrap();
        Some(MarketSnapshot {
            params: market.params.clone(),
            id,
            stats: market.stats.clone(),
            positions: market.positions.clone(),
        })
    }
}



