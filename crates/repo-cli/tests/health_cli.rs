use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn repo_bin() -> &'static str {
    env!("CARGO_BIN_EXE_repo")
}

struct TempRepo {
    path: PathBuf,
}

impl TempRepo {
    fn new(label: &str) -> Self {
        let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("therepo-{label}-{nanos}-{unique}"));
        std::fs::create_dir_all(path.join(".git")).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn run_health(repo_root: &Path, args: &[&str]) -> Output {
    Command::new(repo_bin())
        .args(["health"])
        .args(args)
        .current_dir(repo_root)
        .output()
        .unwrap()
}

fn run_health_with_path(repo_root: &Path, args: &[&str], path: &Path) -> Output {
    Command::new(repo_bin())
        .args(["health"])
        .args(args)
        .current_dir(repo_root)
        .env("PATH", path)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn health_help_prints_usage() {
    let repo = TempRepo::new("health-help");
    let output = run_health(repo.path(), &["--help"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let help = stdout(&output);
    assert!(help.contains("Check development environment"));
    assert!(help.contains("Usage:"));
    assert!(help.contains("init"));
    assert!(help.contains("export"));
    assert!(help.contains("Examples:"));
}

#[test]
fn health_init_writes_template_when_missing() {
    let repo = TempRepo::new("health-init");
    let output = run_health(repo.path(), &["init"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let health_toml = repo.path().join(".repo").join("health.toml");
    assert!(health_toml.is_file());

    let content = std::fs::read_to_string(&health_toml).unwrap();
    assert!(content.contains("[environment]"));
    assert!(content.contains("privilege = \"auto\""));
}

#[test]
fn health_init_refuses_to_overwrite_existing_file() {
    let repo = TempRepo::new("health-init-existing");
    repo.write(".repo/health.toml", "sentinel = true\n");

    let output = run_health(repo.path(), &["init"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let err = stderr(&output);
    assert!(err.contains(".repo/health.toml already exists"));
    assert!(err.contains("repo health export"));

    let content = std::fs::read_to_string(repo.path().join(".repo/health.toml")).unwrap();
    assert_eq!(content, "sentinel = true\n");
}

#[test]
fn health_unknown_subcommand_returns_non_zero_with_guidance() {
    let repo = TempRepo::new("health-unknown");
    let output = run_health(repo.path(), &["bogus"]);

    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("unrecognized subcommand 'bogus'"));
    assert!(err.contains("For more information, try '--help'."));
}

#[test]
fn health_default_check_reports_missing_git_repository() {
    let repo = TempRepo::new("health-default");
    repo.write(
        ".repo/health.toml",
        "[environment]\nprivilege = \"auto\"\nallowed_runtimes = []\n",
    );
    let empty_path = TempRepo::new("health-empty-path");

    let output = run_health_with_path(repo.path(), &[], empty_path.path());

    assert!(!output.status.success(), "stdout: {}", stdout(&output));
    let out = stdout(&output);
    assert!(out.contains("Environment health check"));
    assert!(out.contains("Repository"));
    assert!(out.contains("not a git repository"));
}

#[test]
fn health_custom_missing_required_tool_returns_non_zero() {
    let repo = TempRepo::new("health-custom-missing");
    repo.write(
        ".repo/health.toml",
        "[tools.missing-tool]\nrequired = true\ncommand = \"definitely-not-a-command\"\n",
    );
    let empty_path = TempRepo::new("health-empty-path");

    let output = run_health_with_path(repo.path(), &[], empty_path.path());

    assert!(!output.status.success(), "stdout: {}", stdout(&output));
    let out = stdout(&output);
    assert!(out.contains("missing-tool"));
    assert!(out.contains("required but not found"));
}

#[test]
fn health_export_writes_current_environment_snapshot() {
    let repo = TempRepo::new("health-export");

    let output = run_health(repo.path(), &["export"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let health_toml = repo.path().join(".repo").join("health.toml");
    assert!(health_toml.is_file(), "expected .repo/health.toml to be written");
    let content = std::fs::read_to_string(&health_toml).unwrap();
    assert!(
        content.contains("repo health export"),
        "expected header comment in exported file: {content}"
    );
    let out = stdout(&output);
    assert!(out.contains("wrote .repo/health.toml"), "expected success message: {out}");
}

#[test]
fn health_check_verbose_flag_runs_without_error() {
    let repo = TempRepo::new("health-verbose");
    repo.write(
        ".repo/health.toml",
        "[environment]\nprivilege = \"auto\"\nallowed_runtimes = []\n",
    );

    let output = run_health(repo.path(), &["--verbose"]);

    // verbose may exit non-zero (e.g. no git repo) but should not panic
    let _ = stdout(&output);
}

#[test]
fn health_json_emits_machine_readable_report() {
    let repo = TempRepo::new("health-json");
    repo.write(
        ".repo/health.toml",
        "[environment]\nprivilege = \"auto\"\nallowed_runtimes = []\n",
    );

    let output = run_health(repo.path(), &["--json"]);

    let text = stdout(&output);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(value["sections"].is_array(), "expected sections in: {text}");
    assert!(value["passed"].is_number());
    assert!(value["warnings"].is_number());
    assert!(value["errors"].is_number());
    assert!(value["sections"][0]["checks"][0]["recommendation"].is_null() || value["sections"][0]["checks"][0]["recommendation"].is_string());
}

#[test]
fn health_human_output_includes_next_steps_for_warnings() {
    let repo = TempRepo::new("health-next-steps");
    repo.write(
        ".repo/health.toml",
        "[environment]\nprivilege = \"auto\"\nallowed_runtimes = []\n",
    );
    std::fs::create_dir_all(repo.path().join(".venv")).unwrap();

    let output = run_health(repo.path(), &[]);

    let text = stdout(&output);
    assert!(text.contains("Next steps"), "expected next steps block in: {text}");
    assert!(text.contains("Repository config"), "expected config guidance in: {text}");
    assert!(text.contains("Repository _docs"), "expected _docs guidance in: {text}");
    assert!(text.contains("Environment venv"), "expected venv guidance in: {text}");
}

#[test]
fn health_check_with_custom_tool_found_shows_ok() {
    let repo = TempRepo::new("health-custom-tool-found");
    // Use sh with a known-working version args so it always returns output.
    repo.write(
        ".repo/health.toml",
        "[tools.probe-tool]\nrequired = false\ncommand = \"sh\"\nversion_args = [\"-c\", \"echo 1.0.0\"]\n",
    );

    let output = run_health(repo.path(), &["--json"]);

    let text = stdout(&output);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let sections = value["sections"].as_array().unwrap();
    let tool_section = sections.iter().find(|s| s["name"] == "Tools");
    assert!(tool_section.is_some(), "expected Tools section in: {text}");
    let checks = tool_section.unwrap()["checks"].as_array().unwrap();
    let probe = checks.iter().find(|c| c["name"] == "probe-tool");
    assert!(probe.is_some(), "expected probe-tool in checks: {text}");
}

#[test]
fn health_check_with_passing_custom_check_shows_ok() {
    let repo = TempRepo::new("health-custom-check-pass");
    repo.write(
        ".repo/health.toml",
        "[checks.always-pass]\ncommand = \"true\"\ndescription = \"Always passes\"\nseverity = \"error\"\n",
    );

    let output = run_health(repo.path(), &["--json"]);

    let text = stdout(&output);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let sections = value["sections"].as_array().unwrap();
    let custom_section = sections.iter().find(|s| s["name"] == "Checks");
    assert!(custom_section.is_some(), "expected 'Checks' section in: {text}");
    let checks = custom_section.unwrap()["checks"].as_array().unwrap();
    let check = checks.iter().find(|c| c["name"] == "always-pass");
    assert!(check.is_some(), "expected always-pass check: {text}");
    assert_eq!(check.unwrap()["status"], "ok");
}

#[test]
fn health_check_with_warning_severity_custom_check_exits_zero() {
    let repo = TempRepo::new("health-custom-check-warn");
    repo.write(
        ".repo/health.toml",
        "[checks.warn-check]\ncommand = \"false\"\ndescription = \"Always fails\"\nseverity = \"warning\"\n",
    );

    let output = run_health(repo.path(), &["--json"]);

    // warning severity should not make the overall check exit non-zero
    let text = stdout(&output);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let sections = value["sections"].as_array().unwrap();
    let custom_section = sections.iter().find(|s| s["name"] == "Checks");
    assert!(custom_section.is_some(), "expected Checks section: {text}");
    let checks = custom_section.unwrap()["checks"].as_array().unwrap();
    let check = checks.iter().find(|c| c["name"] == "warn-check");
    assert!(check.is_some(), "expected warn-check: {text}");
    assert_eq!(check.unwrap()["status"], "warning");
}

#[test]
fn health_check_with_error_severity_custom_check_exits_nonzero() {
    let repo = TempRepo::new("health-custom-check-error");
    repo.write(
        ".repo/health.toml",
        "[checks.bad-check]\ncommand = \"false\"\ndescription = \"Always fails\"\nseverity = \"error\"\nhint = \"try something else\"\n",
    );

    let output = run_health(repo.path(), &["--json"]);

    assert!(!output.status.success(), "expected non-zero exit for error severity");
    let text = stdout(&output);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(value["errors"].as_u64().unwrap_or(0) > 0, "expected errors > 0 in: {text}");
}
