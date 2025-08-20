mod db;
mod handlers;
mod models;

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

        create_key
    };

    let routes = health.or(admin_routes);

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
