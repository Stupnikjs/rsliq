mod morpho;
mod api; 


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let market_id = parse_market_id("0x8793cf302b8ffd655ab97bd1c695dbd967807e8367a65cb2f4edaf1380ba1bda")?; // ta marketUniqueKey en hex
    let chain_id = 8453u32; // Base, par ex.
    /* 
    let positions = api::fetch_all_positions(market_id, chain_id).await?;

    println!("fetched {} positions", positions.len());
    for p in &positions {
        println!(
            "user={} borrow_shares={} collateral={}",
            p.user.address,
            p.state.borrow_shares,
            p.state.collateral
        );
    }

    Ok(())
    */

     let markets = api::fetch_all_market(chain_id).await?;

    println!("fetched {} market", markets.len());
    for p in &markets {
        // passer tout les marché dans un Vec<MorphoMarket>
        println!(
         "markt {:?}", p.collateral_asset);
    }

    Ok(())

}

fn parse_market_id(s: &str) -> anyhow::Result<[u8; 32]> {
    let s = s.trim_start_matches("0x");
    let bytes = hex::decode(s)?;
    bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("market id must be 32 bytes"))
}