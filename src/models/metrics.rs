use crate::db::DbPool;
use serde::Serialize;
use std::convert::Infalliable;
use warp::{Reply, reply};

#[derive(Serialize)]
pub struct SystemMetrics {
    pub total_requests: i64,
    pub total_api_keys: i64,
    pub active_api_keys: i64,
    pub avg_response_time_ms: Option<f64>,
    pub requests_last_hour: i64,
    pub requests_last_24h: i64,
    pub top_endpoints: Vec<EndpointUsage>,
    pub status_distribution: StatusDistribution,
    pub database_pool_stats: PoolStats,
}

#[derive(Serialize)]
pub struct EndpointUsage {
    pub endpoint: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct StatusDistribution {
    pub success_2xx: i64,
    pub client_error_4xx: i64,
    pub server_error_5xx: i64,
}

#[derive(Serialize)]
pub struct PoolStats {
    pub size: u32,
    pub num_idle: usize,
}
