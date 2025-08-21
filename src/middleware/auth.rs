use crate::db::DbPool;
use crate::models::ApiKey;
use warp::http::StatusCode;
use warp::{Filter, Rejection, reject, reply};

#[derive(Debug)]
pub struct Unauthorized;
impl reject::Reject for Unauthorized {}

pub fn with_api_key(db: DbPool) -> impl Filter<Extract = (ApiKey,), Error = Rejection> + Clone {
    warp::header::optional::<String>("x-api-key")
        .and(warp::any().map(move || db.clone()))
        .and_then(validate_api_key)
}

async fn validate_api_key(api_key: Option<String>, db: DbPool) -> Result<ApiKey, Rejection> {
    let key = api_key.ok_or_else(|| warp::reject::custom(Unauthorized))?;

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

pub async fn handle_rejection(
    err: Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(Unauthorized) = err.find::<Unauthorized>() {
        code = StatusCode::UNAUTHORIZED;
        message = "Unauthorized: Invalid or missing API key";
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed";
    } else {
        tracing::error!("Unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let json = warp::reply::json(&serde_json::json!({
        "error": message
    }));

    Ok(reply::with_status(json, code))
}
