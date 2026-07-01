use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use alloy::consensus::{SignableTransaction, TxEip1559};
use alloy::network::TxSignerSync;
use alloy::primitives::{Address, Bytes, TxHash, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{BlockNumberOrTag, TransactionRequest};
use alloy::signers::local::PrivateKeySigner;
use futures::StreamExt;

pub struct TxSender {
    signer: PrivateKeySigner,
    next_nonce: AtomicU64,
    base_fee: RwLock<u128>,
    chain_id: u64,
}

impl TxSender {
    pub async fn init<P: Provider>(
        http: &P,
        signer: PrivateKeySigner,
        chain_id: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let nonce = http.get_transaction_count(signer.address()).await?;
        Ok(Self {
            signer,
            next_nonce: AtomicU64::new(nonce),
            base_fee: RwLock::new(0),
            chain_id,
        })
    }

    pub fn address(&self) -> Address {
        self.signer.address()
    }

    fn next_nonce(&self) -> u64 {
        self.next_nonce.fetch_add(1, Ordering::SeqCst)
    }

    fn release_nonce(&self, nonce: u64) {
        let _ = self.next_nonce.compare_exchange(
            nonce + 1,
            nonce,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
    }

    fn base_fee(&self) -> u128 {
        *self.base_fee.read().unwrap()
    }

    fn set_base_fee(&self, v: u128) {
        *self.base_fee.write().unwrap() = v;
    }

    /// Spawns a background task keeping base_fee up to date via block subscription.
    pub fn spawn_base_fee_updater<P>(self: &Arc<Self>, ws: Arc<P>)
    where
        P: Provider + Send + Sync + 'static,
    {
        let sender = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                match ws.subscribe_blocks().await {
                    Ok(sub) => {
                        let mut stream = sub.into_stream();
                        while let Some(header) = stream.next().await {
                            if let Some(fee) = header.base_fee_per_gas {
                                sender.set_base_fee(fee as u128);
                            }
                        }
                        eprintln!("base fee subscription ended, reconnecting");
                    }
                    Err(e) => {
                        eprintln!("subscribe_blocks failed: {e}, retrying in 2s");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
        });
    }

    pub async fn send_tx<P: Provider>(
        &self,
        http: &P,
        to: Address,
        data: Bytes,
    ) -> Result<TxHash, Box<dyn std::error::Error>> {
        let from = self.address();
        let nonce = self.next_nonce();

        let mut base_fee = self.base_fee();
        if base_fee == 0 {
            // fallback if the updater hasn't ticked yet (e.g. right at startup)
            base_fee = http
                .get_block_by_number(BlockNumberOrTag::Latest)
                .await?
                .ok_or("no latest block")?
                .header
                .base_fee_per_gas
                .ok_or("no base fee")? as u128;
        }

        let max_priority_fee = 1_000_000u128;
        let max_fee = base_fee + max_priority_fee;

        let tx_req = TransactionRequest::default()
            .from(from)
            .to(to)
            .input(data.clone().into());

        let gas_limit = match http.estimate_gas(tx_req).await {
            Ok(g) => g,
            Err(e) => {
                self.release_nonce(nonce);
                return Err(e.into());
            }
        };

        let mut tx = TxEip1559 {
            chain_id: self.chain_id,
            nonce,
            max_fee_per_gas: max_fee,
            max_priority_fee_per_gas: max_priority_fee,
            gas_limit,
            to: alloy::primitives::TxKind::Call(to),
            value: U256::ZERO,
            input: data,
            access_list: Default::default(),
        };

        let sig = self.signer.sign_transaction_sync(&mut tx)?;
        let signed = tx.into_signed(sig);

        let mut buf = vec![];
        signed.eip2718_encode(&mut buf);
        let pending = http.send_raw_transaction(&buf).await?;
        let receipt = pending.get_receipt().await?;

        Ok(receipt.transaction_hash)
    }
}