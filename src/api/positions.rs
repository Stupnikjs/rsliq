#![allow(dead_code, unused_variables, unused_imports)]
use alloy_primitives::{FixedBytes, U256, Address}; 
use anyhow::Context;

use hex;
use tokio::runtime::Id; 
use std::str::FromStr;
use crate::api::types::{PositionItem, PositionsResult};
use crate::api::{HttpClient, positions};
use crate::api::queries::positions_query;
use crate::cache::{BorrowPosition, MarketCache};

pub async fn fetch_all_positions(
    market_id: FixedBytes<32>,
    chain_id: u32,
) -> anyhow::Result<Vec<PositionItem>> {
    let client = HttpClient::new();
    let mut all = Vec::new();
    let mut skip: i64 = 0;
    let id_string = format!("{:?}", market_id);

    loop {
        let result: PositionsResult = client
            .query(&positions_query(&id_string, chain_id, skip))
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




pub fn position_item_to_borrow_pos(
    pos_item: PositionItem, 
    market_id: FixedBytes<32>
) -> BorrowPosition {
    // 1. Convertir l'adresse String en type Address d'Alloy
    // Si l'adresse est mal formée, on fallback sur Address::ZERO pour éviter un panic
    let address = Address::from_str(&pos_item.user.address).unwrap_or(Address::ZERO);

    // 2. Extraire et convertir les valeurs numériques de ton type "Number" vers U256
    // Note : Ajuste `.value` ou `.to_u256()` selon la structure réelle de ton type Number
    let borrow_shares = U256::from_str(&pos_item.state.borrow_shares.to_string())
        .unwrap_or(U256::ZERO);
        
    let borrow_assets_usd = U256::from_str(&pos_item.state.borrow_assets_usd.to_string())
        .unwrap_or(U256::ZERO);
        
    let collateral_assets = U256::from_str(&pos_item.state.collateral.to_string())
        .unwrap_or(U256::ZERO);

    BorrowPosition {
        market_id,
        address,
        borrow_shares,
        borrow_assets_usd,
        collateral_assets,
        cached_hf: U256::ZERO, // 0 par défaut comme demandé, le calcul se fera après
    }
}


