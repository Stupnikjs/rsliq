use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{interval, Duration};

#[derive(Clone)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    max_tokens: usize,
}

impl RateLimiter {
    /// `max_rps` = requêtes max par seconde
    pub fn new(max_rps: usize) -> Self {
        let semaphore = Arc::new(Semaphore::new(max_rps));
        let sem_clone = semaphore.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));
            loop {
                ticker.tick().await;
                let current = sem_clone.available_permits();
                let to_add = max_rps.saturating_sub(current);
                sem_clone.add_permits(to_add);
            }
        });

        Self {
            semaphore,
            max_tokens: max_rps,
        }
    }

    pub async fn acquire(&self) {
        self.semaphore
            .acquire()
            .await
            .expect("semaphore closed")
            .forget(); // consume le permit sans le rendre
    }
}