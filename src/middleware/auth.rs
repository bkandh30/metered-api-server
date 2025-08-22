use crate::db::DbPool;
use crate::middleware::rate_limiter::RateLimitExceeded;
use crate::models::ApiKey;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::{Filter, Rejection, log, reject, reply};

#[derive(Debug)]
pub struct Unauthorized;
impl reject::Reject for Unauthorized {}

pub fn with_api_key(db: DbPool) -> impl Filter<Extract = (ApiKey,), Error = Rejection> + Clone {
    warp::header::optional::<String>("x-api-key")
        .and(warp::any().map(move || db.clone()))
        .and_then(validate_api_key)
}

async fn validate_api_key(api_key: Option<String>, db: DbPool) -> Result<ApiKey, Rejection> {
    let key = api_key.ok_or(warp::reject::custom(Unauthorized))?;

    let result = sqlx::query_as::<_, ApiKey>(
        r#"
        UPDATE api_keys
        SET usage_count = usage_count + 1,
            updated_at = NOW()
        WHERE key = $1 AND is_active = true
        RETURNING *
        "#,
    )
    .bind(key)
    .fetch_optional(&*db)
    .await;

    match result {
        Ok(Some(api_key_record)) => Ok(api_key_record),
        Ok(None) => Err(reject::custom(Unauthorized)),
        Err(e) => {
            tracing::error!("Database error during API key validation: {:?}", e);
            Err(reject::custom(Unauthorized))
        }
    }
}

pub fn with_request_logging(db: DbPool) -> log::Log<impl Fn(log::Info) + Clone> {
    log::custom(move |info: log::Info| {
        let path = info.path().to_owned();
        let method = info.method().clone();
        let status = info.status();
        let elapsed = info.elapsed();
        let headers = info.request_headers().clone();
        let db = db.clone();

        tokio::spawn(async move {
            let api_key_id =
                if let Some(key) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
                    sqlx::query!(
                        r#"
                    SELECT id FROM api_keys
                    WHERE key = $1
                    "#,
                        key
                    )
                    .fetch_optional(&*db)
                    .await
                    .ok()
                    .flatten()
                    .map(|rec| rec.id)
                } else {
                    None
                };

            if let Some(id) = api_key_id {
                let result = sqlx::query!(
                    r#"
                    INSERT INTO requests (id, api_key_id, endpoint, method, status_code, response_time_ms)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    Uuid::new_v4(),
                    id,
                    path,
                    method.as_str(),
                    status.as_u16() as i16,
                    elapsed.as_millis() as i32
                )
                .execute(&*db)
                .await;

                if let Err(e) = result {
                    tracing::error!("Failed to log request to database: {:?}", e);
                }
            }
        });
    })
}

pub async fn handle_rejection(
    err: Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Requested resource was not found.";
    } else if err.find::<Unauthorized>().is_some() {
        code = StatusCode::UNAUTHORIZED;
        message = "Authentication error: API key is invalid or missing.";
    } else if err.find::<QuotaExceeded>().is_some() {
        code = StatusCode::FORBIDDEN;
        message = "API key has exceeded its request quota.";
    } else if err.find::<RateLimitExceeded>().is_some() {
        code = StatusCode::TOO_MANY_REQUESTS;
        message = "Rate limit exceeded. Please slow down.";
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "HTTP method is not allowed for the requested resource.";
    } else {
        tracing::error!("Unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error.";
    }

    let json = warp::reply::json(&serde_json::json!({
        "error": message
    }));

    Ok(reply::with_status(json, code))
}
