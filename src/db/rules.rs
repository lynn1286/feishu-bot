use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoutingRule {
    pub id: i64,
    pub name: String,
    pub project_match: Option<String>,
    pub environment_match: Option<String>,
    pub level_match: Option<String>,
    pub group_id: i64,
    pub priority: i32,
    pub enabled: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct RoutingRuleWithGroup {
    #[serde(flatten)]
    pub rule: RoutingRule,
    pub group_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRule {
    pub name: String,
    pub project_match: Option<String>,
    pub environment_match: Option<String>,
    pub level_match: Option<String>,
    pub group_id: i64,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRule {
    pub name: Option<String>,
    pub project_match: Option<String>,
    pub environment_match: Option<String>,
    pub level_match: Option<String>,
    pub group_id: Option<i64>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<RoutingRuleWithGroup>> {
    let rules = sqlx::query_as::<_, RoutingRule>(
        "SELECT * FROM routing_rules ORDER BY priority DESC, created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for rule in rules {
        let group_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM feishu_groups WHERE id = ?"
        )
        .bind(rule.group_id)
        .fetch_optional(pool)
        .await?;

        result.push(RoutingRuleWithGroup { rule, group_name });
    }

    Ok(result)
}

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<RoutingRule>> {
    let rule = sqlx::query_as::<_, RoutingRule>(
        "SELECT * FROM routing_rules WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(rule)
}

pub async fn create(pool: &SqlitePool, input: CreateRule) -> Result<RoutingRule> {
    let priority = input.priority.unwrap_or(0);
    let enabled = if input.enabled.unwrap_or(true) { 1 } else { 0 };

    let result = sqlx::query(
        r#"
        INSERT INTO routing_rules (
            name, project_match, environment_match, level_match,
            group_id, priority, enabled
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&input.name)
    .bind(&input.project_match)
    .bind(&input.environment_match)
    .bind(&input.level_match)
    .bind(input.group_id)
    .bind(priority)
    .bind(enabled)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    get_by_id(pool, id).await?.ok_or_else(|| anyhow::anyhow!("Failed to fetch created rule"))
}

pub async fn update(pool: &SqlitePool, id: i64, input: UpdateRule) -> Result<Option<RoutingRule>> {
    let existing = get_by_id(pool, id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let existing = existing.unwrap();

    let name = input.name.unwrap_or(existing.name);
    let project_match = input.project_match.or(existing.project_match);
    let environment_match = input.environment_match.or(existing.environment_match);
    let level_match = input.level_match.or(existing.level_match);
    let group_id = input.group_id.unwrap_or(existing.group_id);
    let priority = input.priority.unwrap_or(existing.priority);
    let enabled = input.enabled.map(|b| if b { 1 } else { 0 }).unwrap_or(existing.enabled);

    sqlx::query(
        r#"
        UPDATE routing_rules SET
            name = ?, project_match = ?, environment_match = ?, level_match = ?,
            group_id = ?, priority = ?, enabled = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#
    )
    .bind(&name)
    .bind(&project_match)
    .bind(&environment_match)
    .bind(&level_match)
    .bind(group_id)
    .bind(priority)
    .bind(enabled)
    .bind(id)
    .execute(pool)
    .await?;

    get_by_id(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM routing_rules WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Get all enabled rules sorted by priority
pub async fn get_enabled_rules(pool: &SqlitePool) -> Result<Vec<RoutingRule>> {
    let rules = sqlx::query_as::<_, RoutingRule>(
        "SELECT * FROM routing_rules WHERE enabled = 1 ORDER BY priority DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(rules)
}
