use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Serialize, FromRow)]
pub struct Request {
    pub id: Uuid,
    pub api_key_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UsageStats {
    pub api_key_name: String,
    pub total_requests: i64,
    pub requests_today: i64,
    pub requests_this_month: i64,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct MonthlyReport {
    pub api_key_name: String,
    pub month: String,
    pub year: i32,
    pub total_requests: i64,
    pub daily_breakdown: Vec<DailyUsage>,
}

#[derive(Debug, Serialize)]
pub struct DailyUsage {
    pub date: String,
    pub requests: i64,
}
