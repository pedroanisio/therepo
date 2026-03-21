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

fn run_skills_env(repo_root: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(repo_bin());
    cmd.args(["skills"]).args(args).current_dir(repo_root);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().unwrap()
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
    assert!(help.contains("Manage required agent skills"));
    assert!(help.contains("Usage:"));
    assert!(help.contains("deploy"));
    assert!(help.contains("Examples:"));
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

#[test]
fn init_is_idempotent() {
    let repo = TempRepo::new("skills-init-idempotent");

    let out1 = run_skills(repo.path(), &["init"]);
    assert!(out1.status.success(), "first init failed: {}", stderr(&out1));

    let out2 = run_skills(repo.path(), &["init"]);
    assert!(out2.status.success(), "second init failed: {}", stderr(&out2));

    let text = stdout(&out2);
    assert!(
        text.contains("already exists"),
        "expected 'already exists' in second init output: {text}"
    );
}

#[test]
fn check_with_declared_skill_not_installed_returns_non_zero() {
    let repo = TempRepo::new("skills-check-not-installed");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(
        repo_dir.join("skills.toml"),
        "[[skills]]\nname = \"my-skill\"\nsource = \"owner/repo\"\nscope = \"project\"\n",
    )
    .unwrap();

    let output = run_skills(repo.path(), &[]);

    assert!(!output.status.success());
    let text = stdout(&output);
    assert!(text.contains("my-skill"), "expected skill name in output: {text}");
    assert!(text.contains("not installed"), "expected 'not installed' in output: {text}");
}

#[test]
fn check_with_all_declared_skills_installed_succeeds() {
    let repo = TempRepo::new("skills-check-all-ok");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(
        repo_dir.join("skills.toml"),
        "[[skills]]\nname = \"my-skill\"\nsource = \"owner/repo\"\nscope = \"project\"\n",
    )
    .unwrap();
    let skill_dir = repo.path().join(".agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---\nbody\n").unwrap();

    let output = run_skills(repo.path(), &[]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("my-skill"), "expected skill name in output: {text}");
    assert!(text.contains("0 missing"), "expected '0 missing' in output: {text}");
}

#[test]
fn check_json_emits_machine_readable_status() {
    let repo = TempRepo::new("skills-check-json");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(
        repo_dir.join("skills.toml"),
        "[[skills]]\nname = \"my-skill\"\nsource = \"owner/repo\"\nscope = \"project\"\n",
    )
    .unwrap();

    let output = run_skills(repo.path(), &["--json"]);

    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["missing"], 1);
    assert_eq!(value["items"][0]["name"], "my-skill");
}

#[test]
fn check_with_empty_manifest_reports_nothing_declared() {
    let repo = TempRepo::new("skills-check-empty-manifest");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(repo_dir.join("skills.toml"), "").unwrap();

    let output = run_skills(repo.path(), &[]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("No skills declared"), "expected 'No skills declared' in: {text}");
}

#[test]
fn export_with_installed_skills_writes_config() {
    let repo = TempRepo::new("skills-export-with-skills");
    let skill_dir = repo.path().join(".agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: my-skill\ndescription: A test skill\n---\nbody\n",
    )
    .unwrap();

    let output = run_skills(repo.path(), &["export"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(
        repo.path().join(".repo").join("skills.toml").is_file(),
        "skills.toml was not created"
    );
    let text = stdout(&output);
    assert!(text.contains("my-skill"), "expected skill name in output: {text}");
}

#[test]
fn export_json_writes_config_and_emits_report() {
    let repo = TempRepo::new("skills-export-json");
    let skill_dir = repo.path().join(".agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---\nbody\n").unwrap();

    let output = run_skills(repo.path(), &["export", "--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(repo.path().join(".repo").join("skills.toml").is_file());
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["exported"], 1);
    assert_eq!(value["skills"][0]["name"], "my-skill");
}

#[test]
fn sync_without_manifest_creates_config_from_installed() {
    let repo = TempRepo::new("skills-sync-no-manifest");
    let skill_dir = repo.path().join(".agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---\nbody\n").unwrap();

    let output = run_skills(repo.path(), &["sync"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(
        repo.path().join(".repo").join("skills.toml").is_file(),
        "skills.toml was not created"
    );
    let text = stdout(&output);
    assert!(text.contains("my-skill"), "expected skill name in output: {text}");
    assert!(text.contains("added"), "expected 'added' in sync output: {text}");
}

#[test]
fn sync_with_existing_manifest_preserves_source() {
    let repo = TempRepo::new("skills-sync-existing");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(
        repo_dir.join("skills.toml"),
        "[[skills]]\nname = \"my-skill\"\nsource = \"owner/repo\"\nscope = \"project\"\n",
    )
    .unwrap();
    let skill_dir = repo.path().join(".agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---\nbody\n").unwrap();

    let output = run_skills(repo.path(), &["sync"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let config_content = std::fs::read_to_string(repo_dir.join("skills.toml")).unwrap();
    assert!(
        config_content.contains("owner/repo"),
        "source field should be preserved after sync: {config_content}"
    );
    let text = stdout(&output);
    assert!(text.contains("kept"), "expected 'kept' in sync output: {text}");
}

#[test]
fn sync_json_writes_config_and_emits_report() {
    let repo = TempRepo::new("skills-sync-json");
    let skill_dir = repo.path().join(".agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---\nbody\n").unwrap();

    let output = run_skills(repo.path(), &["sync", "--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(repo.path().join(".repo").join("skills.toml").is_file());
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["added"], 1);
    assert_eq!(value["skills"][0]["name"], "my-skill");
}

#[test]
fn install_when_all_skills_present_reports_ok() {
    let repo = TempRepo::new("skills-install-all-present");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(
        repo_dir.join("skills.toml"),
        "[[skills]]\nname = \"my-skill\"\nsource = \"owner/repo\"\nscope = \"project\"\n",
    )
    .unwrap();
    let skill_dir = repo.path().join(".agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---\nbody\n").unwrap();

    let output = run_skills(repo.path(), &["install"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.contains("all skills are installed"),
        "expected 'all skills are installed' in: {text}"
    );
}

#[test]
fn fix_with_empty_skills_list_reports_nothing_to_fix() {
    let repo = TempRepo::new("skills-fix-empty-list");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(repo_dir.join("skills.toml"), "").unwrap();

    let output = run_skills(repo.path(), &["fix"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("nothing to fix"), "expected 'nothing to fix' in: {text}");
}

#[test]
fn unknown_subcommand_prints_error_to_stderr() {
    let repo = TempRepo::new("skills-unknown-cmd");
    let output = run_skills(repo.path(), &["bogus-subcommand"]);

    let text = stderr(&output);
    assert!(
        text.contains("Unknown skills subcommand") || text.contains("bogus-subcommand"),
        "expected unknown subcommand error in stderr: {text}"
    );
}

#[test]
fn install_json_when_all_skills_present_emits_empty_report() {
    let repo = TempRepo::new("skills-install-json-all-ok");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(
        repo_dir.join("skills.toml"),
        "[[skills]]\nname = \"my-skill\"\nsource = \"owner/repo\"\nscope = \"project\"\n",
    )
    .unwrap();
    let skill_dir = repo.path().join(".agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---\nbody\n").unwrap();

    let output = run_skills(repo.path(), &["install", "--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["items"].as_array().map(Vec::len), Some(0));
    assert_eq!(value["needs_fix"].as_array().map(Vec::len), Some(0));
}

#[test]
fn install_json_without_manifest_emits_error_json() {
    let repo = TempRepo::new("skills-install-json-missing");
    let output = run_skills(repo.path(), &["install", "--json"]);

    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(value["error"].is_string(), "expected error field in: {}", stdout(&output));
}

#[test]
fn fix_json_without_manifest_emits_error_json() {
    let repo = TempRepo::new("skills-fix-json-missing");
    let output = run_skills(repo.path(), &["fix", "--json"]);

    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(value["error"].is_string(), "expected error field in: {}", stdout(&output));
}

#[test]
fn fix_json_with_empty_manifest_emits_empty_report() {
    let repo = TempRepo::new("skills-fix-json-empty");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(repo_dir.join("skills.toml"), "").unwrap();

    let output = run_skills(repo.path(), &["fix", "--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["removed"].as_array().map(Vec::len), Some(0));
    assert_eq!(value["kept"].as_array().map(Vec::len), Some(0));
}

#[test]
fn fix_json_with_empty_source_removes_entry() {
    let repo = TempRepo::new("skills-fix-json-empty-source");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(
        repo_dir.join("skills.toml"),
        "[[skills]]\nname = \"orphan\"\nsource = \"\"\nscope = \"project\"\n",
    )
    .unwrap();

    let output = run_skills(repo.path(), &["fix", "--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let removed = value["removed"].as_array().unwrap();
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0]["name"], "orphan");
}

#[test]
fn check_json_with_all_installed_emits_zero_missing() {
    let repo = TempRepo::new("skills-check-json-ok");
    let repo_dir = repo.path().join(".repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    std::fs::write(
        repo_dir.join("skills.toml"),
        "[[skills]]\nname = \"my-skill\"\nsource = \"owner/repo\"\nscope = \"project\"\n",
    )
    .unwrap();
    let skill_dir = repo.path().join(".agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\n---\nbody\n").unwrap();

    let output = run_skills(repo.path(), &["--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["missing"], 0);
    assert_eq!(value["installed"], 1);
}

#[test]
fn deploy_with_force_overwrites_existing_skills() {
    let home = TempRepo::new("skills-deploy-force");
    // First deploy
    let out1 = run_skills_env(
        home.path(),
        &["deploy"],
        &[("HOME", home.path().to_str().unwrap())],
    );
    assert!(out1.status.success(), "first deploy failed: {}", stderr(&out1));

    // Second deploy with --force should succeed and overwrite
    let out2 = run_skills_env(
        home.path(),
        &["deploy", "--force"],
        &[("HOME", home.path().to_str().unwrap())],
    );
    assert!(out2.status.success(), "force deploy failed: {}", stderr(&out2));
    let text = stdout(&out2);
    assert!(
        !text.contains("already installed"),
        "with --force, nothing should be skipped: {text}"
    );
}

#[test]
fn deploy_installs_builtin_skills_to_custom_home() {
    let home = TempRepo::new("skills-deploy-home");
    let output = run_skills_env(
        home.path(),
        &["deploy"],
        &[("HOME", home.path().to_str().unwrap())],
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.contains("Deploying built-in skills"),
        "expected deploy banner in: {text}"
    );
    assert!(
        home.path().join(".agents").join("skills").is_dir(),
        "expected ~/.agents/skills/ to be created"
    );
}
