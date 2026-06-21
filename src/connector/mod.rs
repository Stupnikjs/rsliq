mod rate_limiter;
use crate::connector::rate_limiter::{RateLimiter};

use alloy::{
    network::TransactionBuilder,
    primitives::{Address, Bytes},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest, transports::{RpcError, TransportErrorKind},
};

pub struct Connector<P> {
    provider: P,
    rate_limiter: RateLimiter
}

// Free function — not tied to a generic impl
pub fn new(rpc_url: &str) -> Result<Connector<impl Provider>, Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new()
        .connect_http(rpc_url.parse()?);

    Ok(Connector { 
        provider: provider, 
        rate_limiter: RateLimiter::new(200), 
    })
}

impl<P: Provider> Connector<P> {
  
    pub async fn call_raw(
        &self,
        to: Address,
        data: Bytes,
    ) -> Result<Bytes, RpcError<TransportErrorKind>> {
         self.rate_limiter.acquire().await; // bloque si quota épuisé

        let tx = TransactionRequest::default()
            .with_to(to)
            .with_input(data);

        self.provider.call(tx).await.map_err(|e| e.into())
    }
}
