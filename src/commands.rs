use crate::config::{ModelMapping, ProfileConfig, Provider};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

pub fn cmd_list(config: &ProfileConfig) -> Result<()> {
    println!("Available providers:");
    let mut names: Vec<_> = config.providers.keys().collect();
    names.sort();
    for name in names {
        let marker = if *name == config.active {
            " (active)"
        } else {
            ""
        };
        let provider = &config.providers[name];
        let models = match &provider.models {
            Some(m) => format!("{} / {} / {}", m.haiku, m.sonnet, m.opus),
            None => "(passthrough)".to_string(),
        };
        println!("  {}{} - {} [{}]", name, marker, provider.base_url, models);
    }
    Ok(())
}

pub fn cmd_status(config: &ProfileConfig) -> Result<()> {
    let provider = config.active_provider()?;
    println!("Active provider: {}", config.active);
    println!("Base URL: {}", provider.base_url);
    match &provider.models {
        Some(m) => {
            println!("Haiku  -> {}", m.haiku);
            println!("Sonnet -> {}", m.sonnet);
            println!("Opus   -> {}", m.opus);
        }
        None => println!("Models: passthrough (no rewriting)"),
    }
    let pid_path = pid_file_path()?;
    if pid_path.exists() {
        let pid = std::fs::read_to_string(&pid_path)?;
        println!("Proxy: running (PID {})", pid.trim());
    } else {
        println!("Proxy: not running");
    }
    Ok(())
}

pub fn cmd_use(config: &mut ProfileConfig, provider: &str) -> Result<()> {
    if !config.providers.contains_key(provider) {
        bail!(
            "Unknown provider '{}'. Run 'claude-model-switch list' to see available providers.\nTo add a new provider: claude-model-switch add {} --base-url <URL> --haiku <model> --sonnet <model> --opus <model>",
            provider, provider
        );
    }
    config.active = provider.to_string();
    config.save()?;
    println!("Switched to: {}", provider);

    let pid_path = pid_file_path()?;
    if pid_path.exists() {
        let pid_str = std::fs::read_to_string(&pid_path)?;
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            #[cfg(unix)]
            unsafe {
                libc::kill(pid, libc::SIGHUP);
            }
            println!("Proxy notified to reload configuration.");
        }
    } else {
        println!("Note: Proxy is not running. Start it with: claude-model-switch start");
    }
    Ok(())
}

pub fn cmd_setup(
    config: &mut ProfileConfig,
    provider: &str,
    api_key: Option<String>,
    auth_token: Option<String>,
) -> Result<()> {
    if api_key.is_none() && auth_token.is_none() {
        bail!("Provide --api-key or --auth-token");
    }
    if let Some(existing) = config.providers.get_mut(provider) {
        if let Some(key) = api_key {
            existing.api_key = Some(key);
        }
        if let Some(token) = auth_token {
            existing.auth_token = Some(token);
        }
    } else {
        bail!(
            "Unknown provider '{}'. Add it first with: claude-model-switch add {} --base-url <URL> --haiku <model> --sonnet <model> --opus <model>",
            provider, provider
        );
    }
    config.save()?;
    println!("Credentials saved for '{}'.", provider);
    Ok(())
}

pub fn cmd_add(
    config: &mut ProfileConfig,
    name: &str,
    base_url: &str,
    haiku: &str,
    sonnet: &str,
    opus: &str,
) -> Result<()> {
    config.providers.insert(
        name.to_string(),
        Provider {
            base_url: base_url.to_string(),
            api_key: None,
            auth_token: None,
            models: Some(ModelMapping {
                haiku: haiku.to_string(),
                sonnet: sonnet.to_string(),
                opus: opus.to_string(),
            }),
        },
    );
    config.save()?;
    println!("Added provider '{}'.", name);
    println!(
        "Now run: claude-model-switch setup {} --api-key <YOUR_KEY>",
        name
    );
    Ok(())
}

pub fn cmd_remove(config: &mut ProfileConfig, name: &str) -> Result<()> {
    if name == "claude" {
        bail!("Cannot remove the default 'claude' provider.");
    }
    if config.providers.remove(name).is_none() {
        bail!("Provider '{}' not found.", name);
    }
    if config.active == name {
        config.active = "claude".to_string();
        println!("Active provider was '{}', switched back to 'claude'.", name);
    }
    config.save()?;
    println!("Removed provider '{}'.", name);
    Ok(())
}

pub fn cmd_init() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let settings_path = home.join(".claude").join("settings.json");

    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    let env = settings
        .as_object_mut()
        .context("settings.json is not an object")?
        .entry("env")
        .or_insert(serde_json::json!({}));

    env.as_object_mut().context("env is not an object")?.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        serde_json::Value::String("http://localhost:4000/v1".to_string()),
    );

    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;

    let config = ProfileConfig::default();
    config.save()?;

    println!("Initialized claude-model-switch!");
    println!(
        "  - Set ANTHROPIC_BASE_URL=http://localhost:4000/v1 in {}",
        settings_path.display()
    );
    println!("  - Created default profile config");
    println!();
    println!("Next steps:");
    println!("  1. claude-model-switch setup <provider> --api-key <key>");
    println!("  2. claude-model-switch start");
    println!("  3. claude-model-switch use <provider>");
    Ok(())
}

pub fn pid_file_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".claude").join("model-switch-proxy.pid"))
}
