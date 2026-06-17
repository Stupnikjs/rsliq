#![allow(dead_code, unused_variables, unused_imports)]
use anyhow::Context;

use hex; 
use crate::api::types::{PositionItem, PositionsResult};
use crate::api::HttpClient;
use crate::api::queries::positions_query;

pub async fn fetch_all_positions(
    market_id: [u8; 32],
    chain_id: u32,
) -> anyhow::Result<Vec<PositionItem>> {
    let client = HttpClient::new();
    let mut all = Vec::new();
    let mut skip: i64 = 0;
    let str_id = format!("0x{}", hex::encode(market_id));

    loop {
        let result: PositionsResult = client
            .query(&positions_query(&str_id, chain_id, skip))
            .await
            .with_context(|| format!("fetch positions page skip={skip}"))?;

        let mp = result.market_positions;
        all.extend(mp.items);

        skip += mp.page_info.count as i64;
        if skip >= mp.page_info.count_total as i64 {
            break;
        }
    }

    Ok(all)
}