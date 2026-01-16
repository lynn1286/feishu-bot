use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    /// Database URL (SQLite path)
    pub database_url: String,

    /// Sentry webhook client secret for signature verification
    pub sentry_client_secret: Option<String>,

    /// Data directory for storing database and logs
    pub data_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Determine data directory
        let data_dir = env::var("FEISHU_BOT_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("feishu-bot")
            });

        // Ensure data directory exists
        std::fs::create_dir_all(&data_dir)
            .with_context(|| format!("Failed to create data directory: {:?}", data_dir))?;

        // Database URL
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            format!("sqlite:{}/data.db?mode=rwc", data_dir.display())
        });

        // Sentry client secret (optional, for signature verification)
        let sentry_client_secret = env::var("SENTRY_CLIENT_SECRET").ok();

        Ok(Config {
            database_url,
            sentry_client_secret,
            data_dir,
        })
    }
}
