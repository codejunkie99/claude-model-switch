mod commands;
mod config;
mod daemon;
mod proxy;
mod registry;
mod rewrite;

use clap::{Parser, Subcommand};
use config::ProfileConfig;

#[derive(Parser)]
#[command(name = "claude-model-switch", version, about = "Local API proxy for seamless Claude Code model provider switching")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the proxy server
    Start {
        #[arg(short, long, default_value = "4000")]
        port: u16,
        #[arg(long, hide = true)]
        foreground: bool,
    },
    /// Stop the proxy server
    Stop,
    /// Switch to a provider
    Use {
        provider: String,
    },
    /// Register API credentials for a provider
    Setup {
        provider: String,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        auth_token: Option<String>,
    },
    /// Add a custom provider
    Add {
        name: String,
        #[arg(long)]
        base_url: String,
        #[arg(long)]
        haiku: String,
        #[arg(long)]
        sonnet: String,
        #[arg(long)]
        opus: String,
    },
    /// Remove a provider
    Remove { name: String },
    /// List available providers
    List,
    /// Show current status
    Status,
    /// First-time setup
    Init,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::cmd_init(),
        Commands::List => {
            let config = ProfileConfig::load()?;
            commands::cmd_list(&config)
        }
        Commands::Status => {
            let config = ProfileConfig::load()?;
            commands::cmd_status(&config)
        }
        Commands::Use { provider } => {
            let mut config = ProfileConfig::load()?;
            commands::cmd_use(&mut config, &provider)
        }
        Commands::Setup { provider, api_key, auth_token } => {
            let mut config = ProfileConfig::load()?;
            commands::cmd_setup(&mut config, &provider, api_key, auth_token)
        }
        Commands::Add { name, base_url, haiku, sonnet, opus } => {
            let mut config = ProfileConfig::load()?;
            commands::cmd_add(&mut config, &name, &base_url, &haiku, &sonnet, &opus)
        }
        Commands::Remove { name } => {
            let mut config = ProfileConfig::load()?;
            commands::cmd_remove(&mut config, &name)
        }
        Commands::Start { port, foreground } => {
            if foreground {
                tokio::runtime::Runtime::new()?.block_on(proxy::run_proxy(port))
            } else {
                daemon::start_daemon(port)
            }
        }
        Commands::Stop => daemon::stop_daemon(),
    }
}
