use std::sync::Arc;
use std::sync::atomic::{AtomicU64, At Ordering};

use alloy::providers::Provider;
use alloy_primitives::Address;
use futures_util::StreamExt;

/// Tracks the next nonce to use locally, avoiding a get_transaction_count
/// round-trip on every tx send.
pub struct NonceManager {
    next: AtomicU64,
}

impl NonceManager {
    /// Initialize once at startup with the current pending nonce.
    pub async fn init<P: Provider>(provider: &P, address: Address) -> anyhow::Result<Self> {
        let current = provider.get_transaction_count(address).pending().await?;
        Ok(Self { next: AtomicU64::new(current) })
    }

    /// Reserve and return the next nonce, incrementing atomically.
    pub fn next(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }

    /// Call this if a tx fails before being broadcast (e.g. signing error)
    /// to give the nonce back, avoiding a gap.
    pub fn release(&self, nonce: u64) {
        // Only safe to roll back if nonce == next-1 (last reserved).
        let _ = self.next.compare_exchange(
            nonce + 1,
            nonce,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
    }

    /// Resync from chain if you suspect drift (e.g. after a dropped tx
    /// or external wallet activity).
    pub async fn resync<P: Provider>(&self, provider: &P, address: Address) -> anyhow::Result<()> {
        let current = provider.get_transaction_count(address).pending().await?;
        self.next.store(current, Ordering::SeqCst);
        Ok(())
    }
}

/// Tracks the latest base fee, updated in the background via a block
/// subscription instead of being fetched on every tx.
pub struct BaseFeeTracker {
    base_fee: AtomicU128,
}

impl BaseFeeTracker {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { base_fee: AtomicU128::new(0) })
    }

    pub fn get(&self) -> u128 {
        self.base_fee.load(Ordering::Relaxed)
    }

    fn set(&self, value: u128) {
        self.base_fee.store(value, Ordering::Relaxed);
    }

    /// Spawns a background task subscribing to new block headers and
    /// keeping base_fee up to date. Call once at startup.
    pub fn spawn_updater<P: Provider + Clone + Send + Sync + 'static>(
        self: &Arc<Self>,
        ws_provider: P,
    ) {
        let tracker = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                match ws_provider.subscribe_blocks().await {
                    Ok(sub) => {
                        let mut stream = sub.into_stream();
                        while let Some(header) = stream.next().await {
                            if let Some(fee) = header.base_fee_per_gas {
                                tracker.set(fee as u128);
                            }
                        }
                        eprintln!("base fee subscription stream ended, reconnecting");
                    }
                    Err(e) => {
                        eprintln!("subscribe_blocks failed: {e}, retrying in 2s");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
        });
    }
}