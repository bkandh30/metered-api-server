use crate::db::DbPool;
use crate::models::{EndpointUsage, PoolStats, StatusDistribution, SystemMetrics};
use std::convert::Infallible;
use warp::{Reply, reply};

pub async fn get_metrics(db: DbPool) -> Result<impl Reply, Infallible> {
    let total_requests = sqlx::query!("SELECT COUNT(*) as count FROM requests")
        .fetch_one(&*db)
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let key_stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(CASE WHEN is_active = true THEN 1 END) as active
        FROM api_keys;
        "#
    )
    .fetch_one(&*db)
    .await
    .ok();

    let avg_response = sqlx::query!(
        r#"
        SELECT
            AVG(response_time_ms)::DOUBLE PRECISION as avg
        FROM requests
        WHERE response_time_ms IS NOT NULL;
        "#
    )
    .fetch_one(&*db)
    .await
    .ok()
    .and_then(|r| r.avg.map(|v| v as f64));

    let requests_last_hour = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as count
        FROM requests
        WHERE created_at >= NOW() - INTERVAL '1 hour';
        "#
    )
    .fetch_one(&*db)
    .await
    .map(|r| r.count.unwrap_or(0))
    .unwrap_or(0);

    let requests_last_24h = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as count
        FROM requests
        WHERE created_at >= NOW() - INTERVAL '24 hours';
        "#
    )
    .fetch_one(&*db)
    .await
    .map(|r| r.count.unwrap_or(0))
    .unwrap_or(0);

    let top_endpoints = sqlx::query!(
        r#"
        SELECT endpoint,
            COUNT(*) as count
        FROM requests
        GROUP BY endpoint
        ORDER BY count DESC
        LIMIT 5
        "#
    )
    .fetch_all(&*db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| EndpointUsage {
        endpoint: r.endpoint,
        count: r.count.unwrap_or(0),
    })
    .collect();

    let status_dist = sqlx::query!(
        r#"
        SELECT
            COUNT(CASE WHEN status_code >= 200 AND status_code < 300 THEN 1 END) as success,
            COUNT(CASE WHEN status_code >= 400 AND status_code < 500 THEN 1 END) as client_error,
            COUNT(CASE WHEN status_code >= 500 THEN 1 END) as server_error
        FROM requests
        "#
    )
    .fetch_one(&*db)
    .await
    .map(|r| StatusDistribution {
        success_2xx: r.success.unwrap_or(0),
        client_error_4xx: r.client_error.unwrap_or(0),
        server_error_5xx: r.server_error.unwrap_or(0),
    })
    .unwrap_or(StatusDistribution {
        success_2xx: 0,
        client_error_4xx: 0,
        server_error_5xx: 0,
    });

    let pool_stats = PoolStats {
        size: db.size() as u32,
        num_idle: db.num_idle(),
    };

    let metrics = SystemMetrics {
        total_requests,
        total_api_keys: key_stats.as_ref().and_then(|k| k.total).unwrap_or(0),
        active_api_keys: key_stats.as_ref().and_then(|k| k.active).unwrap_or(0),
        avg_response_time_ms: avg_response,
        requests_last_hour,
        requests_last_24h,
        top_endpoints,
        status_distribution: status_dist,
        database_pool_stats: pool_stats,
    };

    Ok(reply::json(&metrics))
}
