use alloy::primitives::{Address, U256};
use alloy::sol_types::SolType;
use alloy::sol_types::sol_data::{self, FixedBytes};
use crate::connector::Connector; 
use alloy::providers::Provider;
use crate::onchain::encode::{encode_calldata, selector};   



#[derive(Debug)]
pub struct Market {
    pub total_supply_assets: u128,
    pub total_supply_shares: u128,
    pub total_borrow_assets: u128,
    pub total_borrow_shares: u128,
    pub last_update: u128,
    pub fee: u128,
}

// Market(bytes32) retourne 6 × uint128 packed en 3 × uint256 (ABI encode uint128 sur 32 bytes)
type MarketTuple = (
    sol_data::Uint<128>,
    sol_data::Uint<128>,
    sol_data::Uint<128>,
    sol_data::Uint<128>,
    sol_data::Uint<128>,
    sol_data::Uint<128>,
);

pub fn decode_market(data: &[u8]) -> Result<Market, Box<dyn std::error::Error>> {
    let (a, b, c, d, e, f) = MarketTuple::abi_decode(data)?;
    Ok(Market {
        total_supply_assets: a.try_into()?,
        total_supply_shares: b.try_into()?,
        total_borrow_assets: c.try_into()?,
        total_borrow_shares: d.try_into()?,
        last_update: e.try_into()?,
        fee: f.try_into()?,
    })
}

pub async fn fetch_market(
    connector: &Connector<impl Provider>,
    morpho: Address,
    market_id:  &[u8],
) -> Result<Market, Box<dyn std::error::Error>> {
    let sel = selector("market(bytes32)");
    let calldata = encode_calldata(sel,market_id);
    let raw = connector.call_raw(morpho, calldata).await?;

    println!("raw response len: {}", raw.len());
    println!("raw response: 0x{}", hex::encode(&raw));
    decode_market(&raw)
}