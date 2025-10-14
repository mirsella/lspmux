use std::env;

use anyhow::Result;
use clap::{Parser, Subcommand};
use lspmux::config::Config;
use lspmux::{ext, proxy, server};
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// No command defaults to client
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Connect to an lspmux server [default]
    Client {
        /// Path to the LSP server executable
        #[arg(
            long = "server-path",
            env = "LSPMUX_SERVER",
            default_value = "rust-analyzer",
            name = "SERVER_PATH"
        )]
        server: String,

        /// Arguments passed to the LSP server
        #[arg(name = "SERVER_ARGS")]
        args: Vec<String>,
    },

    /// Start a lpsmux server
    Server {},

    /// Print server status
    Status {
        /// Output data as machine readable JSON
        #[clap(long = "json", default_value = "false")]
        json: bool,

        /// Output more information
        #[clap(short = 'v', long = "verbose", default_value = "false")]
        verbose: bool,
    },

    /// Print server configuration
    Config {},

    /// Reload workspace
    ///
    /// For rust-analyzer send the `rust-analyzer/reloadWorkspace` extension request.
    /// Do nothing for other language servers.
    Reload {},
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = match Config::try_load() {
        Ok(config) => {
            config.init_logger();
            config
        }
        Err(err) => {
            let config = Config::default();
            config.init_logger();
            // Log only after the logger has been initialized
            info!(?err, "cannot load config file, continuing with defaults");
            config
        }
    };

    match cli.command {
        Some(Cmd::Server {}) => server::run(&config).await,
        Some(Cmd::Client { server, args }) => proxy::run(&config, server, args).await,
        Some(Cmd::Status { json, verbose }) => ext::status(&config, json, verbose).await,
        Some(Cmd::Config {}) => ext::config(&config),
        Some(Cmd::Reload {}) => ext::reload(&config).await,
        None => {
            let server_path = env::var("LSPMUX_SERVER").unwrap_or_else(|_| "rust-analyzer".into());
            proxy::run(&config, server_path, vec![]).await
        }
    }
}
