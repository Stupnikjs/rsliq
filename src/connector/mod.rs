use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::network::Ethereum;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use limiter::SimpleRateLimiter;

mod limiter;

#[derive(Clone)]
pub struct RpcConnector {
    provider: Arc<RootProvider<Ethereum>>,
    limiter: SimpleRateLimiter,
}

impl RpcConnector {
    pub fn new(url: &str, rps: u32) -> anyhow::Result<Self> {
        let provider = ProviderBuilder::new()
            .connect_http(url.parse()?);

        Ok(Self {
            provider: Arc::new(provider),
            limiter: SimpleRateLimiter::new(rps),
        })
    }

    pub async fn call<T, P>(&self, method: &str, params: P) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
        P: Serialize + Send + Sync,
    {
        self.limiter.acquire().await;

        let res = self.provider.request(method, params).await?;
        Ok(res)
    }
}