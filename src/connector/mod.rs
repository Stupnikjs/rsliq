pub struct Connector<P> {
    provider: P,
}

impl<P: Provider> Connector<P> {
    pub async fn call_raw(
        &self,
        to: Address,
        data: Bytes,
    ) -> Result<Bytes> {
        let tx = TransactionRequest::default()
            .with_to(to)
            .with_input(data);

        self.provider.call(tx).await
    }
}