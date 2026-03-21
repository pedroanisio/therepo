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

fn cleanup(repo_root: PathBuf) {
    std::fs::remove_dir_all(repo_root).ok();
}

fn ulid_lines(output: &std::process::Output) -> Vec<String> {
    stdout(output)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn prompt_help_prints_usage() {
    let repo_root = temp_repo("prompt-help");
    let output = run_repo(&repo_root, &["prompt", "--help"]);

    assert!(output.status.success());
    let help = stdout(&output);
    assert!(help.contains("Reusable prompt snippets"));
    assert!(help.contains("Usage:"));
    assert!(help.contains("init"));
    assert!(help.contains("Examples:"));

    cleanup(repo_root);
}

#[test]
fn prompt_init_writes_defaults_and_list_reports_builtins() {
    let repo_root = temp_repo("prompt-init");
    let init = run_repo(&repo_root, &["prompt", "init"]);

    assert!(init.status.success(), "stderr: {}", stderr(&init));

    let prompts_dir = repo_root.join(".repo").join("prompts");
    assert!(prompts_dir.join("assess-corpus.md").is_file());
    assert!(prompts_dir.join("validate-plan.md").is_file());

    let list = run_repo(&repo_root, &["prompt", "list"]);

    assert!(list.status.success(), "stderr: {}", stderr(&list));
    let text = stdout(&list);
    assert!(text.contains("assess-corpus"));
    assert!(text.contains("validate-plan"));
    assert!(text.contains("6 custom"));

    cleanup(repo_root);
}

#[test]
fn prompt_custom_prompt_overrides_builtin_body() {
    let repo_root = temp_repo("prompt-override");
    let prompts_dir = repo_root.join(".repo").join("prompts");
    std::fs::create_dir_all(&prompts_dir).unwrap();
    std::fs::write(
        prompts_dir.join("format-plan.md"),
        "---\nname: format-plan\ndescription: Custom formatter\ntags: [format, review]\n---\ncustom prompt body\n",
    )
    .unwrap();

    let output = run_repo(&repo_root, &["prompt", "format-plan"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output).trim(), "custom prompt body");

    cleanup(repo_root);
}

#[test]
fn prompt_tag_filter_returns_matching_prompt_only() {
    let repo_root = temp_repo("prompt-tag");
    let prompts_dir = repo_root.join(".repo").join("prompts");
    std::fs::create_dir_all(&prompts_dir).unwrap();
    std::fs::write(
        prompts_dir.join("team-review.md"),
        "---\nname: team-review\ndescription: Team review prompt\ntags: [alpha-test]\n---\nreview body\n",
    )
    .unwrap();

    let output = run_repo(&repo_root, &["prompt", "list", "--tag", "alpha-test"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("team-review"));
    assert!(!text.contains("validate-plan"));

    cleanup(repo_root);
}

#[test]
fn prompt_unknown_name_returns_non_zero_with_guidance() {
    let repo_root = temp_repo("prompt-unknown");
    let output = run_repo(&repo_root, &["prompt", "missing-prompt"]);

    assert!(!output.status.success());
    let text = stderr(&output);
    assert!(text.contains("Unknown prompt: missing-prompt"));
    assert!(text.contains("repo prompt list"));

    cleanup(repo_root);
}

#[test]
fn ulid_help_prints_usage() {
    let repo_root = temp_repo("ulid-help");
    let output = run_repo(&repo_root, &["ulid", "--help"]);

    assert!(output.status.success());
    let help = stdout(&output);
    assert!(help.contains("repo ulid"));
    assert!(help.contains("Usage:"));
    assert!(help.contains("Examples:"));

    cleanup(repo_root);
}

#[test]
fn ulid_default_generates_one_ulid() {
    let repo_root = temp_repo("ulid-default");
    let output = run_repo(&repo_root, &["ulid"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let lines = ulid_lines(&output);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].len(), 26);

    cleanup(repo_root);
}

#[test]
fn ulid_count_generates_requested_number_of_ulids() {
    let repo_root = temp_repo("ulid-count");
    let output = run_repo(&repo_root, &["ulid", "-n", "3"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let lines = ulid_lines(&output);
    assert_eq!(lines.len(), 3);
    assert!(lines.iter().all(|line| line.len() == 26));

    cleanup(repo_root);
}

#[test]
fn ulid_invalid_count_returns_non_zero_with_error() {
    let repo_root = temp_repo("ulid-invalid");
    let output = run_repo(&repo_root, &["ulid", "-n", "0"]);

    assert!(!output.status.success());
    let text = stderr(&output);
    assert!(text.contains("count must be greater than zero"));

    cleanup(repo_root);
}
