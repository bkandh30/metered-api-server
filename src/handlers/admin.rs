use crate::db::DbPool;
use crate::models::{
    ApiKey, ApiKeyInfo, ApiKeyListResponse, CreateApiKeyRequest, CreateApiKeyResponse,
};
use rand::Rng;
use std::convert::Infallible;
use uuid::Uuid;
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

pub async fn list_api_keys(db: DbPool) -> Result<impl Reply, Infallible> {
    let result = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys ORDER BY created_at DESC")
        .fetch_all(&*db)
        .await;

    match result {
        Ok(keys) => {
            let key_infos: Vec<ApiKeyInfo> = keys
                .into_iter()
                .map(|k| ApiKeyInfo {
                    id: k.id,
                    name: k.name,
                    usage_count: k.usage_count,
                    is_active: k.is_active,
                    created_at: k.created_at,
                })
                .collect();

            let response = ApiKeyListResponse { keys: key_infos };
            Ok(reply::with_status(reply::json(&response), StatusCode::OK))
        }
        Err(e) => {
            tracing::error!("Failed to list API keys: {:?}", e);
            Ok(reply::with_status(
                reply::json(&serde_json::json!({
                    "error": "Failed to list API keys"
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn delete_api_key(id: String, db: DbPool) -> Result<impl Reply, Infallible> {
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return Ok(reply::with_status(
                reply::json(&serde_json::json!({
                    "error": "Invalid UUID format"
                })),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    let result = sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(uuid)
        .execute(&*db)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                Ok(reply::with_status(
                    reply::json(&serde_json::json!({
                        "message": "API key not found"
                    })),
                    StatusCode::NOT_FOUND,
                ))
            } else {
                Ok(reply::with_status(
                    reply::json(&serde_json::json!({
                        "message": "API key deleted successfully"
                    })),
                    StatusCode::OK,
                ))
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete API key: {:?}", e);
            Ok(reply::with_status(
                reply::json(&serde_json::json!({
                    "error": "Failed to delete API key"
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
