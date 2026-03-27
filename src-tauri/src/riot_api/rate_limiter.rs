use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Dual token bucket rate limiter for the Riot API.
/// Enforces both short-window and long-window rate limits.
pub struct RateLimiter {
    short: Mutex<TokenBucket>,
    long: Mutex<TokenBucket>,
}

struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, window_secs: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate: max_tokens / window_secs,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    fn try_acquire(&mut self) -> Option<Duration> {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            None // No wait needed
        } else {
            // How long until a token is available
            let wait = (1.0 - self.tokens) / self.refill_rate;
            Some(Duration::from_secs_f64(wait))
        }
    }
}

impl RateLimiter {
    /// Create a rate limiter with development key limits
    pub fn dev() -> Self {
        Self {
            // 20 requests per 1 second
            short: Mutex::new(TokenBucket::new(20.0, 1.0)),
            // 100 requests per 2 minutes
            long: Mutex::new(TokenBucket::new(100.0, 120.0)),
        }
    }

    /// Create a rate limiter with production key limits
    #[allow(dead_code)]
    pub fn production() -> Self {
        Self {
            // 500 requests per 10 seconds
            short: Mutex::new(TokenBucket::new(500.0, 10.0)),
            // 30000 requests per 10 minutes
            long: Mutex::new(TokenBucket::new(30000.0, 600.0)),
        }
    }

    /// Acquire a token, waiting if necessary.
    /// Returns when a request is allowed.
    pub async fn acquire(&self) {
        loop {
            let wait = {
                let short_wait = self.short.lock().unwrap().try_acquire();
                if let Some(w) = short_wait {
                    Some(w)
                } else {
                    self.long.lock().unwrap().try_acquire()
                }
            };

            match wait {
                None => return, // Both buckets had tokens
                Some(duration) => {
                    tokio::time::sleep(duration).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_immediate() {
        let mut bucket = TokenBucket::new(5.0, 1.0);
        // Should get 5 tokens immediately
        for _ in 0..5 {
            assert!(bucket.try_acquire().is_none());
        }
        // 6th should need to wait
        assert!(bucket.try_acquire().is_some());
    }

    #[test]
    fn test_rate_limiter_dev_limits() {
        let limiter = RateLimiter::dev();
        // Should allow 20 immediate requests
        for _ in 0..20 {
            let wait = limiter.short.lock().unwrap().try_acquire();
            assert!(wait.is_none());
        }
        // 21st should require waiting
        let wait = limiter.short.lock().unwrap().try_acquire();
        assert!(wait.is_some());
    }
}
