use crate::db::DbPool;
use crate::models::{DailyUsage, MonthlyReport, UsageStats};
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use serde::Deserialize;
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

// Helper structs for optional `?format=csv` query parameters
#[derive(Deserialize)]
pub struct ReportParams {
    pub format: Option<String>,
}

// Helper struct for result of database query
#[derive(sqlx::FromRow)]
struct DailyCount {
    date: NaiveDate,
    requests: i64,
}

pub async fn get_monthly_report(
    key: String,
    params: ReportParams,
    db: DbPool,
) -> Result<impl Reply, Infallible> {
    let api_key = match sqlx::query!("SELECT id, name FROM api_keys WHERE key = $1", key)
        .fetch_optional(&*db)
        .await
    {
        Ok(Some(record)) => record,
        Ok(None) => {
            let err = reply::json(&serde_json::json!({
                "error": "API key not found"
            }));
            return Ok(reply::with_status(err, StatusCode::NOT_FOUND).into_response());
        }
        Err(e) => {
            tracing::error!("DB error validating key for report: {:?}", e);
            let err = reply::json(&serde_json::json!({
                "error": "Failed to generate report"
            }));
            return Ok(reply::with_status(err, StatusCode::INTERNAL_SERVER_ERROR).into_response());
        }
    };

    let now = Utc::now();

    let start_of_month = Utc::with_ymd_and_hms(&Utc, now.year(), now.month(), 1, 0, 0, 0).unwrap();

    let daily_counts = match sqlx::query_as::<_, DailyCount>(
        r#"
        SELECT
            DATE(created_at) as date,
            COUNT(id) as requests
        FROM requests
        WHERE api_key_id = $1 AND created_at >= $2
        GROUP BY DATE(created_at)
        ORDER BY date
        "#,
    )
    .bind(api_key.id)
    .bind(start_of_month)
    .fetch_all(&*db)
    .await
    {
        Ok(counts) => counts,
        Err(e) => {
            tracing::error!("Failed to get monthly report data: {:?}", e);
            let err = reply::json(&serde_json::json!({
                "error": "Failed to generate report"
            }));
            return Ok(reply::with_status(err, StatusCode::INTERNAL_SERVER_ERROR).into_response());
        }
    };

    let daily_breakdown: Vec<DailyUsage> = daily_counts
        .into_iter()
        .map(|dc| DailyUsage {
            date: dc.date.to_string(),
            requests: dc.requests,
        })
        .collect();

    let report = MonthlyReport {
        api_key_name: api_key.name,
        month: format!("{:02}", now.month()),
        year: now.year(),
        total_requests: daily_breakdown.iter().map(|d| d.requests).sum(),
        daily_breakdown,
    };

    let response = if params.format.as_deref() == Some("csv") {
        let mut csv = "Date, Requests\n".to_string();

        for daily in &report.daily_breakdown {
            csv.push_str(&format!("{},{}\n", daily.date, daily.requests));
        }

        let csv_reply = reply::with_header(csv, "Content-Type", "text/csv");

        reply::with_header(csv_reply, "Content-Type", "text/csv").into_response()
    } else {
        let json_reply = reply::json(&report);

        reply::with_status(json_reply, StatusCode::OK).into_response()
    };

    Ok(response)
}
