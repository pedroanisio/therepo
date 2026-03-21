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
