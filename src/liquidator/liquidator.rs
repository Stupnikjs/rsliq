async fn liquidator_loop(
    mut rx: mpsc::Receiver<Candidate>,
    connector: Arc<Connector>,
) {
    while let Some(candidate) = rx.recv().await {

        // simulation

        // liquidation

    }
}