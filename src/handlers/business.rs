use chrono::Utc;
use std::convert::Infallible;
use uuid::Uuid;
use warp::{Reply, http::StatusCode, reply};

use crate::{
    db::DbPool,
    models::{ApiKey, Reading, ReadingData, ReadingRequest, ReadingResponse},
};

pub async fn submit_reading(
    api_key: ApiKey,
    db: DbPool,
    reading: ReadingRequest,
) -> Result<impl Reply, Infallible> {
    tracing::info!(
        "Received reading from API key {}: sensor={}, value={}, unit={}",
        api_key.key,
        reading.sensor_id,
        reading.value,
        reading.unit
    );

    let result = sqlx::query(
        r#"
        INSERT INTO readings (id, api_key_id, sensor_id, value, unit)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(api_key.id)
    .bind(&reading.sensor_id)
    .bind(reading.value)
    .bind(&reading.unit)
    .execute(&*db)
    .await;

    match result {
        Ok(_) => {
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

            Ok(reply::with_status(
                reply::json(&response),
                StatusCode::CREATED,
            ))
        }
        Err(e) => {
            tracing::error!("Failed to save reading: {:?}", e);

            let err_response =
                reply::json(&serde_json::json!({"error": "Failed to save the reading"}));

            Ok(reply::with_status(
                err_response,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn get_readings(api_key: ApiKey, db: DbPool) -> Result<impl Reply, Infallible> {
    tracing::info!("Fetching readings for API key ID: {}", api_key.id);

    let result = sqlx::query_as::<_, Reading>(
        r#"
        SELECT * FROM readings
        WHERE api_key_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(api_key.id)
    .fetch_all(&*db)
    .await;

    match result {
        Ok(readings) => {
            let response = serde_json::json!({
                "status": "success",
                "count": readings.len(),
                "readings": readings
            });

            Ok(reply::with_status(reply::json(&response), StatusCode::OK))
        }
        Err(e) => {
            tracing::error!("Failed to fetch readings: {:?}", e);

            let err_response =
                reply::json(&serde_json::json!({"error": "Failed to fetch readings"}));

            Ok(reply::with_status(
                err_response,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
