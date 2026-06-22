// ethcaller.rs
use alloy::{
    primitives::{Address, Bytes},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    transports::{RpcError, TransportErrorKind},
};
use futures_util::StreamExt;

use crate::connector::rate_limiter::RateLimiter;

pub struct EthCaller<P> {
    provider: P,
    rate_limiter: RateLimiter,
}

pub fn new_caller(rpc_url: String) -> Result<EthCaller<impl Provider>, Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
    Ok(EthCaller { provider, rate_limiter: RateLimiter::new(200) })
}

impl<P: Provider> EthCaller<P> {
    pub async fn call_raw(&self, to: Address, data: Bytes) -> Result<Bytes, RpcError<TransportErrorKind>> {
        self.rate_limiter.acquire().await;
        let tx = TransactionRequest::default().with_to(to).with_input(data);
        self.provider.call(tx).await.map_err(|e| e.into())
    }
}


// wssubscriber.rs
use alloy::{
    providers::{Provider, ProviderBuilder, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::Header,
};


pub struct WsSubscriber<P> {
    provider: P,
}

pub async fn new_subscriber(ws_url: String) -> Result<WsSubscriber<impl Provider<PubSubFrontend>>, Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new().connect_ws(WsConnect::new(ws_url)).await?;
    Ok(WsSubscriber { provider })
}

impl<P: Provider<PubSubFrontend>> WsSubscriber<P> {
    pub async fn subscribe_blocks<F>(&self, mut on_block: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Header),
    {
        let sub = self.provider.subscribe_blocks().await?;
        let mut stream = sub.into_stream();
        while let Some(block) = stream.next().await {
            on_block(block);
        }
        Ok(())
    }
}