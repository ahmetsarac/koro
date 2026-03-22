use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use uuid::Uuid;

const ISSUE_COUNT: i32 = 500;

const WORKFLOW_SLUGS: [&str; 4] = ["backlog", "todo", "in_progress", "done"];
const PRIORITIES: [&str; 4] = ["critical", "high", "medium", "low"];

const TITLES: [&str; 20] = [
    "Fix login page styling",
    "Add user profile settings",
    "Implement dark mode toggle",
    "Refactor authentication flow",
    "Add email notifications",
    "Fix mobile responsive layout",
    "Implement search functionality",
    "Add pagination to list views",
    "Fix memory leak in dashboard",
    "Add export to CSV feature",
    "Implement drag and drop",
    "Fix timezone handling",
    "Add keyboard shortcuts",
    "Improve loading performance",
    "Fix Safari compatibility",
    "Add bulk actions support",
    "Implement undo/redo",
    "Fix race condition in sync",
    "Add offline support",
    "Implement real-time updates",
];

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

    // Find first user to use as assignee
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users LIMIT 1")
        .fetch_optional(&pool)
        .await
        .context("failed to query users")?
        .context("No users found. Please create a user first (run the app and register).")?;

    println!("Using user {} as assignee", user_id);

    // Find first org or fail
    let org_id: Uuid = sqlx::query_scalar("SELECT id FROM organizations LIMIT 1")
        .fetch_optional(&pool)
        .await
        .context("failed to query organizations")?
        .context("No organizations found. Please create an org first.")?;

    println!("Using org {}", org_id);

    // Find first project in that org or fail
    let project_row: Option<(Uuid, String, i32)> = sqlx::query_as(
        "SELECT id, project_key, next_issue_seq FROM projects WHERE org_id = $1 LIMIT 1",
    )
    .bind(org_id)
    .fetch_optional(&pool)
    .await
    .context("failed to query projects")?;

    let (project_id, project_key, mut next_seq) = project_row
        .context("No projects found in this org. Please create a project first.")?;

    println!("Using project {} ({})", project_key, project_id);

    // Add user as project member if not already
    sqlx::query(
        r#"
        INSERT INTO project_members (project_id, user_id, project_role)
        VALUES ($1, $2, 'developer')
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .context("failed to add user as project member")?;

    let mut tx = pool.begin().await.context("failed to start transaction")?;

    // Delete existing issues for this project (for clean re-seed)
    let deleted = sqlx::query("DELETE FROM issues WHERE project_id = $1")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .context("failed to clear existing issues")?;

    println!("Deleted {} existing issues", deleted.rows_affected());

    // Reset next_issue_seq
    next_seq = 1;

    for i in 0..ISSUE_COUNT {
        let title = format!(
            "{} #{}",
            TITLES[i as usize % TITLES.len()],
            i + 1
        );
        let slug = WORKFLOW_SLUGS[i as usize % WORKFLOW_SLUGS.len()];
        let is_blocked = i % 23 == 0;
        let priority = PRIORITIES[i as usize % PRIORITIES.len()];

        let workflow_status_id: Uuid = sqlx::query_scalar(
            r#"SELECT id FROM project_workflow_statuses WHERE project_id = $1 AND slug = $2"#,
        )
        .bind(project_id)
        .bind(slug)
        .fetch_one(&mut *tx)
        .await
        .with_context(|| format!("resolve workflow slug {slug} for project"))?;

        sqlx::query(
            r#"
            INSERT INTO issues (org_id, project_id, key_seq, title, workflow_status_id, is_blocked, priority, assignee_id, reporter_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
            "#,
        )
        .bind(org_id)
        .bind(project_id)
        .bind(next_seq)
        .bind(&title)
        .bind(workflow_status_id)
        .bind(is_blocked)
        .bind(priority)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("failed to insert issue {}", next_seq))?;

        next_seq += 1;

        if (i + 1) % 100 == 0 {
            println!("Inserted {} issues...", i + 1);
        }
    }

    // Update project's next_issue_seq
    sqlx::query("UPDATE projects SET next_issue_seq = $1 WHERE id = $2")
        .bind(next_seq)
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .context("failed to update next_issue_seq")?;

    tx.commit()
        .await
        .context("failed to commit seed transaction")?;

    println!(
        "Seeded {} issues into project {} ({})",
        ISSUE_COUNT, project_key, project_id
    );
    println!("All issues assigned to user {}", user_id);

    Ok(())
}
