use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::models::{AccountQuery, AccountStatusRequest};
use crate::AppState;

// 1. GET /hello?accountNo=xxxx
pub async fn get_account_holder(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AccountQuery>,
) -> (StatusCode, String) {
    let redis_key = format!("account:{}", query.account_no);

    if let Ok(mut con) = state.rdb.get_multiplexed_tokio_connection().await {
        let redis_result: Result<String, _> = redis::cmd("GET").arg(&redis_key).query_async(&mut con).await;
        if let Ok(cached) = redis_result {
            println!("DEBUG REDIS: cached={}, err=None", cached);
            return (StatusCode::OK, format!("Account Holder: {}", cached));
        }
    }

    let sql_query = "SELECT gl_segment FROM bill_media WHERE rec_id = '3' AND account_no = $1";
    let row: Result<String, sqlx::Error> = sqlx::query_scalar(sql_query)
        .bind(&query.account_no)
        .fetch_one(&state.db)
        .await;

    match row {
        Ok(holder_name) => {
            if let Ok(mut con) = state.rdb.get_multiplexed_tokio_connection().await {
                let _: Result<(), _> = redis::cmd("SETEX").arg(&redis_key).arg(120).arg(&holder_name).query_async(&mut con).await;
            }
            println!("DEBUG DB HIT: account={}", query.account_no);
            (StatusCode::OK, format!("Account Holder: {}", holder_name))
        }
        Err(_) => (
            StatusCode::OK,
            format!("Error: Account number {} not found.", query.account_no),
        ),
    }
}

// 2. POST /updateStatus
pub async fn update_account_status(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AccountStatusRequest>,
) -> (StatusCode, String) {
    let sql_query = "UPDATE cim_account SET status = $1 WHERE account_no = $2";
    
    let result = sqlx::query(sql_query)
        .bind(&payload.status)
        .bind(&payload.account_no)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                (
                    StatusCode::OK,
                    format!("SUCCESS: Account {} updated to status '{}'", payload.account_no, payload.status),
                )
            } else {
                (
                    StatusCode::OK,
                    format!("WARNING! No record found for account_no {}", payload.account_no),
                )
            }
        }
        Err(err) => (
            StatusCode::OK,
            format!("FAILURE: Database error: {}", err),
        ),
    }
}

// 3. POST /updateStatusBulk
pub async fn update_account_status_bulk(
    State(state): State<Arc<AppState>>,
    Json(payload_list): Json<Vec<AccountStatusRequest>>,
) -> (StatusCode, String) {
    let json_payload = match serde_json::to_value(&payload_list) {
        Ok(val) => val,
        Err(err) => return (StatusCode::OK, format!("FAILURE: JSON marshal error: {}", err)),
    };

    let sql_query = "SELECT update_account_status_bulk($1::jsonb)";
    let result: Result<i32, sqlx::Error> = sqlx::query_scalar(sql_query)
        .bind(json_payload)
        .fetch_one(&state.db)
        .await;

    match result {
        Ok(total_rows_updated) => (
            StatusCode::OK,
            format!("SUCCESS: Record-batch complete. Total records updated: {}", total_rows_updated),
        ),
        Err(err) => (
            StatusCode::OK,
            format!("FAILURE: Database error: {}", err),
        ),
    }
}