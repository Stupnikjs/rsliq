mod rate_limiter;
use std::sync::Arc;
use alloy::{
    network::{Ethereum, TransactionBuilder}, primitives::{Address, Bytes}, providers::{Provider, ProviderBuilder, RootProvider, WsConnect}, rpc::types::{Filter, Log, TransactionRequest}, transports::{BoxTransport, RpcError, TransportErrorKind},
};

use futures_util::{Stream, StreamExt};
use crate::connector::rate_limiter::RateLimiter;

pub struct Connector {
    http: Arc<dyn Provider>,
    ws: RootProvider<Ethereum>,
    rate_limiter: RateLimiter,
}

pub async fn new(
    rpc_url: String,
    ws_url: String,
) -> Result<Connector, Box<dyn std::error::Error>> {
    let http = Arc::new(ProviderBuilder::new().connect_http(rpc_url.parse()?));
    let ws = ProviderBuilder::new()
        .disable_recommended_fillers()
        .connect_ws(WsConnect::new(ws_url))
        .await?; 
    Ok(Connector { http, ws, rate_limiter: RateLimiter::new(200) })
}

impl Connector {
    pub async fn call_raw(
        &self,
        to: Address,
        data: Bytes,
    ) -> Result<Bytes, RpcError<TransportErrorKind>> {
        self.rate_limiter.acquire().await;
        let tx = TransactionRequest::default().with_to(to).with_input(data);
        self.http.call(tx).await.map_err(|e| e.into())
    }

    pub async fn subscribe_logs(
        &self,
        filter: Filter,
    ) -> Result<impl Stream<Item = Log>, Box<dyn std::error::Error>> {
        let sub = self.ws.subscribe_logs(&filter.clone()).await?;
        Ok(sub.into_stream())
    }
}