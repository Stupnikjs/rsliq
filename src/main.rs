#![allow(dead_code, unused_variables, unused_imports)]

use crate::api::{fetch_all_market,fetch_all_positions, market, positions::position_item_to_borrow_pos}; 
use crate::morpho::types::MarketParam;
use crate::cache::{BorrowPosition, MarketCache}; 

mod morpho;
mod api;
mod cache; 

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let chain_id = 8453u32; // 1 pour Ethereum Mainnet, ou 8453 pour Base (999 n'existe pas chez Morpho, attention !)
    let res: &[MarketParam] = &api_fetch_all_market_by_chainid(chain_id).await?; 
    let cache = cache::MarketCache::new(res);
    
    for id in cache.ids() {
       let position_items = fetch_all_positions(id, chain_id).await?; 
       let borrow_pos_vec: Vec<BorrowPosition> = Vec::new(); 
       for item in position_items {
            let borrow_pos = position_item_to_borrow_pos(item, id); 
            borrow_pos_vec.push(borrow_pos)
       }
       let ok = cache.update(id.0, |m| {
         let borrow_positions = position_item_to_borrow_pos(pos_item, market_id); 
         m.positions = borrow_positions;   
       })
    } 
    
    Ok(())
}


 pub async fn api_fetch_all_market_by_chainid(chain_id: u32) -> anyhow::Result<Vec<MarketParam>> {
     let market_result = api::fetch_all_market(chain_id).await;
    // On crée le vecteur qui va recevoir les marchés en cas de succès
    let mut all_markets = Vec::new();

    match market_result {
        // 1. On extrait la valeur "result" à l'intérieur du Ok
        Ok(result) => {
            all_markets.extend(result);
        }
        Err(e) => {
            // Si ça plante, on intercepte l'erreur ici !
            println!("❌ Erreur lors de la requête GraphQL : {:?}", e);
            // Affiche la cause exacte (ex: quel champ est 'null')
            println!("🔍 Cause détaillée : {}", e.root_cause());
            // On s'arrête ici en retournant l'erreur au main
            return Err(e);
        }
    }

    let mut all_morpho_markets:Vec<MarketParam> = Vec::new();
    
    for m in &all_markets {
        // Si la conversion réussit, on récupère le marché, sinon on passe au suivant
    if let Ok(morpho_m) = market::market_item_to_morpho_market(m, chain_id) {
        all_morpho_markets.push(morpho_m);
    } else {
        println!("error while getting params on {} market", m.id)
    }
    }


    Ok(all_morpho_markets)
}
