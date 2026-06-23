use alloy::{
    primitives::{keccak256, Address, U256},
    rpc::types::{Filter, BlockNumberOrTag},
    providers::Provider,
    pubsub::PubSubFrontend,
};
use futures::StreamExt;

const MORPHO_ADDRESS: Address = address!("BBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb");

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

    pub async fn subscribe_morpho_events(
        &self,
        cache: &mut Cache,
    ) -> Result<(), Box<dyn std::error::Error>>
    {
        let filter = Filter::new()
            .address(MORPHO_ADDRESS)
            .from_block(BlockNumberOrTag::Latest);

        let sub = self.provider.subscribe_logs(&filter).await?;
        let mut stream = sub.into_stream();

        while let Some(log) = stream.next().await {
            let topic0 = log.topics()[0];

            match topic0 {
                x if x == keccak256("Supply(bytes32,address,address,uint256,uint256)") => {
                    let on_behalf = Address::from_slice(&log.topics()[2].as_slice()[12..]);
                    let assets = U256::from_be_slice(&log.data().data);
                    cache.update(on_behalf, assets);
                }
                x if x == keccak256("Borrow(bytes32,address,address,address,uint256,uint256)") => {
                    let on_behalf = Address::from_slice(&log.topics()[2].as_slice()[12..]);
                    let assets = U256::from_be_slice(&log.data().data);
                    cache.update(on_behalf, assets);
                }
                _ => {}
            }
        }

        Ok(())
    }
}