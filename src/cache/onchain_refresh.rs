
use crate::cache::MarketCache; 
use crate::onchain::calls::{oracle_call, market_call}; 
use crate::connector::Connector; 
use alloy_primitives::{Address,FixedBytes};
use alloy::providers::Provider;


   impl MarketCache {
    pub async fn onchain_oracle_refresh(
        &self,
        conn: &Connector<impl Provider>,
        market_id: FixedBytes<32>,
    ) -> Result<(), anyhow::Error> {
        let params = self.get_market_param_by_id(market_id)
            .ok_or(anyhow::anyhow!("market not found"))?;

        let price = oracle_call(conn, params.oracle).await?;

        self.update(market_id, |m| {
            m.stats.oracle_price = price;
        });

        Ok(())
    }

    pub async fn onchain_market_refresh(
        &self,
        conn: &Connector<impl Provider>,
        morpho_addr:Address,
        market_id: FixedBytes<32>,
    ) -> Result<(), anyhow::Error> {
        let m_stats_result = market_call(conn, morpho_addr, market_id.as_slice()).await?;
        self.update(market_id, |m| {
            m.stats.total_borrow_assets = m_stats_result.total_borrow_assets;
            m.stats.total_borrow_shares = m_stats_result.total_borrow_shares;
        });

        Ok(())
    }
}