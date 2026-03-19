use std::sync::Arc;

use object_store::aws::AmazonS3Builder;
use tracing_subscriber;

mod auth;
mod auth_user;
mod db;
mod error;
mod http;
mod jwt;
mod repos;
mod services;
mod state;
use state::AppState;
mod invite;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    dotenvy::from_filename("../../.env").ok(); // repo root
    dotenvy::dotenv().ok(); // apps/api/.env

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is not set (check root .env)");

    let pool = db::create_pool(&database_url).await?;

    db::ping(&pool).await?;

    let upload_store = build_upload_store();
    let state = AppState {
        db: pool,
        upload_store,
    };
    let app = routes::router(state);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("Koro API listening on http://localhost:{port}");

    axum::serve(listener, app).await?;

    Ok(())
}

fn build_upload_store() -> Option<Arc<object_store::aws::AmazonS3>> {
    let endpoint = std::env::var("MINIO_ENDPOINT").ok()?;
    let access_key = std::env::var("MINIO_ACCESS_KEY").ok()?;
    let secret_key = std::env::var("MINIO_SECRET_KEY").ok()?;
    let bucket = std::env::var("MINIO_BUCKET").ok()?;

    let store = AmazonS3Builder::new()
        .with_endpoint(endpoint)
        .with_allow_http(true)
        .with_region("us-east-1")
        .with_bucket_name(&bucket)
        .with_access_key_id(access_key)
        .with_secret_access_key(secret_key)
        .build()
        .ok()?;

    Some(Arc::new(store))
}
