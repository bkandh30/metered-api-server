mod db;
mod handlers;
mod middleware;
mod models;

use crate::middleware::rate_limiter::RateLimiter;
use anyhow::Result;
use std::env;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid u16");

    tracing::info!("Connecting to database...");
    let db_pool = db::create_pool(&database_url).await?;

    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&*db_pool).await?;

    // Rate Limiter Instance
    let rate_limiter = RateLimiter::new();

    // health route
    let health = warp::path("health").map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "healthy"
        }))
    });

    // Admin routes
    let admin_routes = {
        let create_key = warp::path!("admin" / "keys")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(handlers::admin::create_api_key);

        let list_keys = warp::path!("admin" / "keys")
            .and(warp::get())
            .and(with_db(db_pool.clone()))
            .and_then(handlers::admin::list_api_keys);

        let delete_key = warp::path!("admin" / "keys" / String)
            .and(warp::delete())
            .and(with_db(db_pool.clone()))
            .and_then(handlers::admin::delete_api_key);

        let get_stats = warp::path!("admin" / "keys" / String / "stats")
            .and(warp::get())
            .and(with_db(db_pool.clone()))
            .and_then(handlers::usage::get_usage_stats);

        let get_report = warp::path!("admin" / "keys" / String / "report")
            .and(warp::get())
            .and(warp::query::<handlers::usage::ReportParams>())
            .and(with_db(db_pool.clone()))
            .and_then(handlers::usage::get_monthly_report);

        create_key
            .or(list_keys)
            .or(delete_key)
            .or(get_stats)
            .or(get_report)
    };

    // Protected business routes
    let protected_routes = {
        let submit_reading = warp::path!("readings")
            .and(warp::post())
            .and(middleware::auth::with_api_key(
                db_pool.clone(),
                rate_limiter.clone(),
            ))
            .and(middleware::validation::Validator::body_limit())
            .and(with_db(db_pool.clone()))
            .and(middleware::validation::validate_reading_request())
            .and_then(handlers::business::submit_reading);

        let get_readings = warp::path!("readings")
            .and(warp::get())
            .and(middleware::auth::with_api_key(
                db_pool.clone(),
                rate_limiter.clone(),
            ))
            .and(with_db(db_pool.clone()))
            .and_then(handlers::business::get_readings);

        submit_reading.or(get_readings)
    };

    let metrics = warp::path!("metrics")
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handlers::metrics::get_metrics);

    let routes = health
        .or(metrics)
        .or(admin_routes)
        .or(protected_routes)
        .recover(middleware::auth::handle_rejection);

    let routes = routes.with(middleware::auth::with_request_logging(db_pool.clone()));

    tracing::info!("Server starting on {}:{}", host, port);

    warp::serve(routes)
        .run((host.parse::<std::net::IpAddr>()?, port))
        .await;

    Ok(())
}

// Function to pass database pool to handlers
fn with_db(
    db: db::DbPool,
) -> impl Filter<Extract = (db::DbPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}
