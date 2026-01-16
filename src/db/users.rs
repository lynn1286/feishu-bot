use anyhow::Result;
use sqlx::SqlitePool;

pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<(i64, String)>> {
    let row = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, password_hash FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_user(pool: &SqlitePool, username: &str, password_hash: &str) -> Result<i64> {
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)"
    )
    .bind(username)
    .bind(password_hash)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn update_password(pool: &SqlitePool, username: &str, password_hash: &str) -> Result<()> {
    sqlx::query(
        "UPDATE users SET password_hash = ?, updated_at = CURRENT_TIMESTAMP WHERE username = ?"
    )
    .bind(password_hash)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}
