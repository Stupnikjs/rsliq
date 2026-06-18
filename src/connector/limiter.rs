use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Instant, Duration};

#[derive(Clone)]
pub struct SimpleRateLimiter {
    inner: Arc<Mutex<State>>,
}

struct State {
    last: Instant,
    interval: Duration,
}

impl SimpleRateLimiter {
    pub fn new(rps: u32) -> Self {
        let interval = Duration::from_millis(1000 / rps as u64);

        Self {
            inner: Arc::new(Mutex::new(State {
                last: Instant::now() - interval,
                interval,
            })),
        }
    }

    pub async fn acquire(&self) {
        let wait = {
            let mut state = self.inner.lock().unwrap();

            let now = Instant::now();
            let elapsed = now.duration_since(state.last);

            if elapsed >= state.interval {
                state.last = now;
                None
            } else {
                let wait = state.interval - elapsed;
                state.last = now + wait;
                Some(wait)
            }
        };

        if let Some(w) = wait {
            sleep(w).await;
        }
    }
}