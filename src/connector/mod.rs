use alloy::providers::{Provider, ProviderBuilder};
use alloy::pubsub::PubSubFrontend;
use alloy::transports::ws::WsConnect;
use alloy::rpc::types::{Filter, BlockNumberOrTag, Log};
use alloy::primitives::{Address, Bytes, keccak256, U256};
use futures::StreamExt;

pub struct Connector<H, W> {
    pub http: H,
    pub ws: W,
}

impl<H: Provider, W: Provider<PubSubFrontend>> Connector<H, W> {
    pub fn new(http: H, ws: W) -> Self {
        Self { http, ws }
    }

    // HTTP — eth_call brut
    pub async fn call_raw(
        &self,
        to: Address,
        data: Bytes,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        let tx = alloy::rpc::types::TransactionRequest::default()
            .to(to)
            .input(data.into());

        let result = self.http.call(&tx).await?;
        Ok(result)
    }

    // WS — subscribe aux logs
    pub async fn subscribe_logs<F>(
        &self,
        filter: Filter,
        mut on_log: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Log),
    {
        let sub = self.ws.subscribe_logs(&filter).await?;
        let mut stream = sub.into_stream();

        while let Some(log) = stream.next().await {
            on_log(log);
        }

        Ok(())
    }

    // WS — subscribe aux blocs
    pub async fn subscribe_blocks<F>(
        &self,
        mut on_block: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(alloy::rpc::types::Header),
    {
        let sub = self.ws.subscribe_blocks().await?;
        let mut stream = sub.into_stream();

        while let Some(block) = stream.next().await {
            on_block(block);
        }

        Ok(())
    }
}

pub async fn build(
    http_url: &str,
    ws_url: &str,
) -> Result<
    Connector<impl Provider, impl Provider<PubSubFrontend>>,
    Box<dyn std::error::Error>,
> {
    let http = ProviderBuilder::new()
        .on_http(http_url.parse()?);

    let ws = ProviderBuilder::new()
        .on_ws(WsConnect::new(ws_url))
        .await?;

    Ok(Connector::new(http, ws))
}