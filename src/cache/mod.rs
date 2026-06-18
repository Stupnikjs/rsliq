#![allow(dead_code, unused_variables, unused_imports)]

use alloy_primitives::{Address, U256, FixedBytes};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::morpho::types::MarketParam;

pub type MarketId = [u8; 32];

pub struct Oracle {
    pub price: U256,
    pub address: Address,
}

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


    pub fn update<F>(&self, id: MarketId, f: F) -> bool
    where
        F: FnOnce(&mut Market),
    {
        let guard = self.markets.read().unwrap();
        match guard.get(&id) {
            Some(market) => {
                f(&mut market.write().unwrap());
                true
            }
            None => false,
        }
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



#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BorrowPosition {
    pub market_id: FixedBytes<32>,
    pub address: Address,
    pub borrow_shares: U256,
    pub borrow_assets_usd: U256,
    pub collateral_assets: U256,
    pub cached_hf: U256, // Jamais nil, 0 par défaut si non calculé
}

// Pour que le cache trie automatiquement du pire au meilleur Health Factor (HF)
impl Ord for BorrowPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // On trie par HF inversé (le plus petit HF a la priorité absolue)
        other.cached_hf.cmp(&self.cached_hf)
    }
}

impl PartialOrd for BorrowPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}



impl MarketCache {

    pub fn ids(&self) -> Vec<FixedBytes<32>> {
        self.markets
            .read()
            .unwrap()
            .iter()
            // On prend le premier élément du tuple (la clé), et on ignore le second avec `_`
            .map(|(&id_bytes, _market)| FixedBytes::from(id_bytes)) 
            .collect()
    }
}
