use alloy::primitives::{Address, U256};
use std::time::Instant;
use crate::morpho::types::{MarketParam}; 
use crate::swap::abi::uni::encode_exact_input_single_uni; 
use crate::swap::abi::pankake::encode_exact_input_single_pancake; 

pub mod quoter;
pub mod routes; 
pub mod abi; 




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