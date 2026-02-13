use std::process::Command;

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
