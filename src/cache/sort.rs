use alloy_primitives::{FixedBytes, U256};
use crate::cache::{MarketCache, positions::BorrowPosition,WAD};



impl MarketCache {
    
    pub fn recompute_all_hf(&self, id: FixedBytes<32>) {
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
    pub fn sort_by_hf(&self, id: FixedBytes<32>) {
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
    pub fn lowest_hf_and_interval(&self, id: FixedBytes<32>) -> (Option<BorrowPosition>, u64) {
    let Some(snap) = self.snapshot(id) else {
    return (None, 3600);
    };
    let Some(first) = snap.positions.first().cloned() else {
        return (None, 3600);
    };

    let Some(hf) = first.cached_hf else {
        return (Some(first), 3600);
    };

    let interval = if hf < WAD {
        0
    } else if hf < WAD * U256::from(105u64) / U256::from(100u64) {
        5
    } else if hf < WAD * U256::from(110u64) / U256::from(100u64) {
        15
    } else if hf < WAD * U256::from(120u64) / U256::from(100u64) {
        60
    } else if hf < WAD * U256::from(150u64) / U256::from(100u64) {
        300
    } else {
        3600
    };

    (Some(first), interval)
}
  pub fn lowest_hf(&self, id: FixedBytes<32>) -> BorrowPosition {
    self.snapshot(id).expect("snapshot failed while finding lowest_hf").positions[0].clone()
  } 

    
}





