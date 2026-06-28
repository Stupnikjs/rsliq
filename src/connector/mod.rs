use alloy::network::Ethereum;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::transports::{BoxTransport};
use alloy::rpc::types::{Filter, BlockNumberOrTag, Log};
use alloy::rpc::client::WsConnect;
use alloy::primitives::{Address, Bytes};
use alloy::providers::Provider;
use futures::StreamExt;

pub struct Connector {
    pub http: RootProvider<Ethereum>,
    pub ws: RootProvider<Ethereum>,
}

impl Connector {
    pub async fn call_raw(&self, to: Address, data: Bytes) -> Result<Bytes, Box<dyn std::error::Error>> {
        let tx = alloy::rpc::types::TransactionRequest::default()
            .to(to)
            .input(data.into());
        Ok(self.http.call(tx).await?)
    }

    pub async fn subscribe<F>(&self, morpho_addr: Address, mut on_log: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Log),
    {
        let filter = Filter::new()
            .address(morpho_addr)
            .from_block(BlockNumberOrTag::Latest)
            .events([
                "Supply(bytes32,address,address,uint256,uint256)",
                "Borrow(bytes32,address,address,address,uint256,uint256)",
                "Repay(bytes32,address,address,uint256,uint256)",
                "Liquidate(bytes32,address,address,uint256,uint256,uint256,uint256,uint256)",
                "AccrueInterest(bytes32,uint256,uint256,uint256)",
            ]);
        let sub = self.ws.watch_logs(&filter).await?;
        let mut stream = sub.into_stream();
        while let Some(log) = stream.next().await {
            for l in log {
                on_log(l);
            }
        }
        Ok(())
    }
}

pub async fn build(http_url: &str, ws_url: &str) -> Result<Connector, Box<dyn std::error::Error>> {
    let http = RootProvider::<Ethereum>::new_http(http_url.parse()?);
    let ws = ProviderBuilder::new()
        .disable_recommended_fillers()
        .connect_ws(WsConnect::new(ws_url))
        .await?;
    Ok(Connector { http, ws })
}