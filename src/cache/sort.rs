use alloy_primitives::FixedBytes;
use crate::cache::{MarketCache, positions::BorrowPosition};



impl MarketCache {
    
    pub fn recompute_all_hf(&mut self, id: FixedBytes<32>) {
        let snap = self.snapshot(id).expect("snap not found");
        let mparam = self.get_market_param_by_id(id).expect("market param not found");

        let updated: Vec<BorrowPosition> = snap
            .positions
            .iter()
            .map(|p| {
                let mut new_pos = p.clone();
                new_pos.cached_hf = p.health_factor(
                    snap.stats.total_borrow_assets,
                    snap.stats.total_borrow_shares,
                    mparam.lltv,
                    snap.stats.oracle_price,
                );
                new_pos
            })
            .collect();

       _ = self.update(id, |m| {
        m.positions = updated; 
       })
    }
    pub fn sort_by_hf(&mut self, id: FixedBytes<32>) {
    _ = self.update(id, |m| {
        m.positions.sort_by(|a, b| {
            match (a.cached_hf, b.cached_hf) {
                (Some(a_hf), Some(b_hf)) => a_hf.cmp(&b_hf),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    });
}


    
}




