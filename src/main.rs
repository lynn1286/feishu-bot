mod cli;
mod config;
mod db;
mod handlers;
mod models;
mod services;

use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use clap::Parser;
use rand::Rng;
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{Cli, Commands};
use crate::config::Config;

// 临时目录守卫，确保清理
struct TempDirGuard(PathBuf);

impl TempDirGuard {
    fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// 检查依赖工具是否存在 (Unix only: macOS/Linux)
fn check_dependencies() -> anyhow::Result<()> {
    let check_cmd = |cmd: &str| -> bool {
        std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map_or(false, |o| o.status.success())
    };

    let deps = [("git", "git"), ("pnpm", "pnpm"), ("cargo", "Rust/Cargo")];

    let missing: Vec<_> = deps
        .iter()
        .filter(|(cmd, _)| !check_cmd(cmd))
        .map(|(_, name)| *name)
        .collect();

    if !missing.is_empty() {
        anyhow::bail!(
            "Missing required dependencies: {}. Please install them first.",
            missing.join(", ")
        );
    }

    Ok(())
}

pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle commands that don't need full initialization
    match cli.command {
        Commands::Update => {
            return cmd_update();
        }
        Commands::Uninstall => {
            return cmd_uninstall();
        }
        Commands::Logs => {
            return cmd_logs();
        }
        Commands::ResetPassword { password } => {
            return cmd_reset_password(password).await;
        }
        Commands::Serve { port, host } => {
            init_logging();
            let (state, generated_password) = init_app_state().await?;
            if let Some(pwd) = generated_password {
                println!();
                println!("⚠️  Admin user created with password: {}", pwd);
                println!("   Please change it after login!");
                println!("   You can view this password later with: feishu-bot logs");
                println!();
                tracing::warn!(target: "feishu_bot::secret", "Admin user created with password: {}", pwd);
            }
            run_server(state, &host, port).await?;
        }
        Commands::Admin { port, no_open } => {
            init_logging();
            let (state, generated_password) = init_app_state().await?;
            if let Some(pwd) = generated_password {
                println!();
                println!("⚠️  Admin user created with password: {}", pwd);
                println!("   Please change it after login!");
                println!("   You can view this password later with: feishu-bot logs");
                println!();
                tracing::warn!(target: "feishu_bot::secret", "Admin user created with password: {}", pwd);
            }
            let host = "127.0.0.1";
            let url = format!("http://{}:{}/admin", host, port);

            if !no_open {
                tracing::info!("Opening admin panel in browser: {}", url);
                if let Err(e) = open::that(&url) {
                    tracing::warn!("Failed to open browser: {}", e);
                    println!("Please open {} in your browser", url);
                }
            }

            run_server(state, host, port).await?;
        }
    }

    Ok(())
}

use tracing_subscriber::Layer;

// ...

async fn run_server(state: Arc<AppState>, host: &str, port: u16) -> anyhow::Result<()> {
    // Spawn background task for log cleanup
    let data_dir = state.config.data_dir.clone();
    tokio::spawn(async move {
        // Run cleanup on startup
        if let Err(e) = cleanup_logs(&data_dir) {
            tracing::error!("Failed to clean up logs: {}", e);
        }

        // Run cleanup every 24 hours
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(24 * 3600));
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_logs(&data_dir) {
                tracing::error!("Failed to clean up logs: {}", e);
            }
        }
    });

    // Build API routes with auth middleware
    let api_routes = Router::new()
        // Groups
        .route("/groups", get(handlers::api::groups::list_groups))
        .route("/groups", post(handlers::api::groups::create_group))
        .route("/groups/{id}", get(handlers::api::groups::get_group))
        .route("/groups/{id}", put(handlers::api::groups::update_group))
        .route("/groups/{id}", delete(handlers::api::groups::delete_group))
        .route("/groups/{id}/test", post(handlers::api::groups::test_group))
        // Rules
        .route("/rules", get(handlers::api::rules::list_rules))
        .route("/rules", post(handlers::api::rules::create_rule))
        .route("/rules/{id}", get(handlers::api::rules::get_rule))
        .route("/rules/{id}", put(handlers::api::rules::update_rule))
        .route("/rules/{id}", delete(handlers::api::rules::delete_rule))
        // Alerts
        .route("/alerts", get(handlers::api::alerts::list_alerts))
        .route("/alerts/{id}", get(handlers::api::alerts::get_alert))
        .route(
            "/alerts/{id}/retry",
            post(handlers::api::alerts::retry_alert),
        )
        // Stats
        .route("/stats", get(handlers::api::stats::get_stats))
        // Projects
        .route("/projects", get(handlers::api::projects::list_projects))
        .route("/projects", post(handlers::api::projects::create_project))
        .route("/projects/{id}", get(handlers::api::projects::get_project))
        .route(
            "/projects/{id}",
            put(handlers::api::projects::update_project),
        )
        .route(
            "/projects/{id}",
            delete(handlers::api::projects::delete_project),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            handlers::admin::auth_middleware,
        ));

    // Build main router
    let app = Router::new()
        // Webhook endpoint
        .route(
            "/webhook/sentry",
            post(handlers::webhook::handle_sentry_webhook),
        )
        // Health check
        .route("/health", get(|| async { "OK" }))
        // Login endpoint (no auth required)
        .route("/api/login", post(handlers::admin::login))
        // Change password (with auth)
        .route(
            "/api/change-password",
            post(handlers::admin::change_password),
        )
        // API routes (with auth)
        .nest("/api", api_routes)
        // Admin panel (static files)
        .fallback(handlers::admin::serve_admin)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    // Use localhost for display if binding to 0.0.0.0
    let display_host = if host == "0.0.0.0" {
        "localhost"
    } else {
        &host
    };

    println!();
    println!("🚀 Feishu Bot is running!");
    println!(
        "   Webhook:  http://{}:{}/webhook/sentry",
        display_host, port
    );
    println!("   Admin:    http://{}:{}/admin", display_host, port);
    println!("   Health:   http://{}:{}/health", display_host, port);
    println!();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_logging() {
    let config = Config::from_env().expect("Failed to load config");
    // Remove the old log file definition used for display only
    let log_file = config.data_dir.join("feishu-bot.log");

    // Use daily rolling logs
    let file_appender = tracing_appender::rolling::daily(&config.data_dir, "feishu-bot.log");

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            metadata.target() != "feishu_bot::secret"
        }));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "feishu_bot=debug,tower_http=debug".into()),
        )
        .with(stdout_layer)
        .with(file_layer)
        .init();

    tracing::info!("Log file configured at: {}", log_file.display());
}

fn cleanup_logs(data_dir: &std::path::Path) -> std::io::Result<()> {
    let now = std::time::SystemTime::now();
    let retention_period = std::time::Duration::from_secs(7 * 24 * 3600); // 7 days

    if !data_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check if it's a log file
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("feishu-bot.log") {
                // Check modification time
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > retention_period {
                                tracing::info!("Deleting old log file: {}", path.display());
                                if let Err(e) = fs::remove_file(&path) {
                                    tracing::error!(
                                        "Failed to delete old log file {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn init_app_state() -> anyhow::Result<(Arc<AppState>, Option<String>)> {
    let config = Config::from_env()?;
    let db = db::init_db(&config.database_url).await?;
    let http_client = reqwest::Client::new();

    // Check if admin user exists, create if not
    let generated_password = if db::users::get_user_by_username(&db, "admin")
        .await?
        .is_none()
    {
        let password: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();
        let password_hash = handlers::admin::hash_password(&password);
        db::users::create_user(&db, "admin", &password_hash).await?;
        Some(password)
    } else {
        None
    };

    Ok((
        Arc::new(AppState {
            db,
            config,
            http_client,
        }),
        generated_password,
    ))
}


fn cmd_update() -> anyhow::Result<()> {
    println!("🔄 Checking for updates...");

    // 检查依赖
    check_dependencies()?;

    // Get current version
    let current_version = env!("PKG_VERSION");

    // Get latest version from GitHub API
    let output = std::process::Command::new("curl")
        .args([
            "-s",
            "https://api.github.com/repos/lynn1286/feishu-bot/releases/latest",
        ])
        .output()?;

    let response = String::from_utf8_lossy(&output.stdout);
    if let Some(tag) = response
        .split("\"tag_name\": \"")
        .nth(1)
        .and_then(|s| s.split('"').next())
    {
        let latest_version = tag.trim_start_matches('v');
        if current_version == latest_version {
            println!("✅ Already up to date (v{})", current_version);
            return Ok(());
        }
        println!(
            "📦 New version available: v{} -> v{}",
            current_version, latest_version
        );
    } else {
        println!("⚠️  Unable to check for updates. Proceeding anyway...");
    }

    println!();

    // Get the repo URL and script directory
    let repo_url = "https://github.com/lynn1286/feishu-bot.git";
    let temp_dir = env::temp_dir().join("feishu-bot-update");

    // Clean up temp dir if exists
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    // 创建临时目录守卫，确保清理
    let _guard = TempDirGuard::new(temp_dir.clone());

    // Clone repo
    println!("📥 Cloning repository...");
    std::process::Command::new("git")
        .args(["clone", repo_url, &temp_dir.to_string_lossy()])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    // Build frontend
    println!("🔧 Building frontend...");
    let web_dir = temp_dir.join("web");
    std::process::Command::new("pnpm")
        .args(["install"])
        .current_dir(&web_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    std::process::Command::new("pnpm")
        .args(["run", "build"])
        .current_dir(&web_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    // Build backend
    println!("🔧 Building backend...");
    std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&temp_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    // Install binary
    println!("📦 Installing new binary...");
    std::process::Command::new("cargo")
        .args(["install", "--path", &temp_dir.to_string_lossy()])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    println!();
    println!("✅ Update complete!");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("💡 Quick start commands:");
    println!();
    println!("  feishu-bot serve     # Run server in foreground");
    println!("  feishu-bot admin     # Open admin panel in browser");
    println!("  feishu-bot update    # Update to latest version");
    println!();

    Ok(())
}

fn cmd_uninstall() -> anyhow::Result<()> {
    println!("🗑️  Uninstalling feishu-bot...");
    println!();

    // Get current binary path
    let exe_path = env::current_exe()?;
    println!("Current binary: {}", exe_path.display());

    // Confirm
    print!("Are you sure you want to uninstall? (y/N): ");
    use std::io::Write;
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !input.trim().to_lowercase().starts_with('y') {
        println!("❌ Cancelled");
        return Ok(());
    }

    // Remove data directory
    #[cfg(target_os = "macos")]
    let data_dir = dirs::home_dir().map(|h| h.join("Library/Application Support/feishu-bot"));

    #[cfg(target_os = "linux")]
    let data_dir = dirs::data_dir().map(|d| d.join("feishu-bot"));

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let data_dir = None;

    if let Some(path) = &data_dir {
        if path.exists() {
            print!("Remove data directory ({})? (y/N): ", path.display());
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase().starts_with('y') {
                if fs::remove_dir_all(path).is_err() {
                    println!("⚠️  Failed to remove data directory (permission denied)");
                } else {
                    println!("✅ Removed data: {}", path.display());
                }
            }
        }
    }

    // 删除二进制文件
    if exe_path.exists() {
        if fs::remove_file(&exe_path).is_err() {
            println!("❌ Failed to remove binary: {}", exe_path.display());
        } else {
            println!("✅ Removed binary: {}", exe_path.display());
        }
    }

    println!();
    println!("✅ Uninstall complete!");

    Ok(())
}

fn cmd_logs() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let log_file = config.data_dir.join("feishu-bot.log");

    if !log_file.exists() {
        println!("No logs found. Run 'feishu-bot serve' first.");
        return Ok(());
    }

    println!("📄 Log file: {}", log_file.display());
    println!();

    let content = fs::read_to_string(&log_file)?;
    print!("{}", content);

    Ok(())
}

async fn cmd_reset_password(password: Option<String>) -> anyhow::Result<()> {
    println!("🔐 Resetting admin password...");

    let config = Config::from_env()?;
    let db = db::init_db(&config.database_url).await?;

    // Check if admin user exists
    if db::users::get_user_by_username(&db, "admin")
        .await?
        .is_none()
    {
        println!("❌ Admin user does not exist. Run 'feishu-bot serve' first to create it.");
        return Ok(());
    }

    // Use provided password or generate a random one
    let new_password = password.unwrap_or_else(|| {
        rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(12)
            .map(char::from)
            .collect()
    });

    let password_hash = handlers::admin::hash_password(&new_password);
    db::users::update_password(&db, "admin", &password_hash).await?;

    println!();
    println!("✅ Admin password has been reset!");
    println!("   New password: {}", new_password);
    println!();

    Ok(())
}
