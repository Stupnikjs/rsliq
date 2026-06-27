use alloy_primitives::FixedBytes;
use crate::cache::MarketCache; 



impl MarketCache {

    pub fn log_market(&self, id: FixedBytes<32>) {
        let snap = self.snapshot(id).expect("snapshot failed"); 
        println!("{} positions {}", snap.params.get_pair(), snap.positions.len() )
    }
    
}