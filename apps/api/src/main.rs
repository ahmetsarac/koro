use tracing_subscriber;

mod modules;

use modules::core::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    modules::core::config::load_dotenv();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is not set (check root .env)");

    let pool = modules::core::db::create_pool(&database_url).await?;

    modules::core::db::ping(&pool).await?;

    let upload_store = modules::core::uploads::build_upload_store();
    let state = AppState {
        db: pool,
        upload_store,
    };
    let app = modules::core::http::router(state);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("Koro API listening on http://localhost:{port}");
    println!("OpenAPI JSON: http://localhost:{port}/openapi.json");
    println!("Swagger UI:  http://localhost:{port}/swagger-ui/");

    axum::serve(listener, app).await?;

    Ok(())
}
