use alloy::rpc::client::{ClientBuilder, RpcClient};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use nonzero_ext::nonzero;
use serde_json::{Value, json};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// BatchElem
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BatchElem {
    pub method: String,
    pub args: Value,
    pub result: Option<Value>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// ConnectorError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("connector: rate limit exceeded")]
    RateLimited,
    #[error("rpc error: {0}")]
    Rpc(String),
}

// ---------------------------------------------------------------------------
// EthConnector
// ---------------------------------------------------------------------------

pub struct EthConnector {
    primary: RpcClient,
    secondary: RpcClient,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}
impl EthConnector {
    // -----------------------------------------------------------------------
    // Core (Existing)
    // -----------------------------------------------------------------------
    pub fn new(primary_url: &str, secondary_url: &str) -> Self {
        let primary = ClientBuilder::default()
            .http(primary_url.parse().expect("invalid primary URL"));
        
        let secondary = ClientBuilder::default()
            .http(secondary_url.parse().expect("invalid secondary URL"));

        let quota = Quota::per_minute(nonzero!(200u32)).allow_burst(nonzero!(10u32));

        Self {
            primary,
            secondary,
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }

    pub async fn call_single(
        &self,
        method: &str,
        args: &Value,
    ) -> Result<Value, ConnectorError> {
        self.limiter.until_ready().await;
        let primary_res = Self::dispatch_single(&self.primary, method, args).await;
        if primary_res.is_ok() { return primary_res; }
        Self::dispatch_single(&self.secondary, method, args).await
    }

    async fn dispatch_single(
        client: &RpcClient,
        method: &str,
        args: &Value,
    ) -> Result<Value, ConnectorError> {
        client
            .request(method.to_string(), args.clone())
            .await
            .map_err(|e| ConnectorError::Rpc(e.to_string()))
    }

    pub async fn call_batch(&self, elems: &mut [BatchElem]) -> Result<(), ConnectorError> {
        for _ in elems.iter() { self.limiter.until_ready().await; }

        let primary_futs = elems.iter()
            .map(|e| Self::dispatch_single(&self.primary, &e.method, &e.args))
            .collect::<Vec<_>>();
        let primary_results = futures::future::join_all(primary_futs).await;

        let mut failed = Vec::new();
        for (i, res) in primary_results.into_iter().enumerate() {
            match res {
                Ok(val) => elems[i].result = Some(val),
                Err(_)  => failed.push(i),
            }
        }

        if !failed.is_empty() {
            for _ in failed.iter() { self.limiter.until_ready().await; }
            let secondary_futs = failed.iter()
                .map(|&i| Self::dispatch_single(&self.secondary, &elems[i].method, &elems[i].args))
                .collect::<Vec<_>>();
            let secondary_results = futures::future::join_all(secondary_futs).await;

            for (j, &i) in failed.iter().enumerate() {
                match &secondary_results[j] {
                    Ok(val)  => elems[i].result = Some(val.clone()),
                    Err(err) => elems[i].error = Some(err.to_string()),
                }
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // High-Level eth_call API
    // -----------------------------------------------------------------------

    /// Single eth_call (falls back to secondary automatically)
    pub async fn eth_call(
        &self,
        to: &str,
        calldata: &str,
        block: &str,
    ) -> Result<Value, ConnectorError> {
        let args = json!([{ "to": to, "data": calldata }, block]);
        self.call_single("eth_call", &args).await
    }

    /// Single eth_call with a `from` address
    pub async fn eth_call_from(
        &self,
        from: &str,
        to: &str,
        calldata: &str,
        block: &str,
    ) -> Result<Value, ConnectorError> {
        let args = json!([{ "from": from, "to": to, "data": calldata }, block]);
        self.call_single("eth_call", &args).await
    }

    /// Batch eth_call for multiple contracts/same or different calldata
    pub async fn eth_call_batch(
        &self,
        calls: &[(/* to */ &str, /* calldata */ &str)],
        block: &str,
    ) -> Vec<Result<Value, ConnectorError>> {
        
        // 1. Build BatchElems from the (to, calldata) tuples
        let mut elems: Vec<BatchElem> = calls
            .iter()
            .map(|(to, data)| BatchElem {
                method: "eth_call".into(),
                args: json!([{ "to": to, "data": data }, block]),
                result: None,
                error: None,
            })
            .collect();

        // 2. Dispatch using the existing batch mechanism
        // If the whole batch dispatcher fails (unlikely), map to errors
        if let Err(e) = self.call_batch(&mut elems).await {
            return vec![Err(e); elems.len()];
        }

        // 3. Convert BatchElem results back into a Vec<Result>
        elems
            .into_iter()
            .map(|e| match (e.result, e.error) {
                (Some(val), _) => Ok(val),
                (None, Some(err)) => Err(ConnectorError::Rpc(err)),
                // Should never happen if call_batch works correctly
                _ => Err(ConnectorError::Rpc("Unknown batch state".into())),
            })
            .collect()
    }
}