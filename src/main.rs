use axum::{
    routing::{get, post},
    Router,
};
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;

mod models;
mod handlers;

pub struct AppState {
    pub db: Pool<Postgres>,
    pub rdb: redis::Client,
}

#[tokio::main]
async fn main() {
    if let Err(_) = dotenvy::dotenv() {
        println!("INFO: No .env file found. Relying on system environment variables.");
    }

    let db_host = env::var("DATABASE_HOST").unwrap_or_else(|_| "10.36.168.15".to_string());
    let db_port = env::var("DATABASE_PORT").unwrap_or_else(|_| "5432".to_string());
    let db_user = env::var("DB_CREDS_USR").unwrap_or_default();
    let db_pass = env::var("DB_CREDS_PSW").unwrap_or_default();
    let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());

    println!("DEBUG DB: host={} port={} user={}", db_host, db_port, db_user);

    let db_url = format!(
        "postgres://{}:{}@{}:{}/mdbase",
        db_user, db_pass, db_host, db_port
    );

    let db_pool = match Pool::<Postgres>::connect(&db_url).await {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("Failed to initialize postgres database: {}", err);
            std::process::exit(1);
        }
    };

    let redis_url = format!("redis://{}:6379", redis_host);
    let redis_client = redis::Client::open(redis_url).unwrap();

    let shared_state = Arc::new(AppState {
        db: db_pool,
        rdb: redis_client,
    });

    let app = Router::new()
        .route("/hello", get(handlers::get_account_holder))
        .route("/updateStatus", post(handlers::update_account_status))
        .route("/updateStatusBulk", post(handlers::update_account_status_bulk))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("Server running on http://localhost:8081");
    axum::serve(listener, app).await.unwrap();
}