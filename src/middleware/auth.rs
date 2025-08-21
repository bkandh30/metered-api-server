use crate::db::DbPool;
use warp::http::StatusCode;
use warp::{Filter, Rejection, reject};

#[derive(Debug)]
pub struct Unauthorized;
impl reject::Reject for Unauthorized {}

pub fn with_api_key(db: DbPool) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::header::optional::<String>("x-api-key")
        .and(warp::any().map(move || db.clone()))
        .and_then(validate_api_key)
}

async fn validate_api_key(api_key: Option<String>, db: DbPool) -> Result<String, Rejection> {
    match api_key {
        None => {
            tracing::warn!("No API key provided");
            Err(reject::custom(Unauthorized))
        }
        Some(key) => {
            let result = sqlx::query!(
                r#"
                UPDATE api_keys
                SET usage_count = usage_count + 1,
                    updated_at = NOW()
                WHERE key = $1 AND is_active = true
                RETURNING id::text
                "#,
                key
            )
            .fetch_optional(&**db)
            .await;

            match result {
                Ok(Some(record)) => {
                    tracing::info!("API key validated and usage incremented");
                    Ok(key)
                }
                Ok(None) => {
                    tracing::warn!("Invalid or inactive API key: {}", key);
                    Err(reject::custom(Unauthorized))
                }
                Err(e) => {
                    tracing::error!("Database error using API key validation: {:?}", e);
                    Err(reject::custom(Unauthorized))
                }
            }
        }
    }
}
