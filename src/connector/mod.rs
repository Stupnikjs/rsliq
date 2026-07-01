use std::sync::Arc;
use alloy::network::Ethereum;
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::rpc::client::WsConnect;
use alloy::rpc::types::{BlockNumberOrTag, Filter, Log, TransactionRequest};
use alloy::signers::local::PrivateKeySigner;
use alloy::primitives::{Address, Bytes, TxHash};
use futures::StreamExt;
use tx_sender::TxSender;

mod tx_sender;
mod rate_limiter;

pub struct Connector {
    pub http: RootProvider<Ethereum>,
    pub ws: Arc<RootProvider<Ethereum>>,
    pub tx_sender: Arc<TxSender>,
    pub rate_limiter: rate_limiter::RateLimiter,
}

impl Connector {
    pub async fn call_raw(&self, to: Address, data: Bytes) -> Result<Bytes, Box<dyn std::error::Error>> {
        self.rate_limiter.acquire().await;
        let tx = TransactionRequest::default().to(to).input(data.into());
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
        let sub = self.ws.subscribe_logs(&filter).await?;
        let mut stream = sub.into_stream();
        while let Some(log) = stream.next().await {
            on_log(log);
        }
        Ok(())
    }

    pub async fn send_tx(&self, to: Address, data: Bytes) -> Result<TxHash, Box<dyn std::error::Error>> {
        self.tx_sender.send_tx(&self.http, to, data).await
    }
}

pub async fn build(
    http_url: &str,
    ws_url: &str,
    signer: PrivateKeySigner,
    chain_id: u64,
    max_rps: usize,
) -> Result<Connector, Box<dyn std::error::Error>> {
    let http = RootProvider::<Ethereum>::new_http(http_url.parse()?);

    let ws = Arc::new(
        ProviderBuilder::new()
            .disable_recommended_fillers()
            .connect_ws(WsConnect::new(ws_url))
            .await?,
    );

    let rate_limiter = rate_limiter::RateLimiter::new(max_rps);
    let tx_sender = Arc::new(TxSender::init(&http, signer, chain_id).await?);
    tx_sender.spawn_base_fee_updater(Arc::clone(&ws));

    Ok(Connector { http, ws, tx_sender, rate_limiter })
}