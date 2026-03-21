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
fn docs_overview_with_designs_shows_nonzero_count() {
    let repo = TempRepo::new("docs-overview-with-docs");
    repo.write(
        "_docs/designs/2026-03-21-test.md",
        &design_doc("Test Design", "accepted", "2026-03-21"),
    );

    // `repo docs` with no subcommand calls list_all()
    let output = run_docs(repo.path(), &[]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("docs overview"), "expected header in: {text}");
    assert!(text.contains("designs"), "expected kind in: {text}");
    // The count line should show "1 doc(s)"
    assert!(text.contains('1'), "expected count in: {text}");
}

#[test]
fn docs_overview_empty_shows_header_and_empty_counts() {
    let repo = TempRepo::new("docs-overview-empty");

    let output = run_docs(repo.path(), &[]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("docs overview"), "expected header in: {text}");
}

#[test]
fn status_filter_with_no_match_shows_empty_message() {
    let repo = TempRepo::new("docs-filter-empty");
    repo.write(
        "_docs/designs/2026-03-21-accepted.md",
        &design_doc("Accepted Design", "accepted", "2026-03-21"),
    );

    let output = run_docs(repo.path(), &["designs", "--status", "draft"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.contains("No matching") || text.contains("0 doc") || text.contains("No designs match"),
        "expected empty-filter message in: {text}"
    );
}

#[test]
fn complete_plan_progress_shows_all_tasks_done() {
    // Regression: plans with metadata.status=complete but valDone=0 in every
    // step were showing "0/N tasks" in the PROGRESS column.
    let repo = TempRepo::new("plans-complete-progress");
    repo.write(
        ".repo/storage/plan-done/plan-done.json",
        r#"{
          "schemaVersion": "0.3.0",
          "metadata": {
            "planId": "plan-done",
            "version": "1.0.0",
            "status": "complete",
            "createdAt": "2026-03-20T10:00:00Z",
            "updatedAt": "2026-03-20T12:00:00Z"
          },
          "problem": {
            "successOutcome": "Widget is wired up"
          },
          "steps": [
            {
              "id": "step_a",
              "title": "Do the thing",
              "size": "S",
              "validationBudget": { "valReq": 2, "valDone": 0 }
            },
            {
              "id": "step_b",
              "title": "Check the thing",
              "size": "M",
              "validationBudget": { "valReq": 1, "valDone": 0 }
            }
          ],
          "executionOrder": { "sequence": ["step_a", "step_b"] }
        }"#,
    );

    let output = run_docs(repo.path(), &["plans", "--json"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let json: Value = serde_json::from_str(&stdout(&output)).unwrap();
    let docs = json.as_array().expect("expected JSON array");
    assert_eq!(docs.len(), 1);

    let doc = docs[0].as_object().unwrap();
    assert_eq!(doc.get("status").and_then(Value::as_str), Some("complete"));

    let progress = doc.get("progress").and_then(Value::as_object).unwrap();
    let done_tasks = progress.get("done_tasks").and_then(Value::as_u64).unwrap_or(0);
    let total_tasks = progress.get("total_tasks").and_then(Value::as_u64).unwrap_or(0);
    assert!(total_tasks > 0, "expected non-zero total_tasks");
    assert_eq!(
        done_tasks, total_tasks,
        "complete plan should show all tasks done, got {done_tasks}/{total_tasks}"
    );
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
