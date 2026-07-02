use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use alloy_primitives::U256;
use serde::{Serialize, de};
use crate::morpho::utils::{WAD, hf_to_f64}; 
use super::{MarketSnapshot};


#[derive(Serialize)]
struct MarketLog {
    pair: String,
    ts: u64,
    total_positions: usize,
    positions_below_5pct: usize,
    positions_below_20pct: usize,
    positions_below_50pct: usize,
    total_borrowed_collateral_equiv: String, // string pour ne pas perdre de précision en JSON
    price_normalized: f64,                   // oracle_price / 1e36
}

pub fn write_market_log(snapshot: &MarketSnapshot, log_dir: &str) -> std::io::Result<()> {
    let pair = snapshot.params.get_pair().to_string();
    let ts = now_ms();

    let threshold_5 = WAD * U256::from(5u64) / U256::from(100u64);
    let threshold_20 = WAD * U256::from(20u64) / U256::from(100u64);
    let threshold_50 = WAD * U256::from(50u64) / U256::from(100u64);

    let mut below_5 = 0usize;
    let mut below_20 = 0usize;
    let mut below_50 = 0usize;

    for pos in &snapshot.positions {
        if let Some(hf) = pos.cached_hf {
            if hf < threshold_5 {
                below_5 += 1;
            }
            if hf < threshold_20 {
                below_20 += 1;
            }
            if hf < threshold_50 {
                below_50 += 1;
            }
        }
    }

    // total emprunté converti en unités collateral : loan_amount * 1e36 / price
    let e36 = U256::from(10u64).pow(U256::from(36u64));
    let total_borrowed_collateral_equiv = snapshot
        .stats
        .total_borrow_assets
        .checked_mul(e36)
        .and_then(|v| v.checked_div(snapshot.stats.oracle_price))
        .unwrap_or(U256::ZERO);

    let dec = 36 + snapshot.params.loan_token_decimals as u32 - snapshot.params.collateral_token_decimals as u32 ;      
    let price_normalized = snapshot.stats.oracle_price.to_string().parse::<f64>().unwrap_or(0.0) / 10_f64.powi(dec as i32); 

    let log = MarketLog {
        pair,
        ts,
        total_positions: snapshot.positions.len(),
        positions_below_5pct: below_5,
        positions_below_20pct: below_20,
        positions_below_50pct: below_50,
        total_borrowed_collateral_equiv: total_borrowed_collateral_equiv.to_string(),
        price_normalized,
    };

    fs::create_dir_all(log_dir)?;
    let path = format!("{}/log_{}.json", log_dir, ts);
    let json = serde_json::to_string_pretty(&log)?;
    let mut file = fs::File::create(path)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}