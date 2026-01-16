use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "feishu-bot")]
#[command(about = "Sentry to Feishu alert forwarding service", long_about = None)]
#[command(version = env!("PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the server in foreground (for development)
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
    },

    /// Open the admin panel in browser
    Admin {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Don't open browser automatically
        #[arg(long)]
        no_open: bool,
    },

    /// Update to the latest version
    Update,

    /// Uninstall feishu-bot
    Uninstall,

    /// View application logs
    Logs,

    /// Reset admin password
    ResetPassword {
        /// New password (if not provided, a random password will be generated)
        #[arg(short, long)]
        password: Option<String>,
    },
}
