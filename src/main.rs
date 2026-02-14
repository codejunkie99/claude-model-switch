mod commands;
mod config;
mod daemon;
mod orchestrator;
mod proxy;
mod rewrite;

use clap::{Parser, Subcommand};
use config::ProfileConfig;

#[derive(Parser)]
#[command(
    name = "claude-model-switch",
    version,
    about = "Local API proxy for seamless Claude Code model provider switching"
)]
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
    Use { provider: String },
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
        /// Shorthand: `add <name> <api-key>` (preset) or
        /// `add <name> <base-url> <api-key>` (custom).
        input1: Option<String>,
        /// Shorthand API key used with `add <name> <base-url> <api-key>`.
        input2: Option<String>,
        /// Optional for built-in presets (glm, openrouter, minimax).
        #[arg(long)]
        base_url: Option<String>,
        /// Optional model mapping for Claude tiers; provide all three or none.
        #[arg(long)]
        haiku: Option<String>,
        /// Optional model mapping for Claude tiers; provide all three or none.
        #[arg(long)]
        sonnet: Option<String>,
        /// Optional model mapping for Claude tiers; provide all three or none.
        #[arg(long)]
        opus: Option<String>,
        /// Optional API key to save immediately.
        #[arg(long)]
        api_key: Option<String>,
        /// Optional bearer token to save immediately.
        #[arg(long)]
        auth_token: Option<String>,
    },
    /// Remove a provider
    Remove { name: String },
    /// List available providers
    List,
    /// Show current status
    Status,
    /// First-time setup
    Init,
    /// Multi-agent tmux orchestration
    Orchestrate {
        #[command(subcommand)]
        command: OrchestrateCommands,
    },
}

#[derive(Subcommand)]
enum OrchestrateCommands {
    /// Start a multi-pane tmux session with role-specific providers/models
    Start {
        #[arg(long, default_value = "cms-swarm")]
        session: String,
        #[arg(long, default_value = "4000")]
        port: u16,
        #[arg(long, default_value = "trio")]
        preset: String,
        #[arg(long, default_value = ".")]
        cwd: String,
    },
    /// Show pane status for an orchestration session
    Status {
        #[arg(long, default_value = "cms-swarm")]
        session: String,
    },
    /// Stop an orchestration session
    Stop {
        #[arg(long, default_value = "cms-swarm")]
        session: String,
        #[arg(long)]
        stop_proxy: bool,
    },
    /// Send a prompt to a role pane
    Send {
        #[arg(long, default_value = "cms-swarm")]
        session: String,
        role: String,
        prompt: String,
    },
    /// Capture the recent output from a role pane
    Capture {
        #[arg(long, default_value = "cms-swarm")]
        session: String,
        role: String,
        #[arg(long, default_value = "120")]
        lines: u16,
    },
    /// Switch a role pane to another provider/model and relaunch Claude
    Switch {
        #[arg(long, default_value = "cms-swarm")]
        session: String,
        role: String,
        provider: String,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value = "4000")]
        port: u16,
    },
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
        Commands::Setup {
            provider,
            api_key,
            auth_token,
        } => {
            let mut config = ProfileConfig::load()?;
            commands::cmd_setup(&mut config, &provider, api_key, auth_token)
        }
        Commands::Add {
            name,
            input1,
            input2,
            base_url,
            haiku,
            sonnet,
            opus,
            api_key,
            auth_token,
        } => {
            let mut config = ProfileConfig::load()?;
            commands::cmd_add(
                &mut config,
                &name,
                input1,
                input2,
                base_url.as_deref(),
                haiku.as_deref(),
                sonnet.as_deref(),
                opus.as_deref(),
                api_key,
                auth_token,
            )
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
        Commands::Orchestrate { command } => match command {
            OrchestrateCommands::Start {
                session,
                port,
                preset,
                cwd,
            } => {
                let config = ProfileConfig::load()?;
                orchestrator::cmd_orchestrate_start(&config, &session, port, &preset, &cwd)
            }
            OrchestrateCommands::Status { session } => {
                orchestrator::cmd_orchestrate_status(&session)
            }
            OrchestrateCommands::Stop {
                session,
                stop_proxy,
            } => orchestrator::cmd_orchestrate_stop(&session, stop_proxy),
            OrchestrateCommands::Send {
                session,
                role,
                prompt,
            } => orchestrator::cmd_orchestrate_send(&session, &role, &prompt),
            OrchestrateCommands::Capture {
                session,
                role,
                lines,
            } => orchestrator::cmd_orchestrate_capture(&session, &role, lines),
            OrchestrateCommands::Switch {
                session,
                role,
                provider,
                model,
                port,
            } => {
                let config = ProfileConfig::load()?;
                orchestrator::cmd_orchestrate_switch(
                    &config,
                    &session,
                    &role,
                    &provider,
                    model.as_deref(),
                    port,
                )
            }
        },
    }
}
