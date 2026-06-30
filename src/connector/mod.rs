use alloy::network::Ethereum;
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::transports::{BoxTransport};
use alloy::rpc::types::{BlockNumberOrTag, Filter, Log, TransactionRequest};
use alloy::rpc::client::WsConnect;
use futures::StreamExt;
use alloy::consensus::{TxEip1559, SignableTransaction};
use alloy::network::TxSignerSync;
use alloy::primitives::{Address, Bytes, TxHash, U256};
use alloy::signers::local::PrivateKeySigner;

use crate::connector::nonce_gas::NonceManager;

mod nonce_gas;
mod rate_limiter;

pub struct Connector {
    pub http: RootProvider<Ethereum>,
    pub ws: Box<dyn Provider>,
    pub signer: PrivateKeySigner,
    pub nonce_mgr: NonceManager,
    pub base_fee: Arc<BaseFeeTracker>,
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
        let sub = self.ws.subscribe_logs(&filter).await?;
        let mut stream = sub.into_stream();
        while let Some(log) = stream.next().await {
          println!("{:?}", log); 
          on_log(log);
        
        }
        Ok(())
    }
     pub async fn send_tx(&self, to: Address, data: Bytes) -> Result<TxHash, Box<dyn std::error::Error>> {
        let from = self.signer.address();
        
        // nonce
        let nonce = self.http.get_transaction_count(from).await?;
        
        // gas
        let base_fee = self.http
            .get_block_by_number(BlockNumberOrTag::Latest)
            .await?
            .unwrap()
            .header
            .base_fee_per_gas
            .unwrap();
        
        let max_priority_fee = 1_000_000u128; // 0.001 gwei
        let max_fee = base_fee as u128 + max_priority_fee;

        // estimate gas
        let tx_req = TransactionRequest::default()
            .from(from)
            .to(to)
            .input(data.clone().into());
        let gas_limit = self.http.estimate_gas(tx_req).await?;

        // build tx
        let mut tx = TxEip1559 {
            chain_id: 8453, // Base
            nonce,
            max_fee_per_gas: max_fee,
            max_priority_fee_per_gas: max_priority_fee,
            gas_limit,
            to: alloy::primitives::TxKind::Call(to),
            value: U256::ZERO,
            input: data,
            access_list: Default::default(),
        };

        // sign
        let sig = self.signer.sign_transaction_sync(&mut tx)?;
        let signed = tx.into_signed(sig);
        
        // encode + send
        let mut buf = vec![];
        signed.eip2718_encode(&mut buf);
        let pending = self.http.send_raw_transaction(&buf).await?;
        let receipt = pending.get_receipt().await?;
        
        Ok(receipt.transaction_hash)
    }
}



pub async fn build(http_url: &str, ws_url: &str, signer: PrivateKeySigner) -> Result<Connector, Box<dyn std::error::Error>> {
    let http = RootProvider::<Ethereum>::new_http(http_url.parse()?);
    let ws = Box::new(
        ProviderBuilder::new()
            .disable_recommended_fillers()
            .connect_ws(WsConnect::new(ws_url))
            .await?
    );
     let nonce_mgr = NonceManager::init(&http, signer.address()).await?;
        let base_fee = nonce_gas::BaseFeeTracker::new();
        base_fee.spawn_updater(ws.clone());
    Ok(Connector { http, ws, signer, nonce_mgr, base_fee })
}
