use serde_json::Value;
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

fn run_docs(repo_root: &Path, args: &[&str]) -> Output {
    Command::new(repo_bin())
        .args(["docs"])
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

fn design_doc(title: &str, status: &str, date: &str) -> String {
    format!(
        concat!(
            "---\n",
            "title: \"{title}\"\n",
            "version: \"1.0.0\"\n",
            "status: {status}\n",
            "date: {date}\n",
            "---\n\n",
            "# {title}\n"
        ),
        title = title,
        status = status,
        date = date,
    )
}

#[test]
fn help_prints_docs_usage() {
    let repo = TempRepo::new("docs-help");
    let output = run_docs(repo.path(), &["--help"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let help = stdout(&output);
    assert!(help.contains("Browse plans, ADRs, and references"));
    assert!(help.contains("Usage:"));
    assert!(help.contains("references"));
    assert!(help.contains("Examples:"));
}

#[test]
fn empty_designs_reports_no_docs_found() {
    let repo = TempRepo::new("docs-empty");
    let output = run_docs(repo.path(), &["designs"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output).trim(), "No designs found in _docs/designs/");
}

#[test]
fn status_filter_returns_only_matching_docs() {
    let repo = TempRepo::new("docs-filter");
    repo.write(
        "_docs/designs/2026-03-21-accepted.md",
        &design_doc("Accepted design", "accepted", "2026-03-21"),
    );
    repo.write(
        "_docs/designs/2026-03-20-draft.md",
        &design_doc("Draft design", "draft", "2026-03-20"),
    );

    let output = run_docs(repo.path(), &["designs", "--status", "accepted"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Accepted design"));
    assert!(!text.contains("Draft design"));
}

#[test]
fn json_output_emits_machine_readable_docs() {
    let repo = TempRepo::new("docs-json");
    repo.write(
        "_docs/designs/2026-03-21-accepted.md",
        &design_doc("Accepted design", "accepted", "2026-03-21"),
    );

    let output = run_docs(repo.path(), &["designs", "--json"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let json: Value = serde_json::from_str(&stdout(&output)).unwrap();
    let docs = json.as_array().expect("expected a JSON array");
    assert_eq!(docs.len(), 1);

    let doc = docs[0].as_object().expect("expected an object");
    assert_eq!(doc.get("file").and_then(Value::as_str), Some("2026-03-21-accepted.md"));
    assert_eq!(doc.get("title").and_then(Value::as_str), Some("Accepted design"));
    assert_eq!(doc.get("status").and_then(Value::as_str), Some("accepted"));
    assert_eq!(doc.get("date").and_then(Value::as_str), Some("2026-03-21"));
}
