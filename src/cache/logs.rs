
use alloy_primitives::{U256, FixedBytes};
use crate::cache::MarketCache; 
use crate::cache::WAD;


impl MarketCache {

  pub fn log_market(&self, id: FixedBytes<32>) {
    match self.snapshot(id) {
        Some(snap) => println!("{} positions {} price {}", snap.params.get_pair(), snap.positions.len(), snap.stats.oracle_price),
        None => println!("market {:?} not ready yet (empty or unknown)", id),
    }
  }

 pub fn hf_avg(&self, id: FixedBytes<32>) -> Result<U256, anyhow::Error> {
    let snap = self.snapshot(id).ok_or_else(|| anyhow::anyhow!("snapshot not found for {:?}", id))?;

    let mut sum = U256::ZERO;
    let mut count: u64 = 0;

    for p in snap.positions {
        if let Some(hf) = p.cached_hf {
            sum += hf;
            count += 1;
        }
    }

    if count == 0 {
        return Ok(U256::ZERO);
    }

    Ok(sum / U256::from(count))
  }

  pub fn count_below_hf_threshold(&self, id: FixedBytes<32>, pct: u64) -> Result<u64, anyhow::Error> {
    let snap = self.snapshot(id).ok_or_else(|| anyhow::anyhow!("snapshot not found for {:?}", id))?;

    let threshold = U256::from(100 + pct) * WAD / U256::from(100);

    let count = snap
        .positions
        .iter()
        .filter(|p| p.cached_hf.map_or(false, |hf| hf < threshold))
        .count() as u64;

    Ok(count)
}


}