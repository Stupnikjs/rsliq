use alloy::rpc::types::{Filter, Log};
use alloy::primitives::{Address,keccak256, U256, FixedBytes};
use alloy::eips::BlockNumberOrTag;
use crate::cache::MarketCache;



  fn read_address(topic: &alloy::primitives::B256) -> Address {
    Address::from_slice(&topic.as_slice()[12..])
}

fn read_u256(data: &[u8], offset: usize) -> U256 {
    U256::from_be_slice(&data[offset..offset + 32])
}


impl MarketCache {
   

   pub fn process_log(&self, log: &Log) {
    let Some(topic0) = log.topics().first() else { return };

    match topic0 {
        x if *x == keccak256("Borrow(bytes32,address,address,address,uint256,uint256)") => {
            self.update_borrow(log);
        }
        x if *x == keccak256("Repay(bytes32,address,address,uint256,uint256)") => {
            self.update_repay(log);
        }
        x if *x == keccak256("Liquidate(bytes32,address,address,uint256,uint256,uint256,uint256,uint256)") => {
            self.update_liquidate(log);
        }
        x if *x == keccak256("AccrueInterest(bytes32,uint256,uint256,uint256)") => {
            self.update_accrue_interest(log);
        }
        _ => {}
    }
}

pub fn update_borrow(&self, log: &Log) {
    let market_id = FixedBytes::from(log.topics()[1]);
    let on_behalf = read_address(&log.topics()[2]);
    let shares = read_u256(&log.data().data, 32);
    self.update(market_id, |m| {
        if let Some(pos) = m.positions.iter_mut().find(|p| p.address == on_behalf) {
            pos.borrow_shares += shares;
        }
    });
}

pub fn update_repay(&self, log: &Log) {
    let market_id = FixedBytes::from(log.topics()[1]);
    let on_behalf = read_address(&log.topics()[2]);
    let shares = read_u256(&log.data().data, 32);
    self.update(market_id, |m| {
        if let Some(pos) = m.positions.iter_mut().find(|p| p.address == on_behalf) {
            pos.borrow_shares = pos.borrow_shares.saturating_sub(shares);
        }
    });
}

pub fn update_liquidate(&self, log: &Log) {
    let market_id = FixedBytes::from(log.topics()[1]);
    let borrower = read_address(&log.topics()[2]);
    self.update(market_id, |m| {
        m.positions.retain(|p| p.address != borrower);
    });
}

pub fn update_accrue_interest(&self, log: &Log) {

    let market_id = FixedBytes::from(log.topics()[1]);
    let interest = read_u256(&log.data().data, 32);
    println!("accrue interest {}", interest); 
    self.update(market_id, |m| {
        m.stats.total_borrow_assets += interest;
    });
}

}