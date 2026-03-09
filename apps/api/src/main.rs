use tracing_subscriber;

mod auth;
mod auth_user;
mod db;
mod http;
mod jwt;
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

    let state = AppState { db: pool };
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
