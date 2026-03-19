use tracing_subscriber;

mod auth;
mod comments;
mod core;
mod error;
mod events;
mod invites;
mod issues;
mod orgs;
mod projects;
mod relations;
mod users;

use core::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    core::config::load_dotenv();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is not set (check root .env)");

    let pool = core::db::create_pool(&database_url).await?;

    core::db::ping(&pool).await?;

    let upload_store = core::uploads::build_upload_store();
    let state = AppState {
        db: pool,
        upload_store,
    };
    let app = core::http::router(state);

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
