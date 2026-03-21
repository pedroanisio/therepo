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

}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn run_skills(repo_root: &Path, args: &[&str]) -> Output {
    Command::new(repo_bin())
        .args(["skills"])
        .args(args)
        .current_dir(repo_root)
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
fn help_prints_skills_usage() {
    let repo = TempRepo::new("skills-help");
    let output = run_skills(repo.path(), &["--help"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let help = stdout(&output);
    assert!(help.contains("repo skills [COMMAND]"));
    assert!(help.contains("deploy      Install all built-in skills"));
}

#[test]
fn init_writes_skills_manifest_and_builtin_assets() {
    let repo = TempRepo::new("skills-init");
    let output = run_skills(repo.path(), &["init"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(repo.path().join(".repo").join("skills.toml").is_file());
    assert!(
        repo.path()
            .join(".repo")
            .join("skills")
            .join("01KM1A156P4VEY0KT304QXA466-testing-standards.md")
            .is_file()
    );
    assert!(
        repo.path()
            .join(".repo")
            .join("references")
            .join("01KM23VWVQWH62NBFF0TTFWVXR-sync-checks.md")
            .is_file()
    );
    assert!(
        repo.path()
            .join(".repo")
            .join("schemas")
            .join("01KM18ZD23GC3TDVN7W0GX2000-plan-schema.ts")
            .is_file()
    );
}

#[test]
fn check_without_manifest_reports_missing_config() {
    let repo = TempRepo::new("skills-check-missing");
    let output = run_skills(repo.path(), &[]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("no .repo/skills.toml found"));
    assert!(text.contains("repo skills init"));
}

#[test]
fn export_without_installed_skills_reports_empty_state() {
    let repo = TempRepo::new("skills-export-empty");
    let output = run_skills(repo.path(), &["export"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output).trim(), "No skills found in .agents/skills/");
}

#[test]
fn install_without_manifest_returns_non_zero() {
    let repo = TempRepo::new("skills-install-missing");
    let output = run_skills(repo.path(), &["install"]);

    assert!(!output.status.success());
    let text = stderr(&output);
    assert!(text.contains("No .repo/skills.toml found"));
    assert!(text.contains("repo skills init"));
}

#[test]
fn fix_without_manifest_returns_non_zero() {
    let repo = TempRepo::new("skills-fix-missing");
    let output = run_skills(repo.path(), &["fix"]);

    assert!(!output.status.success());
    let text = stderr(&output);
    assert!(text.contains("No .repo/skills.toml found"));
    assert!(text.contains("repo skills init"));
}
