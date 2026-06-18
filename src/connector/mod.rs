use alloy::{
    network::Ethereum,
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::client::{BatchRequest, ClientBuilder, RpcClient},
    transports::http::{Client, Http},
};
use governor::{Quota, RateLimiter, state::{NotKeyed, InMemoryState}, clock::DefaultClock};
use nonzero_ext::nonzero;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// ---------------------------------------------------------------------------
// BatchElem
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BatchElem {
    pub method: String,
    pub args:   Value,           // serde_json::Value — flexible
    pub result: Option<Value>,
    pub error:  Option<String>,
}

// ---------------------------------------------------------------------------
// EthConnector
// ---------------------------------------------------------------------------

pub struct EthConnector {
    primary: RpcClient<Http<Client>>,
    second:  RpcClient<Http<Client>>,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl EthConnector {
    pub fn new(primary_url: &str, second_url: &str) -> Self {
        let primary = ClientBuilder::default()
            .http(primary_url.parse().expect("invalid primary URL"));
        let second = ClientBuilder::default()
            .http(second_url.parse().expect("invalid second URL"));

        let quota = Quota::per_minute(nonzero!(200u32))
            .allow_burst(nonzero!(10u32));

        Self {
            primary,
            second,
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }
}

// ---------------------------------------------------------------------------
// call_ctx
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("connector: rate limit exceeded")]
    RateLimited,
    #[error("rpc error: {0}")]
    Rpc(String),
    #[error("context cancelled")]
    ContextCancelled,
}

impl EthConnector {
    pub async fn call_ctx(
        &self,
        ctx: CancellationToken,
        calls: &mut Vec<BatchElem>,
    ) -> Result<(), ConnectorError> {
        self.limiter.until_ready().await;

        // Tentative primaire
        let primary_result = tokio::select! {
            res = Self::dispatch_batch(&self.primary, calls) => res,
            _ = ctx.cancelled() => return Err(ConnectorError::ContextCancelled),
        };

        if primary_result.is_ok() {
            return primary_result;
        }

        if ctx.is_cancelled() {
            return Err(ConnectorError::ContextCancelled);
        }

        // Fallback secondaire
        tokio::select! {
            res = Self::dispatch_batch(&self.second, calls) => res,
            _ = ctx.cancelled() => Err(ConnectorError::ContextCancelled),
        }
    }

    async fn dispatch_batch(
        client: &RpcClient<Http<Client>>,
        calls: &mut Vec<BatchElem>,
    ) -> Result<(), ConnectorError> {
        let mut batch = client.new_batch();

        // On enregistre chaque call dans le batch et on garde les futures
        let futs: Vec<_> = calls
            .iter()
            .map(|elem| {
                batch.add_call::<Value, Value>(
                    &elem.method,
                    &elem.args,
                )
                // add_call retourne un JoinHandle-like (Waiter<Value>)
                .expect("failed to add call to batch")
            })
            .collect();

        // Envoie le batch HTTP en une seule requête
        batch.send().await.map_err(|e| ConnectorError::Rpc(e.to_string()))?;

        // Résout chaque future et réécrit BatchElem
        for (elem, fut) in calls.iter_mut().zip(futs) {
            match fut.await {
                Ok(val)  => elem.result = Some(val),
                Err(e)   => elem.error  = Some(e.to_string()),
            }
        }

        Ok(())
    }
}