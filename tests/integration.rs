use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin_path() -> String {
    // Build first
    let status = Command::new("cargo")
        .args(["build", "--quiet"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("Failed to build");
    assert!(status.success(), "Build failed");

    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to get metadata");
    let meta: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let target_dir = meta["target_directory"].as_str().unwrap();
    format!("{}/debug/claude-model-switch", target_dir)
}

/// Create a command with HOME set to a temp dir so tests don't depend on
/// the user's real ~/.claude/model-profiles.json.
fn cmd_with_clean_home(bin: &str) -> Command {
    let tmp = std::env::temp_dir().join("claude-model-switch-test");
    std::fs::create_dir_all(&tmp).unwrap();
    let mut cmd = Command::new(bin);
    cmd.env("HOME", &tmp);
    cmd
}

fn unique_home(label: &str) -> std::path::PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "claude-model-switch-test-{}-{}-{}",
        label,
        std::process::id(),
        ts
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_help() {
    let bin = bin_path();
    let output = Command::new(&bin).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Local API proxy"));
}

#[test]
fn test_version() {
    let bin = bin_path();
    let output = Command::new(&bin).arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("claude-model-switch"));
}

#[test]
fn test_list() {
    let bin = bin_path();
    let output = cmd_with_clean_home(&bin).arg("list").output().unwrap();
    // Should succeed (loads default config if none exists)
    assert!(output.status.success());
}

#[test]
fn test_status() {
    let bin = bin_path();
    let output = cmd_with_clean_home(&bin).arg("status").output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_add_glm_with_api_key_without_base_url() {
    let bin = bin_path();
    let home = unique_home("add-glm");

    let output = Command::new(&bin)
        .env("HOME", &home)
        .args(["add", "glm", "sk-test"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);

    let config_path = home.join(".claude").join("model-profiles.json");
    let config_raw = std::fs::read_to_string(config_path).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_raw).unwrap();
    assert_eq!(
        config["providers"]["glm"]["base_url"].as_str(),
        Some("https://open.z.ai/api/paas/v4")
    );
    assert_eq!(
        config["providers"]["glm"]["api_key"].as_str(),
        Some("sk-test")
    );
}

#[test]
fn test_add_custom_with_positional_base_url_and_api_key() {
    let bin = bin_path();
    let home = unique_home("add-custom-positional");

    let output = Command::new(&bin)
        .env("HOME", &home)
        .args(["add", "acme", "https://api.acme.ai/v1", "sk-custom"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);

    let config_path = home.join(".claude").join("model-profiles.json");
    let config_raw = std::fs::read_to_string(config_path).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_raw).unwrap();
    assert_eq!(
        config["providers"]["acme"]["base_url"].as_str(),
        Some("https://api.acme.ai/v1")
    );
    assert_eq!(
        config["providers"]["acme"]["api_key"].as_str(),
        Some("sk-custom")
    );
}

#[test]
fn test_reuse_existing_base_url_with_short_update() {
    let bin = bin_path();
    let home = unique_home("reuse-base-url");

    let add_initial = Command::new(&bin)
        .env("HOME", &home)
        .args(["add", "acme", "https://api.acme.ai/v1", "sk-old"])
        .output()
        .unwrap();
    assert!(add_initial.status.success(), "{:?}", add_initial);

    let update_key = Command::new(&bin)
        .env("HOME", &home)
        .args(["add", "acme", "sk-new"])
        .output()
        .unwrap();
    assert!(update_key.status.success(), "{:?}", update_key);

    let config_path = home.join(".claude").join("model-profiles.json");
    let config_raw = std::fs::read_to_string(config_path).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_raw).unwrap();
    assert_eq!(
        config["providers"]["acme"]["base_url"].as_str(),
        Some("https://api.acme.ai/v1")
    );
    assert_eq!(
        config["providers"]["acme"]["api_key"].as_str(),
        Some("sk-new")
    );
}

#[test]
fn test_add_unknown_without_base_url_fails() {
    let bin = bin_path();
    let home = unique_home("add-unknown");

    let output = Command::new(&bin)
        .env("HOME", &home)
        .args(["add", "acme", "--api-key", "sk-test"])
        .output()
        .unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Missing base URL"));
}
