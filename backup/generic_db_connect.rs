use axum::{extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};

// Generic GET
pub async fn generic_db_connect<T>(
    State(pg_pool): State<PgPool>,
    function_name: String,
    params: Value,
) -> Result<(StatusCode, String), (StatusCode, String)>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let query = format!("SELECT {}($1::jsonb) as data", function_name);

    let row = sqlx::query(&query)
        .bind(params)
        .fetch_one(&pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;

    let result_json: Value = row.try_get("data").map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": format!("Failed to get JSON from row: {}", e)})
                .to_string(),
        )
    })?;

    let data: Vec<T> = serde_json::from_value(result_json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": format!("Failed to deserialize data: {}", e)})
                .to_string(),
        )
    })?;

    Ok((
        StatusCode::OK,
        json!({"success": true, "data": data}).to_string(),
    ))
}
