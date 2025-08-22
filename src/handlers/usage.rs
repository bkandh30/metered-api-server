use crate::db::DbPool;
use crate::models::{DailyUsage, MonthlyReport, UsageStats};
use chrono::{Datelike, Utc};
use std::convert::Infallible;
use warp::{Reply, http::StatusCode, reply};

pub async fn get_usage_stats(key: String, db: DbPool) -> Result<impl Reply, Infallible> {
    let stats_query = sqlx::query!(
        r#"
        SELECT
            ak.name,
            COUNT(r.id) as "total_requests!",
            COUNT(CASE WHEN DATE(r.created_at) = CURRENT_DATE THEN 1 END) as "requests_today!",
            COUNT(CASE WHEN DATE_TRUNC('month', r.created_at) = DATE_TRUNC('month', CURRENT_DATE) THEN 1 END) as "requests_this_month!",
            MAX(r.created_at) as last_used
        FROM api_keys ak
        LEFT JOIN requests r ON ak.id = r.api_key_id
        WHERE ak.key = $1
        GROUP BY ak.id, ak.name
        "#,
        key
    )
    .fetch_optional(&*db)
    .await;

    match stats_query {
        Ok(Some(record)) => {
            let stats = UsageStats {
                api_key_name: record.name,
                total_requests: record.total_requests,
                requests_today: record.requests_today,
                requests_this_month: record.requests_this_month,
                last_used: record.last_used,
            };

            Ok(reply::with_status(reply::json(&stats), StatusCode::OK))
        }

        Ok(None) => Ok(reply::with_status(
            reply::json(&serde_json::json!({
                "error": "API key not found"
            })),
            StatusCode::NOT_FOUND,
        )),

        Err(e) => {
            tracing::error!("Failed to get usage stats: {:?}", e);
            Ok(reply::with_status(
                reply::json(&serde_json::json!({
                    "error": "Failed to retrieve usage statistics"
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
