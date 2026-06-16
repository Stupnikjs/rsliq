use crate::api::types::{MarketItem,MarketsResult};
use crate::api::HttpClient;
use crate::api::queries::markets_query;

pub async fn fetch_all_market(
    chain_id: u32,
) -> anyhow::Result<Vec<MarketItem>> {
    let client = HttpClient::new();
    let mut all = Vec::new();

        let result: MarketsResult = client
            .query(&markets_query(chain_id)).await?;

        let result = result;
        all.extend(result.markets.items);

   

    Ok(all)
}