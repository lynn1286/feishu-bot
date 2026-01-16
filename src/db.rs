pub mod alerts;
pub mod groups;
pub mod projects;
pub mod rules;
pub mod users;

use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

const SCHEMA: &str = r#"
-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Feishu group configuration
CREATE TABLE IF NOT EXISTS feishu_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    webhook_url TEXT NOT NULL,
    description TEXT,

    -- Interactive card message configuration
    card_title_template TEXT DEFAULT '🚨 Sentry 告警',
    card_color TEXT DEFAULT 'red',
    card_show_details_button INTEGER DEFAULT 1,
    card_config_json TEXT,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Routing rules
CREATE TABLE IF NOT EXISTS routing_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    project_match TEXT,        -- Project name match (supports wildcard *)
    environment_match TEXT,    -- Environment match
    level_match TEXT,          -- Alert level match
    group_id INTEGER NOT NULL,
    priority INTEGER DEFAULT 0,
    enabled INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES feishu_groups(id) ON DELETE CASCADE
);

-- Alert history
CREATE TABLE IF NOT EXISTS alert_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sentry_event_id TEXT,
    issue_id TEXT,
    project TEXT,
    environment TEXT,
    platform TEXT,
    title TEXT,
    message TEXT,
    level TEXT,
    web_url TEXT,
    triggered_rule TEXT,
    matched_rule_id INTEGER,
    target_group_id INTEGER,
    target_group_name TEXT,
    status TEXT NOT NULL,      -- success / failed / no_match
    error_message TEXT,
    raw_payload TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (matched_rule_id) REFERENCES routing_rules(id) ON DELETE SET NULL,
    FOREIGN KEY (target_group_id) REFERENCES feishu_groups(id) ON DELETE SET NULL
);

-- Sentry project ID to name mapping
CREATE TABLE IF NOT EXISTS sentry_projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_alert_history_created_at ON alert_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_history_status ON alert_history(status);
CREATE INDEX IF NOT EXISTS idx_alert_history_project ON alert_history(project);
CREATE INDEX IF NOT EXISTS idx_alert_history_issue_id ON alert_history(issue_id);
CREATE INDEX IF NOT EXISTS idx_routing_rules_priority ON routing_rules(priority DESC);
CREATE INDEX IF NOT EXISTS idx_routing_rules_enabled ON routing_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_sentry_projects_project_id ON sentry_projects(project_id);
"#;

pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    tracing::info!("Connecting to database: {}", database_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::raw_sql(SCHEMA).execute(&pool).await?;

    // Primitive migrations for existing tables
    // Ignore errors as SQLite doesn't support 'ADD COLUMN IF NOT EXISTS'
    let _ = sqlx::query("ALTER TABLE feishu_groups ADD COLUMN card_config_json TEXT")
        .execute(&pool)
        .await;

    tracing::info!("Database initialized successfully");
    Ok(pool)
}
