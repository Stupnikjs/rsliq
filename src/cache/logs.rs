use alloy_primitives::FixedBytes;
use crate::cache::MarketCache; 



impl MarketCache {

  pub fn log_market(&self, id: FixedBytes<32>) {
    match self.snapshot(id) {
        Some(snap) => println!("{} positions {} price {}", snap.params.get_pair(), snap.positions.len(), snap.stats.oracle_price),
        None => println!("market {:?} not ready yet (empty or unknown)", id),
    }
}
    
}