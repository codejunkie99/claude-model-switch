use crate::config::{ModelMapping, ProfileConfig, Provider};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

#[derive(Clone, Copy)]
struct BuiltinProviderPreset {
    base_url: &'static str,
}

fn builtin_provider_preset(name: &str) -> Option<BuiltinProviderPreset> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "glm" => Some(BuiltinProviderPreset {
            base_url: "https://open.z.ai/api/paas/v4",
        }),
        "openrouter" => Some(BuiltinProviderPreset {
            base_url: "https://openrouter.ai/api/v1",
        }),
        "minimax" => Some(BuiltinProviderPreset {
            base_url: "https://api.minimax.io/anthropic/v1",
        }),
        _ => None,
    }
}

fn parse_credential(credential: String) -> Result<(Option<String>, Option<String>)> {
    let lower = credential.to_ascii_lowercase();
    if lower.starts_with("bearer:") {
        let token = credential
            .split_once(':')
            .map(|(_, t)| t)
            .unwrap_or_default()
            .trim()
            .to_string();
        if token.is_empty() {
            bail!("Bearer credential cannot be empty. Use: add <name> bearer:<token>");
        }
        Ok((None, Some(token)))
    } else {
        Ok((Some(credential), None))
    }
}

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
            "Unknown provider '{}'. Run 'claude-model-switch list' to see available providers.\nTo add a new provider: claude-model-switch add {} <base-url> <api-key>\nOr for built-in presets: claude-model-switch add {} <api-key>",
            provider, provider, provider
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
            "Unknown provider '{}'. Add it first with: claude-model-switch add {} <base-url> <api-key>\nOr for built-in presets: claude-model-switch add {} <api-key>",
            provider, provider, provider
        );
    }
    config.save()?;
    println!("Credentials saved for '{}'.", provider);
    Ok(())
}

pub fn cmd_add(
    config: &mut ProfileConfig,
    name: &str,
    input1: Option<String>,
    input2: Option<String>,
    base_url: Option<&str>,
    haiku: Option<&str>,
    sonnet: Option<&str>,
    opus: Option<&str>,
    api_key: Option<String>,
    auth_token: Option<String>,
) -> Result<()> {
    let mut base_url_reused_from_existing = false;
    let mut base_url_from_preset: Option<&'static str> = None;

    let (positional_base_url, positional_credential) = match (input1, input2) {
        (Some(base), Some(credential)) => (Some(base), Some(credential)),
        (Some(value), None) => {
            if value.starts_with("http://") || value.starts_with("https://") {
                (Some(value), None)
            } else {
                (None, Some(value))
            }
        }
        (None, Some(_)) => unreachable!("clap positional parsing guarantees first arg exists"),
        (None, None) => (None, None),
    };

    if positional_base_url.is_some() && base_url.is_some() {
        bail!(
            "Provide base URL either positionally (`add <name> <base-url> <api-key>`) or with --base-url, not both"
        );
    }

    let (api_key, auth_token) = match positional_credential {
        Some(credential) => {
            if api_key.is_some() || auth_token.is_some() {
                bail!(
                    "Use either positional credential (`add <name> [<base-url>] <credential>`) or --api-key/--auth-token flags, not both"
                );
            }
            parse_credential(credential)?
        }
        None => (api_key, auth_token),
    };

    let existing = config.providers.get(name).cloned();
    let preset = builtin_provider_preset(name);
    let resolved_base_url = match base_url.or(positional_base_url.as_deref()) {
        Some(url) => url.to_string(),
        None => {
            if let Some(existing_provider) = &existing {
                base_url_reused_from_existing = true;
                existing_provider.base_url.clone()
            } else if let Some(preset) = preset {
                base_url_from_preset = Some(preset.base_url);
                preset.base_url.to_string()
            } else {
                bail!(
                    "Missing base URL for provider '{}'. Use: claude-model-switch add {} <base-url> <api-key>\nOr for built-in presets: claude-model-switch add {} <api-key>",
                    name, name, name
                );
            }
        }
    };

    let models = match (haiku, sonnet, opus, existing.as_ref()) {
        (None, None, None, Some(existing_provider)) => existing_provider.models.clone(),
        (None, None, None, None) => None,
        (Some(haiku), Some(sonnet), Some(opus), _) => Some(ModelMapping {
            haiku: haiku.to_string(),
            sonnet: sonnet.to_string(),
            opus: opus.to_string(),
        }),
        _ => {
            bail!(
                "If you provide model mappings, pass all three flags: --haiku <model> --sonnet <model> --opus <model>"
            )
        }
    };

    let resolved_api_key = match api_key {
        Some(key) => Some(key),
        None => existing.as_ref().and_then(|p| p.api_key.clone()),
    };
    let resolved_auth_token = match auth_token {
        Some(token) => Some(token),
        None => existing.as_ref().and_then(|p| p.auth_token.clone()),
    };

    let provider_existed = existing.is_some();
    let has_model_mapping = models.is_some();

    config.providers.insert(
        name.to_string(),
        Provider {
            base_url: resolved_base_url.clone(),
            api_key: resolved_api_key,
            auth_token: resolved_auth_token,
            models,
        },
    );
    config.save()?;
    if provider_existed {
        println!("Updated provider '{}'.", name);
    } else {
        println!("Added provider '{}'.", name);
    }
    if base_url_reused_from_existing {
        println!(
            "Base URL reused from existing provider: {}",
            resolved_base_url
        );
    } else if let Some(preset_url) = base_url_from_preset {
        println!(
            "Base URL preset applied: {} -> {}",
            name.to_ascii_lowercase(),
            preset_url
        );
    }
    if has_model_mapping {
        println!("Model rewriting: enabled for Claude tiers (haiku/sonnet/opus).");
    } else {
        println!("Model rewriting: passthrough (all model IDs forwarded as-is).");
    }
    if config
        .providers
        .get(name)
        .map(|p| p.api_key.is_some() || p.auth_token.is_some())
        .unwrap_or(false)
    {
        println!("Credentials saved for '{}'.", name);
    } else {
        println!(
            "Now run: claude-model-switch setup {} --api-key <YOUR_KEY>",
            name
        );
    }
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
