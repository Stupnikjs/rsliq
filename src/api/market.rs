use crate::api::types::{MarketItem,MarketsResult};
use crate::api::HttpClient;
use crate::api::queries::markets_query;
use crate::morpho::types::{MarketParam}; 
use std::str::FromStr;
use alloy_primitives::{Address, U256, FixedBytes};

pub async fn fetch_all_market(
    chain_id: u32,
) -> anyhow::Result<Vec<MarketItem>> {
   
    let client = HttpClient::new();
    let mut all = Vec::new();

    let result: MarketsResult = client
        .query(&markets_query(chain_id)).await?; 
    
    all.extend(result.markets.items); 
      
    Ok(all)
}


pub fn market_item_to_morpho_market(item: &MarketItem, chain_id: u32) -> Result<MarketParam, anyhow::Error> {
    // 1. Convertir l'ID du marché (String hex) en FixedBytes<32> ou [u8; 32] selon ton type
    // Si ton MarketParams utilise FixedBytes<32> :
    let market_id = FixedBytes::<32>::from_str(&item.id)?;

    // 2. Convertir les adresses (String hex) en Address
    let loan_token = Address::from_str(&item.loan_asset.address)?;
    let collateral_asset = item.collateral_asset
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing collateral asset for market {}", item.id))?;
    let collateral_token = Address::from_str(&collateral_asset.address.as_str())?; 
    let oracle = Address::from_str(&item.oracle_address)?;
    let irm  = Address::from_str(&item.irm)?;

    // 3. Convertir le LLTV (ton type Number) en U256
    // Supposons que ton type Number ait une méthode pour récupérer un u128 ou un String
    let lltv_raw = item.lltv.parse_u128()?; // Utilise la méthode de ton type Number
    let lltv = U256::from(lltv_raw);
    let collateral_token_str = collateral_asset.symbol.clone(); 
    let collateral_token_decimals = collateral_asset.decimals as u16;
    
    let loan_token_str = item.loan_asset.symbol.clone();
    let loan_token_decimals = item.loan_asset.decimals as u16;
    // 4. Instancier et retourner ton struct MarketParams
    Ok(MarketParam {
        id: market_id,
        loan_token: loan_token,
        collateral_token: collateral_token,
        oracle: oracle,
        irm: irm,
        lltv: lltv,
        chain_id: chain_id,
        collateral_token_str: collateral_token_str,
        collateral_token_decimals: collateral_token_decimals, 
        loan_token_decimals: loan_token_decimals,
        loan_token_str: loan_token_str,
    })
}