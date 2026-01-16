use anyhow::Result;
use serde::{Deserialize, Serialize, Serializer};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeishuGroup {
    pub id: i64,
    pub name: String,
    pub webhook_url: String,
    pub description: Option<String>,
    pub card_title_template: Option<String>,
    pub card_color: Option<String>,
    #[serde(serialize_with = "serialize_option_i32_as_bool")]
    pub card_show_details_button: Option<i32>,
    pub card_config_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn serialize_option_i32_as_bool<S>(v: &Option<i32>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match v {
        Some(n) => s.serialize_some(&(*n == 1)),
        None => s.serialize_none(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateGroup {
    pub name: String,
    pub webhook_url: String,
    pub description: Option<String>,
    pub card_title_template: Option<String>,
    pub card_color: Option<String>,
    pub card_show_details_button: Option<bool>,
    pub card_config_json: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroup {
    pub name: Option<String>,
    pub webhook_url: Option<String>,
    pub description: Option<String>,
    pub card_title_template: Option<String>,
    pub card_color: Option<String>,
    pub card_show_details_button: Option<bool>,
    pub card_config_json: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<FeishuGroup>> {
    let groups =
        sqlx::query_as::<_, FeishuGroup>("SELECT * FROM feishu_groups ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?;

    Ok(groups)
}

pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<FeishuGroup>> {
    let group = sqlx::query_as::<_, FeishuGroup>("SELECT * FROM feishu_groups WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(group)
}

pub async fn create(pool: &SqlitePool, input: CreateGroup) -> Result<FeishuGroup> {
    let card_show_details = input
        .card_show_details_button
        .map(|b| if b { 1 } else { 0 });

    let result = sqlx::query(
        r#"
        INSERT INTO feishu_groups (
            name, webhook_url, description,
            card_title_template, card_color, card_show_details_button,
            card_config_json
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&input.name)
    .bind(&input.webhook_url)
    .bind(&input.description)
    .bind(&input.card_title_template)
    .bind(&input.card_color)
    .bind(card_show_details)
    .bind(&input.card_config_json)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    get_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch created group"))
}

pub async fn update(pool: &SqlitePool, id: i64, input: UpdateGroup) -> Result<Option<FeishuGroup>> {
    // First check if group exists
    let existing = get_by_id(pool, id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let existing = existing.unwrap();

    let name = input.name.unwrap_or(existing.name);
    let webhook_url = input.webhook_url.unwrap_or(existing.webhook_url);
    let description = input.description.or(existing.description);
    let card_title_template = input.card_title_template.or(existing.card_title_template);
    let card_color = input.card_color.or(existing.card_color);
    let card_show_details = input
        .card_show_details_button
        .map(|b| if b { 1 } else { 0 })
        .or(existing.card_show_details_button);
    let card_config_json = input.card_config_json.or(existing.card_config_json);

    sqlx::query(
        r#"
        UPDATE feishu_groups SET
            name = ?, webhook_url = ?, description = ?,
            card_title_template = ?, card_color = ?, card_show_details_button = ?,
            card_config_json = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(&name)
    .bind(&webhook_url)
    .bind(&description)
    .bind(&card_title_template)
    .bind(&card_color)
    .bind(card_show_details)
    .bind(&card_config_json)
    .bind(id)
    .execute(pool)
    .await?;

    get_by_id(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM feishu_groups WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
