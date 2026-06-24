use alloy::primitives::{Address, U256};
use alloy::rpc::types::trace::geth::call;
use alloy::sol_types::SolType;
use alloy::sol_types::sol_data::{self, FixedBytes};
use crate::connector::Connector; 
use alloy::providers::Provider;
use alloy::network::Ethereum;
use crate::onchain::encode::{encode_calldata, selector};   

#[derive(Debug)]
pub struct MarketStatsCall {
    pub total_supply_assets: U256,
    pub total_supply_shares: U256,
    pub total_borrow_assets: U256,
    pub total_borrow_shares: U256,
    pub last_update: U256,
    pub fee: U256,
}

type MarketTuple = (
    sol_data::Uint<128>,
    sol_data::Uint<128>,
    sol_data::Uint<128>,
    sol_data::Uint<128>,
    sol_data::Uint<128>,
    sol_data::Uint<128>,
);

pub fn decode_market_stats(data: &[u8]) -> Result<MarketStatsCall, anyhow::Error> {
    if data.len() < 192 {
        return Err(anyhow::anyhow!("response too short"));
    }

    // chaque slot = 32 bytes, la valeur uint128 est dans les 16 bytes de droite
    let read_u128 = |slot: usize| -> U256 {
        let offset = slot * 32;
        U256::from_be_slice(&data[offset..offset + 32])
    };

    Ok(MarketStatsCall {
        total_supply_assets:  read_u128(0),
        total_supply_shares:  read_u128(1),
        total_borrow_assets:  read_u128(2),
        total_borrow_shares:  read_u128(3),
        last_update:          read_u128(4),
        fee:                  read_u128(5),
    })
}


pub fn decode_oracle_price(data: &[u8])-> Result<U256,anyhow::Error> {
    if data.len() < 32 {
        return Err(anyhow::anyhow!("response too short"));
    }
    Ok(U256::from_be_slice(data))
}

pub async  fn market_call<H: Provider<Ethereum>, W: Provider>(conn: &Connector<H, W>, morpho_addr:Address, market_id: &[u8] ) -> Result<MarketStatsCall, anyhow::Error>{
    let selector = selector("market(bytes32)"); 
    let calldata = encode_calldata(selector, market_id); 
    let resp = conn.call_raw(morpho_addr, calldata).await; 
    decode_market_stats(&resp.expect("market call failed"))
}



pub async  fn oracle_call<H: Provider, W: Provider>(conn: &Connector<H, W>, oracle_addr:Address) -> Result<U256, anyhow::Error>{
    let selector = selector("price()"); 
    let calldata = encode_calldata(selector, &[]); 
    let resp = conn.call_raw(oracle_addr, calldata).await; 
    decode_oracle_price(&resp.expect("erreur calling oracle"))
}

