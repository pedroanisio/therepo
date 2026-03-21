use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn repo_bin() -> &'static str {
    env!("CARGO_BIN_EXE_repo")
}

fn temp_repo(name: &str) -> PathBuf {
    let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("therepo-{name}-{nanos}-{unique}"));
    std::fs::create_dir_all(path.join(".git")).unwrap();
    path
}

fn run_repo(repo_root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(repo_bin())
        .args(args)
        .current_dir(repo_root)
        .output()
        .unwrap()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn version_prints_package_version() {
    let output = Command::new(repo_bin()).arg("--version").output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        stdout(&output).trim(),
        format!("repo {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_prints_top_level_usage() {
    let output = Command::new(repo_bin()).arg("--help").output().unwrap();

    assert!(output.status.success());
    let help = stdout(&output);
    assert!(help.contains("Repository maintenance CLI"));
    assert!(help.contains("Usage:"));
    assert!(help.contains("plugins"));
    assert!(help.contains("Examples:"));
}

#[test]
fn overview_creates_repo_storage_and_reports_missing_config() {
    let repo_root = temp_repo("overview");
    let output = run_repo(&repo_root, &[]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(repo_root.join(".repo").join("storage").join(".keep").is_file());

    let text = stdout(&output);
    assert!(text.contains("plugins"));
    assert!(text.contains("no .repo/config.toml found"));

    std::fs::remove_dir_all(repo_root).ok();
}

#[test]
fn plugins_lists_builtin_commands() {
    let repo_root = temp_repo("plugins");
    let output = run_repo(&repo_root, &["plugins"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("docs"));
    assert!(text.contains("prompt"));
    assert!(text.contains("ulid"));
    assert!(text.contains("plugin(s)"));

    std::fs::remove_dir_all(repo_root).ok();
}

#[test]
fn overview_json_emits_machine_readable_summary() {
    let repo_root = temp_repo("overview-json");
    let output = run_repo(&repo_root, &["--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(value["name"].is_string());
    assert!(value["builtin_plugins"].is_number());
    assert!(value["external_plugins"].is_number());
    assert_eq!(value["config_present"], false);

    std::fs::remove_dir_all(repo_root).ok();
}

#[test]
fn plugins_json_lists_discovered_plugins() {
    let repo_root = temp_repo("plugins-json");
    let output = run_repo(&repo_root, &["plugins", "--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let plugins = value["plugins"].as_array().unwrap();
    assert!(plugins.iter().any(|plugin| plugin["name"] == "docs"));
    assert!(plugins.iter().any(|plugin| plugin["name"] == "prompt"));

    std::fs::remove_dir_all(repo_root).ok();
}

#[test]
fn unknown_command_returns_non_zero_with_guidance() {
    let repo_root = temp_repo("unknown");
    let output = run_repo(&repo_root, &["does-not-exist"]);

    assert!(!output.status.success());
    let text = stderr(&output);
    assert!(text.contains("Unknown command: does-not-exist"));
    assert!(text.contains("repo --help"));

    std::fs::remove_dir_all(repo_root).ok();
}

#[test]
fn plugins_info_shows_plugin_details() {
    let repo_root = temp_repo("plugins-info");
    let output = run_repo(&repo_root, &["plugins", "info", "docs"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("docs"), "expected plugin name in output: {text}");
    assert!(text.contains("built-in"), "expected type in output: {text}");
    assert!(text.contains("version:"), "expected version field: {text}");

    std::fs::remove_dir_all(repo_root).ok();
}

#[test]
fn plugins_info_json_emits_machine_readable_plugin() {
    let repo_root = temp_repo("plugins-info-json");
    let output = run_repo(&repo_root, &["plugins", "info", "docs", "--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["name"], "docs");
    assert!(value["builtin"].as_bool().unwrap_or(false));
    assert!(value["capabilities"].is_array());

    std::fs::remove_dir_all(repo_root).ok();
}

#[test]
fn plugins_info_unknown_name_returns_non_zero() {
    let repo_root = temp_repo("plugins-info-unknown");
    let output = run_repo(&repo_root, &["plugins", "info", "no-such-plugin"]);

    assert!(!output.status.success());
    let text = stderr(&output);
    assert!(
        text.contains("Unknown plugin") || text.contains("no-such-plugin"),
        "expected error in stderr: {text}"
    );

    std::fs::remove_dir_all(repo_root).ok();
}

#[test]
fn external_plugin_without_dispatch_returns_non_zero() {
    let repo_root = temp_repo("external-plugin");
    let plugin_dir = repo_root.join(".repo").join("plugins").join("demo");
    std::fs::create_dir_all(&plugin_dir).unwrap();
    std::fs::write(
        plugin_dir.join("plugin.toml"),
        "[plugin]\nname = \"demo\"\nversion = \"1.0.0\"\ndescription = \"Demo plugin\"\nprovides = [\"command\"]\n",
    )
    .unwrap();

    let output = run_repo(&repo_root, &["demo"]);

    assert!(!output.status.success());
    let text = stderr(&output);
    assert!(text.contains("External plugin dispatch not yet implemented."));
    assert!(text.contains("Plugin 'demo' was found but cannot be run yet."));

    std::fs::remove_dir_all(repo_root).ok();
}
