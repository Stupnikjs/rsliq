use alloy::{
    network::Ethereum,
    primitives::{Address, Bytes},
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    rpc::types::{Filter, Log, TransactionRequest},
    transports::pubsub::PubSubFrontend,
    network::TransactionBuilder,
};
use futures_util::Stream;

pub struct HttpCaller(RootProvider<BoxTransport, Ethereum>);
pub struct WsSubscriber(RootProvider<PubSubFrontend>);

pub async fn new_http(rpc_url: &str) -> Result<HttpCaller, Box<dyn std::error::Error>> {
    let p = ProviderBuilder::new().connect_http(rpc_url.parse()?);
    Ok(HttpCaller(p))
}

pub async fn new_ws(ws_url: &str) -> Result<WsSubscriber, Box<dyn std::error::Error>> {
    let p = ProviderBuilder::new()
        .disable_recommended_fillers()
        .connect_ws(WsConnect::new(ws_url))
        .await?;
    Ok(WsSubscriber(p))
}

impl HttpCaller {
    pub async fn call(&self, to: Address, data: Bytes) -> Result<Bytes, Box<dyn std::error::Error>> {
        let tx = TransactionRequest::default().with_to(to).with_input(data);
        Ok(self.0.call(&tx).await?)
    }
}

impl WsSubscriber {
    pub async fn subscribe_logs(
        &self,
        filter: Filter,
    ) -> Result<impl Stream<Item = Log>, Box<dyn std::error::Error>> {
        let sub = self.0.subscribe_logs(&filter).await?;
        Ok(sub.into_stream())
    }
}