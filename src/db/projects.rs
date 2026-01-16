use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SentryProject {
    pub id: i64,
    pub project_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub project_id: String,
    pub display_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProject {
    pub project_id: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<SentryProject>> {
    let projects = sqlx::query_as::<_, SentryProject>(
        "SELECT * FROM sentry_projects ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(projects)
}

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<SentryProject>> {
    let project = sqlx::query_as::<_, SentryProject>("SELECT * FROM sentry_projects WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(project)
}

pub async fn get_by_project_id(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Option<SentryProject>> {
    let project =
        sqlx::query_as::<_, SentryProject>("SELECT * FROM sentry_projects WHERE project_id = ?")
            .bind(project_id)
            .fetch_optional(pool)
            .await?;

    Ok(project)
}

pub async fn create(pool: &SqlitePool, input: CreateProject) -> Result<SentryProject> {
    let result = sqlx::query(
        r#"
        INSERT INTO sentry_projects (project_id, display_name, description)
        VALUES (?, ?, ?)
        ON CONFLICT(project_id) DO UPDATE SET
            display_name = excluded.display_name,
            description = excluded.description,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&input.project_id)
    .bind(&input.display_name)
    .bind(&input.description)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    get_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch created project"))
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: UpdateProject,
) -> Result<Option<SentryProject>> {
    let existing = get_by_id(pool, id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let existing = existing.unwrap();

    let project_id = input.project_id.unwrap_or(existing.project_id);
    let display_name = input.display_name.unwrap_or(existing.display_name);
    let description = input.description.or(existing.description);

    sqlx::query(
        r#"
        UPDATE sentry_projects SET
            project_id = ?, display_name = ?, description = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(&project_id)
    .bind(&display_name)
    .bind(&description)
    .bind(id)
    .execute(pool)
    .await?;

    get_by_id(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM sentry_projects WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Get display name for a project ID, returns the project_id itself if not found
pub async fn get_display_name(pool: &SqlitePool, project_id: &str) -> String {
    match get_by_project_id(pool, project_id).await {
        Ok(Some(project)) => project.display_name,
        _ => project_id.to_string(),
    }
}
