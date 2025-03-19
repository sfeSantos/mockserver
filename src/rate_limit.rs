use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use serde::Deserialize;
use tokio::sync::Mutex;
use warp::reject::Reject;

pub type RateLimitTracker = Arc<Mutex<HashMap<String, (Instant, usize)>>>;

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimit {
    pub requests: usize, // Allowed requests per window
    pub window_ms: u64,  // Time window in milliseconds
}

#[derive(Debug)]
pub struct RateLimited;
impl Reject for RateLimited {}

/// Initialize the rate limit tracker
pub fn new_rate_limit() -> RateLimitTracker {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn check_rate_limit(
    path: String,
    rate_limit: Option<&RateLimit>,
    rate_limiter: RateLimitTracker,
) -> Result<(), warp::Rejection> {
    if let Some(limit) = rate_limit {
        let mut rate_tracker = rate_limiter.lock().await;
        let now = Instant::now();
        let (start_time, count) = rate_tracker.entry(path.clone()).or_insert((now, 0));

        if now.duration_since(*start_time).as_millis() as u64 > limit.window_ms {
            *start_time = now;
            *count = 1;
        } else if *count > limit.requests {
            return Err(warp::reject::custom(RateLimited))
        } else {
            *count += 1;
        }
    }
    
    Ok(())
}