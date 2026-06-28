use alloy::primitives::{Address, U256};
use std::time::Instant;
use crate::morpho::types::{MarketParam}; 

pub mod uni;
pub mod routes; 
pub trait Dex: Send + Sync {
    fn best_amount_in(
        &self,
        market: MarketParam,
        amount_in: U256,
        oracle_price: U256,
    ) -> Option<PoolEdge>;

    fn dex_name(&self) -> &str;
    fn quoter_address(&self) -> Address;
    fn router_address(&self) -> Address;
}



#[derive(Debug, Clone)]
pub struct SwapStep {
    pub target: Address,
    pub data: Vec<u8>,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in_offset: U256,
}

#[derive(Debug, Clone)]
pub struct PoolEdge {
    pub token_in: Address,
    pub token_out: Address,
    pub quoter: Address,
    pub router: Address,
    pub fee: u32,
    pub wc_slippage: f64,
    pub wc_amount_in: U256,
    pub wc_amount_out: U256,
    pub calibrated_at: Instant,
    pub dex_name: String,
    pub amount_in_offset: i64,
    pub price_at_quote: U256,
}