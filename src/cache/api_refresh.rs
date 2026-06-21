use crate::api::fetch_all_positions; 
use crate::api::positions::position_item_to_borrow_pos; 
use crate::cache::MarketCache; 

impl MarketCache {
    pub async fn api_refresh(&self, chain_id: u32) {
        for id in self.ids() {
            if let Ok(positions) = fetch_all_positions(id, chain_id).await {
                if positions.len() > 5 {
                     let borrow_pos_arr: Vec<_> = positions
                    .into_iter()
                    .map(|p| position_item_to_borrow_pos(p, id))
                    .filter(|p| p.borrow_assets_usd > 1)
                    .collect();

                self.update(id, |m| {
                    m.positions = borrow_pos_arr;
                });
                }
               
            }
        }
    }
}