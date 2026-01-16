use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertHistory {
    pub id: i64,
    pub sentry_event_id: Option<String>,
    pub issue_id: Option<String>,
    pub project: Option<String>,
    pub environment: Option<String>,
    pub platform: Option<String>,
    pub title: Option<String>,
    pub message: Option<String>,
    pub level: Option<String>,
    pub web_url: Option<String>,
    pub triggered_rule: Option<String>,
    pub matched_rule_id: Option<i64>,
    pub target_group_id: Option<i64>,
    pub target_group_name: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub raw_payload: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlert {
    pub sentry_event_id: Option<String>,
    pub issue_id: Option<String>,
    pub project: Option<String>,
    pub environment: Option<String>,
    pub platform: Option<String>,
    pub title: Option<String>,
    pub message: Option<String>,
    pub level: Option<String>,
    pub web_url: Option<String>,
    pub triggered_rule: Option<String>,
    pub matched_rule_id: Option<i64>,
    pub target_group_id: Option<i64>,
    pub target_group_name: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub raw_payload: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListAlertsQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub status: Option<String>,
    pub project: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedAlerts {
    pub data: Vec<AlertHistory>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

pub async fn list(pool: &SqlitePool, query: ListAlertsQuery) -> Result<PaginatedAlerts> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    // Build WHERE clause
    let mut conditions = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(status) = &query.status {
        conditions.push("status = ?");
        bind_values.push(status.clone());
    }

    if let Some(project) = &query.project {
        conditions.push("project LIKE ?");
        bind_values.push(format!("%{}%", project));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Get total count
    let count_sql = format!("SELECT COUNT(*) FROM alert_history {}", where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for value in &bind_values {
        count_query = count_query.bind(value);
    }
    let total: i64 = count_query.fetch_one(pool).await?;

    // Get paginated data
    let data_sql = format!(
        "SELECT * FROM alert_history {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );
    let mut data_query = sqlx::query_as::<_, AlertHistory>(&data_sql);
    for value in &bind_values {
        data_query = data_query.bind(value);
    }
    let alerts = data_query
        .bind(page_size as i32)
        .bind(offset as i32)
        .fetch_all(pool)
        .await?;

    let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

    Ok(PaginatedAlerts {
        data: alerts,
        total,
        page,
        page_size,
        total_pages,
    })
}

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<AlertHistory>> {
    let alert = sqlx::query_as::<_, AlertHistory>("SELECT * FROM alert_history WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(alert)
}

pub async fn create(pool: &SqlitePool, input: CreateAlert) -> Result<AlertHistory> {
    let result = sqlx::query(
        r#"
        INSERT INTO alert_history (
            sentry_event_id, issue_id, project, environment, platform,
            title, message, level, web_url, triggered_rule,
            matched_rule_id, target_group_id, target_group_name,
            status, error_message, raw_payload
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&input.sentry_event_id)
    .bind(&input.issue_id)
    .bind(&input.project)
    .bind(&input.environment)
    .bind(&input.platform)
    .bind(&input.title)
    .bind(&input.message)
    .bind(&input.level)
    .bind(&input.web_url)
    .bind(&input.triggered_rule)
    .bind(input.matched_rule_id)
    .bind(input.target_group_id)
    .bind(&input.target_group_name)
    .bind(&input.status)
    .bind(&input.error_message)
    .bind(&input.raw_payload)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    get_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch created alert"))
}

pub async fn update_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
    error_message: Option<&str>,
) -> Result<()> {
    sqlx::query("UPDATE alert_history SET status = ?, error_message = ? WHERE id = ?")
        .bind(status)
        .bind(error_message)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct AlertStats {
    pub total: i64,
    pub success: i64,
    pub failed: i64,
    pub today: i64,
}

pub async fn get_stats(pool: &SqlitePool) -> Result<AlertStats> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alert_history")
        .fetch_one(pool)
        .await?;

    let success: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM alert_history WHERE status = 'success'")
            .fetch_one(pool)
            .await?;

    let failed = total - success;

    let today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alert_history WHERE date(created_at) = date('now')",
    )
    .fetch_one(pool)
    .await?;

    Ok(AlertStats {
        total,
        success,
        failed,
        today,
    })
}
