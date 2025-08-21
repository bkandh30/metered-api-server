use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ReadingRequest {
    pub sensor_id: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Serialize)]
pub struct ReadingResponse {
    pub status: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub data: ReadingData,
}

#[derive(Debug, Serialize)]
pub struct ReadingData {
    pub sensor_id: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Reading {
    pub id: Uuid,
    pub api_key_id: Uuid,
    pub sensor_id: String,
    pub value: f64,
    pub unit: String,
    pub created_at: DateTime<Utc>,
}
