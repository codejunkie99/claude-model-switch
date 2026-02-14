use anyhow::{bail, Context, Result};
use std::process::Command;

use crate::config::ProfileConfig;
use crate::daemon;

#[derive(Clone, Debug)]
struct RolePreset {
    name: &'static str,
    provider: String,
    model: &'static str,
}

#[derive(Debug)]
struct PaneInfo {
    index: usize,
    title: String,
    current_command: String,
}

fn roles_for_preset(preset: &str, config: &ProfileConfig) -> Result<Vec<RolePreset>> {
    let mut providers: Vec<&str> = config.providers.keys().map(|s| s.as_str()).collect();
    providers.sort();

    match preset {
        "trio" => {
            if providers.len() < 3 {
                bail!(
                    "Preset 'trio' requires at least 3 configured providers, found {}.\nAdd more with: claude-model-switch add <name> --base-url <url> [--haiku <m> --sonnet <m> --opus <m>]",
                    providers.len()
                );
            }
            Ok(vec![
                RolePreset {
                    name: "planner",
                    provider: providers[0].to_string(),
                    model: "sonnet",
                },
                RolePreset {
                    name: "coder",
                    provider: providers[1].to_string(),
                    model: "opus",
                },
                RolePreset {
                    name: "reviewer",
                    provider: providers[2].to_string(),
                    model: "sonnet",
                },
            ])
        }
        "duo" => {
            if providers.len() < 2 {
                bail!(
                    "Preset 'duo' requires at least 2 configured providers, found {}.\nAdd more with: claude-model-switch add <name> --base-url <url> [--haiku <m> --sonnet <m> --opus <m>]",
                    providers.len()
                );
            }
            Ok(vec![
                RolePreset {
                    name: "planner",
                    provider: providers[0].to_string(),
                    model: "sonnet",
                },
                RolePreset {
                    name: "coder",
                    provider: providers[1].to_string(),
                    model: "opus",
                },
            ])
        }
        _ => bail!("Unknown preset '{}'. Supported presets: trio, duo", preset),
    }
}

fn ensure_safe_token(value: &str, field: &str) -> Result<()> {
    if value.is_empty() {
        bail!("{} cannot be empty", field);
    }
    let ok = value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if !ok {
        bail!(
            "{} contains unsupported characters. Allowed: letters, numbers, '-', '_' and '.'",
            field
        );
    }
    Ok(())
}

fn run_tmux(args: &[String]) -> Result<String> {
    let output = Command::new("tmux")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run tmux {:?}", args))?;

    if !output.status.success() {
        bail!(
            "tmux {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn ensure_tmux_available() -> Result<()> {
    let output = Command::new("tmux").arg("-V").output();
    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => bail!(
            "tmux is installed but unavailable: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ),
        Err(e) => bail!("tmux is required for orchestration: {}", e),
    }
}

fn session_exists(session: &str) -> Result<bool> {
    let output = Command::new("tmux")
        .args(["has-session", "-t", session])
        .output()
        .with_context(|| format!("Failed to check tmux session '{}'", session))?;
    Ok(output.status.success())
}

fn pane_target(session: &str, index: usize) -> String {
    format!("{}:0.{}", session, index)
}

fn launch_command(port: u16, provider: &str, model: Option<&str>) -> String {
    match model {
        Some(model) => format!(
            "export ANTHROPIC_BASE_URL=http://127.0.0.1:{}/p/{}/v1; claude --model {}",
            port, provider, model
        ),
        None => format!(
            "export ANTHROPIC_BASE_URL=http://127.0.0.1:{}/p/{}/v1; claude",
            port, provider
        ),
    }
}

fn send_keys(target: &str, text: &str) -> Result<()> {
    let args = vec![
        "send-keys".to_string(),
        "-t".to_string(),
        target.to_string(),
        text.to_string(),
        "C-m".to_string(),
    ];
    run_tmux(&args)?;
    Ok(())
}

fn list_panes(session: &str) -> Result<Vec<PaneInfo>> {
    let args = vec![
        "list-panes".to_string(),
        "-t".to_string(),
        format!("{}:0", session),
        "-F".to_string(),
        "#{pane_index}\t#{pane_title}\t#{pane_current_command}".to_string(),
    ];
    let output = run_tmux(&args)?;
    let mut panes = Vec::new();
    for line in output.lines() {
        let mut parts = line.splitn(3, '\t');
        let idx = parts
            .next()
            .and_then(|v| v.parse::<usize>().ok())
            .with_context(|| format!("Unexpected tmux pane entry '{}'", line))?;
        let title = parts.next().unwrap_or_default().to_string();
        let current_command = parts.next().unwrap_or_default().to_string();
        panes.push(PaneInfo {
            index: idx,
            title,
            current_command,
        });
    }
    Ok(panes)
}

fn pane_index_for_role(session: &str, role: &str) -> Result<usize> {
    let panes = list_panes(session)?;
    panes
        .into_iter()
        .find(|pane| pane.title == role)
        .map(|pane| pane.index)
        .with_context(|| format!("Role '{}' not found in tmux session '{}'", role, session))
}

pub fn cmd_orchestrate_start(
    config: &ProfileConfig,
    session: &str,
    port: u16,
    preset: &str,
    cwd: &str,
) -> Result<()> {
    ensure_safe_token(session, "session")?;
    ensure_safe_token(preset, "preset")?;
    ensure_tmux_available()?;

    if session_exists(session)? {
        bail!("tmux session '{}' already exists", session);
    }

    let roles = roles_for_preset(preset, config)?;
    for role in &roles {
        config.provider(&role.provider)?;
    }

    match daemon::start_daemon(port) {
        Ok(()) => {}
        Err(e) => {
            let msg = format!("{:#}", e);
            if !msg.contains("already running") {
                return Err(e);
            }
            eprintln!("Proxy already running, reusing existing process.");
        }
    }

    let new_args = vec![
        "new-session".to_string(),
        "-d".to_string(),
        "-s".to_string(),
        session.to_string(),
        "-n".to_string(),
        "swarm".to_string(),
        "-c".to_string(),
        cwd.to_string(),
    ];
    run_tmux(&new_args)?;

    for _ in 1..roles.len() {
        let split_args = vec![
            "split-window".to_string(),
            "-t".to_string(),
            format!("{}:0", session),
            "-v".to_string(),
            "-c".to_string(),
            cwd.to_string(),
        ];
        run_tmux(&split_args)?;
    }

    run_tmux(&[
        "select-layout".to_string(),
        "-t".to_string(),
        format!("{}:0", session),
        "tiled".to_string(),
    ])?;

    for (idx, role) in roles.iter().enumerate() {
        let target = pane_target(session, idx);
        run_tmux(&[
            "select-pane".to_string(),
            "-t".to_string(),
            target.clone(),
            "-T".to_string(),
            role.name.to_string(),
        ])?;
        let cmd = launch_command(port, &role.provider, Some(role.model));
        send_keys(&target, &cmd)?;
    }

    println!(
        "Started tmux orchestration session '{}'. Attach with: tmux attach -t {}",
        session, session
    );
    println!("Role routing:");
    for role in roles {
        println!(
            "  {} -> provider={} model={}",
            role.name, role.provider, role.model
        );
    }
    Ok(())
}

pub fn cmd_orchestrate_status(session: &str) -> Result<()> {
    ensure_safe_token(session, "session")?;
    ensure_tmux_available()?;

    if !session_exists(session)? {
        println!("Session '{}' is not running.", session);
        return Ok(());
    }

    let panes = list_panes(session)?;
    println!("Session '{}' panes:", session);
    for pane in panes {
        println!(
            "  pane={} role={} cmd={}",
            pane.index, pane.title, pane.current_command
        );
    }
    Ok(())
}

pub fn cmd_orchestrate_send(session: &str, role: &str, prompt: &str) -> Result<()> {
    ensure_safe_token(session, "session")?;
    ensure_safe_token(role, "role")?;
    ensure_tmux_available()?;

    if !session_exists(session)? {
        bail!("Session '{}' is not running", session);
    }

    let idx = pane_index_for_role(session, role)?;
    let target = pane_target(session, idx);
    send_keys(&target, prompt)?;
    println!("Sent prompt to role '{}' ({})", role, target);
    Ok(())
}

pub fn cmd_orchestrate_capture(session: &str, role: &str, lines: u16) -> Result<()> {
    ensure_safe_token(session, "session")?;
    ensure_safe_token(role, "role")?;
    ensure_tmux_available()?;

    if !session_exists(session)? {
        bail!("Session '{}' is not running", session);
    }

    let idx = pane_index_for_role(session, role)?;
    let target = pane_target(session, idx);
    let args = vec![
        "capture-pane".to_string(),
        "-p".to_string(),
        "-t".to_string(),
        target,
        "-S".to_string(),
        format!("-{}", lines),
    ];
    let output = run_tmux(&args)?;
    print!("{}", output);
    Ok(())
}

pub fn cmd_orchestrate_switch(
    config: &ProfileConfig,
    session: &str,
    role: &str,
    provider: &str,
    model: Option<&str>,
    port: u16,
) -> Result<()> {
    ensure_safe_token(session, "session")?;
    ensure_safe_token(role, "role")?;
    ensure_safe_token(provider, "provider")?;
    if let Some(model) = model {
        ensure_safe_token(model, "model")?;
    }
    ensure_tmux_available()?;
    config.provider(provider)?;

    if !session_exists(session)? {
        bail!("Session '{}' is not running", session);
    }

    let idx = pane_index_for_role(session, role)?;
    let target = pane_target(session, idx);

    run_tmux(&[
        "send-keys".to_string(),
        "-t".to_string(),
        target.clone(),
        "C-c".to_string(),
    ])?;

    let cmd = launch_command(port, provider, model);
    send_keys(&target, &cmd)?;
    println!(
        "Switched role '{}' to provider='{}' model='{}'",
        role,
        provider,
        model.unwrap_or("session-default")
    );
    Ok(())
}

pub fn cmd_orchestrate_stop(session: &str, stop_proxy: bool) -> Result<()> {
    ensure_safe_token(session, "session")?;
    ensure_tmux_available()?;

    if session_exists(session)? {
        run_tmux(&[
            "kill-session".to_string(),
            "-t".to_string(),
            session.to_string(),
        ])?;
        println!("Stopped tmux session '{}'.", session);
    } else {
        println!("Session '{}' is not running.", session);
    }

    if stop_proxy {
        match daemon::stop_daemon() {
            Ok(()) => {}
            Err(e) => eprintln!("Proxy stop skipped: {:#}", e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_command_has_profile_route() {
        let cmd = launch_command(4000, "glm-5", Some("opus"));
        assert!(cmd.contains("http://127.0.0.1:4000/p/glm-5/v1"));
        assert!(cmd.contains("--model opus"));
    }

    #[test]
    fn token_validation() {
        assert!(ensure_safe_token("good-name_1", "x").is_ok());
        assert!(ensure_safe_token("bad name", "x").is_err());
    }
}
