#![allow(dead_code, unused_variables, unused_imports)]

use crate::{api::{fetch_all_market, market}, morpho::types::MarketParam};
mod morpho;
mod api;
mod cache; 

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let market_id = parse_market_id("0x8793cf302b8ffd655ab97bd1c695dbd967807e8367a65cb2f4edaf1380ba1bda")?;
    let chain_id = 8453u32; // 1 pour Ethereum Mainnet, ou 8453 pour Base (999 n'existe pas chez Morpho, attention !)
    let markets = fetch_all_market(chain_id).await?; 

    print!("{} markets found \n ", markets.len()); 
    // retourne tuple vide 
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
        // Tu peux maintenant afficher tes marchés.
        // J'affiche l'ID et l'actif de prêt (loan_asset) comme exemple.
    
        let morpho_m =  market::market_item_to_morpho_market(m, chain_id)?; 
        print!("{}\n", morpho_m.get_pair()); 
        all_morpho_markets.push(morpho_m);
    }


    Ok(all_morpho_markets)
}
