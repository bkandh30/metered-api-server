use chrono::Utc;
use std::convert::Infallible;
use warp::{Reply, http::StatusCode, reply};

use crate::models::{ApiKey, ReadingData, ReadingRequest, ReadingResponse};

pub async fn submit_reading(
    api_key: ApiKey,
    reading: ReadingRequest,
) -> Result<impl Reply, Infallible> {
    tracing::info!(
        "Received reading from API key {}: sensor={}, value={}, unit={}",
        api_key.key,
        reading.sensor_id,
        reading.value,
        reading.unit
    );

    let response = ReadingResponse {
        status: "success".to_string(),
        message: "Reading recorded successfully".to_string(),
        timestamp: Utc::now(),
        data: ReadingData {
            sensor_id: reading.sensor_id,
            value: reading.value,
            unit: reading.unit,
        },
    };

    Ok(reply::with_status(reply::json(&response), StatusCode::OK))
}
