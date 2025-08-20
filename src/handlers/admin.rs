use crate::db::DbPool;
use crate::models::{ApiKey, CreateApiKeyRequest, CreateApiKeyResponse};
use rand::Rng;
use std::convert::Infallible;
use warp::{Reply, http::StatusCode, reply};

fn generate_api_key() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const KEY_LEN: usize = 32;
    let mut rng = rand::rng();

    let key: String = (0..KEY_LEN)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("sk_{}", key)
}

pub async fn create_api_key(
    body: CreateApiKeyRequest,
    db: DbPool,
) -> Result<impl Reply, Infallible> {
    let key = generate_api_key();

    let result = sqlx::query_as::<_, ApiKey>(
        r#"
        INSERT INTO api_keys (key, name)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(&key)
    .bind(&body.name)
    .fetch_one(&*db)
    .await;

    match result {
        Ok(api_key) => {
            let response = CreateApiKeyResponse {
                id: api_key.id,
                key: api_key.key,
                name: api_key.name,
            };
            Ok(reply::with_status(
                reply::json(&response),
                StatusCode::CREATED,
            ))
        }
        Err(e) => {
            tracing::error!("Failed to create API key: {:?}", e);
            Ok(reply::with_status(
                reply::json(&serde_json::json!({
                    "error": "Failed to create API key"
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
