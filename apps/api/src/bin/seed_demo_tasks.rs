use anyhow::Context;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[derive(Deserialize)]
struct SeedTask {
    id: String,
    title: String,
    status: String,
    label: String,
    priority: String,
}

const TASKS_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../web/src/app/dashboard/my-issues/data/tasks.json"
));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_filename("../../.env").ok();
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is not set (check root .env)");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .context("failed to connect to Postgres")?;

    let tasks: Vec<SeedTask> =
        serde_json::from_str(TASKS_JSON).context("failed to parse tasks.json")?;

    let mut tx = pool.begin().await.context("failed to start transaction")?;

    sqlx::query("TRUNCATE TABLE demo_tasks")
        .execute(&mut *tx)
        .await
        .context("failed to clear demo_tasks")?;

    for (sort_order, task) in tasks.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO demo_tasks (id, title, status, label, priority, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&task.id)
        .bind(&task.title)
        .bind(&task.status)
        .bind(&task.label)
        .bind(&task.priority)
        .bind(sort_order as i32)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("failed to insert demo task {}", task.id))?;
    }

    tx.commit()
        .await
        .context("failed to commit seed transaction")?;

    println!("Seeded {} demo tasks into demo_tasks", tasks.len());

    Ok(())
}
