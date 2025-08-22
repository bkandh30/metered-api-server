use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::uuid;
use warp::{Rejection, reject};

#[derive(Debug)]
pub struct RateLimitExceeded;
impl reject::Reject for RateLimitExceeded {}

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<Uuid, Vec<DateTime<Utc>>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, api_key_id: Uuid, limit: i32) -> Result<(), Rejection> {
        let now = Utc::now();
        let window_start = now - Duration::minutes(1);

        let mut requests = self.requests.write().await;
        let timestamps = requests.entry(api_key_id).or_insert_with(Vec::new);

        timestamps.retain(|&t| t > window_start);

        if timestamps.len() >= limit as usize {
            return Err(reject::custom(RateLimitExceeded));
        }

        timestamps.push(now);
        Ok(())
    }
}
