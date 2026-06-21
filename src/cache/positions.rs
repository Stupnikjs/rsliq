use alloy_primitives::{Address,FixedBytes, U256};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BorrowPosition {
    pub market_id: FixedBytes<32>,
    pub address: Address,
    pub borrow_shares: U256,
    pub borrow_assets_usd: U256,
    pub collateral_assets: U256,
    pub cached_hf: U256, 

}



