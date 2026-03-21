use crate::output::{bold, cyan, dim, green, status_color, yellow};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ── Document model ──────────────────────────────────────────────────

pub struct Doc {
    pub file: String,
    pub title: String,
    pub version: String,
    pub status: String,
    pub date: String,
    pub phases: Vec<PlanPhase>,
}

pub struct PlanPhase {
    pub name: String,
    pub done: usize,
    pub total: usize,
}

#[derive(Serialize)]
struct JsonDoc<'a> {
    file: &'a str,
    title: &'a str,
    version: &'a str,
    status: &'a str,
    date: &'a str,
    progress: JsonProgress,
    phases: Vec<JsonPlanPhase<'a>>,
}

#[derive(Serialize)]
struct JsonProgress {
    complete_phases: usize,
    total_phases: usize,
    done_tasks: usize,
    total_tasks: usize,
}

#[derive(Serialize)]
struct JsonPlanPhase<'a> {
    name: &'a str,
    done: usize,
    total: usize,
    status: &'static str,
}

#[derive(Clone, Copy)]
pub enum DocKind {
    Plans,
    Designs,
    Adrs,
    References,
}

impl DocKind {
    pub fn subdir(self) -> &'static str {
        match self {
            Self::Plans => "plans",
            Self::Designs => "designs",
            Self::Adrs => "adrs",
            Self::References => "references",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Plans => "plan",
            Self::Designs => "design",
            Self::Adrs => "ADR",
            Self::References => "reference",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "plans" => Some(Self::Plans),
            "designs" => Some(Self::Designs),
            "adrs" => Some(Self::Adrs),
            "references" | "refs" => Some(Self::References),
            _ => None,
        }
    }
}

pub const ALL_KINDS: [DocKind; 4] = [
    DocKind::Plans,
    DocKind::Designs,
    DocKind::Adrs,
    DocKind::References,
];

// ── Commands ────────────────────────────────────────────────────────

pub fn run(repo_root: &Path, args: &[&str]) {
    let subcommand = args.first().copied();

    if args.iter().any(|a| *a == "--help" || *a == "-h") {
        if let Some(kind) = subcommand.and_then(DocKind::from_str) {
            print_subcommand_help(kind);
        } else {
            print_help();
        }
        return;
    }

    match subcommand {
        Some(sub) => {
            if let Some(kind) = DocKind::from_str(sub) {
                let remaining: Vec<&str> = args[1..].to_vec();
                list_kind(repo_root, kind, &remaining);
            } else {
                eprintln!("Unknown docs subcommand: {sub}");
                eprintln!("Run `repo docs --help` for usage.");
                std::process::exit(1);
            }
        }
        None => list_all(repo_root),
    }
}

fn print_help() {
    println!(
        "\
repo docs — Browse plans, ADRs, and references

USAGE:
    repo docs [COMMAND] [OPTIONS]

COMMANDS:
    plans       List plans in .repo/storage/
    designs     List documents in _docs/designs/
    adrs        List documents in _docs/adrs/
    references  List documents in _docs/references/  (alias: refs)

OPTIONS:
    -h, --help  Print this help message

When no command is given, a summary of all document kinds is shown."
    );
}

fn print_subcommand_help(kind: DocKind) {
    let location = match kind {
        DocKind::Plans => ".repo/storage/**/*.{md,json}".to_string(),
        _ => format!("_docs/{}/*.md", kind.subdir()),
    };
    println!(
        "\
repo docs {sub} — List {label}s

USAGE:
    repo docs {sub} [OPTIONS]

OPTIONS:
    --status <STATUS>  Filter by status (e.g. proposal, draft, active, accepted)
    --json             Emit machine-readable JSON instead of a table
    -h, --help         Print this help message

Scans {location} for plan documents.",
        sub = kind.subdir(),
        label = kind.label(),
        location = location,
    );
}

pub fn list_all(repo_root: &Path) {
    println!("{}", bold("docs overview"));
    println!();

    for kind in ALL_KINDS {
        let docs = resolve_docs(repo_root, kind).unwrap_or_default();
        let count = docs.len();

        let status_summary = if docs.is_empty() {
            dim("(empty)")
        } else {
            let mut counts: HashMap<String, usize> = HashMap::new();
            for d in &docs {
                *counts.entry(d.status.clone()).or_default() += 1;
            }
            let mut parts: Vec<String> = counts
                .iter()
                .map(|(status, n)| format!("{n} {status}"))
                .collect();
            parts.sort();
            dim(&parts.join(", "))
        };

        let location = match kind {
            DocKind::Plans => ".repo/storage/",
            _ => "_docs/",
        };

        println!(
            "  {:<12} {:>3} doc(s)  {}  {}",
            bold(kind.subdir()),
            count,
            status_summary,
            dim(location),
        );
    }

    println!();
    println!(
        "  Run {} for details.",
        dim("repo docs <plans|designs|adrs|refs>")
    );
}

fn list_kind(repo_root: &Path, kind: DocKind, args: &[&str]) {
    let docs = match resolve_docs(repo_root, kind) {
        Ok(docs) => docs,
        Err(e) => {
            eprintln!("Error scanning {}: {e}", kind.subdir());
            std::process::exit(1);
        }
    };

    if docs.is_empty() {
        let location = match kind {
            DocKind::Plans => ".repo/storage/".to_string(),
            _ => format!("_docs/{}/", kind.subdir()),
        };
        println!("No {}s found in {}", kind.label(), location);
        return;
    }

    // Validate --status flag.
    let has_status_flag = args.iter().any(|a| *a == "--status");
    let status_filter = args
        .windows(2)
        .find(|w| w[0] == "--status")
        .map(|w| w[1].to_lowercase());

    if has_status_flag && status_filter.is_none() {
        eprintln!("Missing value for --status. Usage: --status <STATUS>");
        std::process::exit(1);
    }

    let json_output = args.iter().any(|a| *a == "--json");

    let filtered: Vec<&Doc> = docs
        .iter()
        .filter(|d| {
            if let Some(ref filter) = status_filter {
                d.status.to_lowercase() == *filter
            } else {
                true
            }
        })
        .collect();

    if filtered.is_empty() {
        if json_output {
            println!("[]");
            return;
        }
        println!("No {}s match the given filter.", kind.label());
        return;
    }

    if json_output {
        print_json(&filtered);
        return;
    }

    print_table(kind, &filtered);
}

// ── Resolve docs per kind ──────────────────────────────────────────

fn resolve_docs(repo_root: &Path, kind: DocKind) -> Result<Vec<Doc>, String> {
    match kind {
        DocKind::Plans => {
            let storage = repo_root.join(".repo").join("storage");
            if !storage.is_dir() {
                return Ok(Vec::new());
            }
            scan_storage_plans(&storage)
        }
        _ => {
            let dir = repo_root.join("_docs").join(kind.subdir());
            if !dir.is_dir() {
                return Ok(Vec::new());
            }
            scan_docs(&dir)
        }
    }
}

// ── Scanning _docs/ (designs, adrs, references) ────────────────────

pub fn scan_docs(dir: &Path) -> Result<Vec<Doc>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;

    let mut docs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;

        let phases = parse_plan_phases(&content);

        if let Some(mut doc) = parse_frontmatter(&content, &path) {
            doc.phases = phases;
            docs.push(doc);
        } else {
            docs.push(Doc {
                file: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                title: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .replace('-', " "),
                version: String::new(),
                status: "\u{2014}".into(),
                date: "\u{2014}".into(),
                phases,
            });
        }
    }

    docs.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));
    Ok(docs)
}

// ── Scanning .repo/storage/ (plans) ────────────────────────────────

fn scan_storage_plans(storage_dir: &Path) -> Result<Vec<Doc>, String> {
    let mut docs = Vec::new();
    walk_storage_dir(storage_dir, &mut docs);
    docs.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));
    Ok(docs)
}

fn walk_storage_dir(dir: &Path, docs: &mut Vec<Doc>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            walk_storage_dir(&path, docs);
        } else if let Some(doc) = try_parse_plan_file(&path) {
            docs.push(doc);
        }
    }
}

fn try_parse_plan_file(path: &Path) -> Option<Doc> {
    let ext = path.extension().and_then(|e| e.to_str())?;

    match ext {
        "md" => {
            let content = fs::read_to_string(path).ok()?;
            let phases = parse_plan_phases(&content);

            // Only include markdown files that look like plans:
            // they must have frontmatter with a title, or contain ## Phase headings.
            let has_phases = !phases.is_empty();
            let doc = parse_frontmatter(&content, path);

            if let Some(mut doc) = doc {
                doc.phases = phases;
                Some(doc)
            } else if has_phases {
                Some(Doc {
                    file: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    title: path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .replace('-', " "),
                    version: String::new(),
                    status: "\u{2014}".into(),
                    date: "\u{2014}".into(),
                    phases,
                })
            } else {
                None
            }
        }
        "json" => {
            let content = fs::read_to_string(path).ok()?;
            parse_plan_json(&content, path)
        }
        _ => None,
    }
}

// ── PlanSchema JSON parsing ────────────────────────────────────────

fn parse_plan_json(content: &str, path: &Path) -> Option<Doc> {
    let val: serde_json::Value = serde_json::from_str(content).ok()?;
    let obj = val.as_object()?;

    // Must have schemaVersion to be a PlanSchema file.
    obj.get("schemaVersion")?.as_str()?;

    let metadata = obj.get("metadata")?.as_object()?;
    let problem = obj.get("problem").and_then(|v| v.as_object());

    let plan_id = metadata
        .get("planId")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let version = metadata
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Title: prefer successOutcome (concise), then planId, then problemStatement.
    let title = problem
        .and_then(|p| p.get("successOutcome"))
        .and_then(|v| v.as_str())
        .map(|s| truncate_title(s, 72))
        .or_else(|| {
            if !plan_id.is_empty() {
                Some(plan_id.replace('-', " "))
            } else {
                None
            }
        })
        .or_else(|| {
            problem
                .and_then(|p| p.get("problemStatement"))
                .and_then(|v| v.as_str())
                .map(|s| truncate_title(s, 72))
        })
        .unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .replace('-', " ")
        });

    // Date: prefer updatedAt, then createdAt.
    let date = metadata
        .get("updatedAt")
        .or_else(|| metadata.get("createdAt"))
        .and_then(|v| v.as_str())
        .map(|d| {
            // Extract just the date part from ISO 8601.
            if d.len() >= 10 {
                d[..10].to_string()
            } else {
                d.to_string()
            }
        })
        .unwrap_or_else(|| "\u{2014}".into());

    // Extract steps as pseudo-phases grouped by execution order.
    let phases = extract_json_phases(obj);
    let status = derive_plan_status(obj, &phases);

    Some(Doc {
        file: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        title,
        version,
        status,
        date,
        phases,
    })
}

fn derive_plan_status(
    obj: &serde_json::Map<String, serde_json::Value>,
    phases: &[PlanPhase],
) -> String {
    if !phases.is_empty() {
        let all_complete = phases.iter().all(|p| p.total > 0 && p.done == p.total);
        if all_complete {
            return "complete".into();
        }

        let any_progress = phases.iter().any(|p| p.done > 0);
        if any_progress {
            return "active".into();
        }
    }

    // Plans with versionHistory entries suggest they've been through review.
    let has_history = obj
        .get("metadata")
        .and_then(|m| m.get("versionHistory"))
        .and_then(|v| v.as_array())
        .is_some_and(|a| !a.is_empty());

    if has_history {
        "active".into()
    } else {
        "proposal".into()
    }
}

fn extract_json_phases(obj: &serde_json::Map<String, serde_json::Value>) -> Vec<PlanPhase> {
    let steps = match obj.get("steps").and_then(|v| v.as_array()) {
        Some(s) => s,
        None => return Vec::new(),
    };

    if steps.is_empty() {
        return Vec::new();
    }

    // Build a step lookup.
    let step_map: HashMap<String, &serde_json::Value> = steps
        .iter()
        .filter_map(|s| {
            let id = s.get("id")?.as_str()?;
            Some((id.to_string(), s))
        })
        .collect();

    // Build parallel group lookup: step_id -> group index.
    let parallel_of: HashMap<String, usize> = obj
        .get("executionOrder")
        .and_then(|eo| eo.get("parallelizableGroups"))
        .and_then(|v| v.as_array())
        .map(|groups| {
            groups
                .iter()
                .enumerate()
                .flat_map(|(gi, g)| {
                    g.as_array().into_iter().flat_map(move |a| {
                        a.iter()
                            .filter_map(move |v| v.as_str().map(|s| (s.to_string(), gi)))
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Follow executionOrder.sequence — one phase per step.
    let sequence = obj
        .get("executionOrder")
        .and_then(|eo| eo.get("sequence"))
        .and_then(|v| v.as_array());

    let ordered_ids: Vec<&str> = match sequence {
        Some(seq) => seq.iter().filter_map(|v| v.as_str()).collect(),
        None => steps
            .iter()
            .filter_map(|s| s.get("id").and_then(|v| v.as_str()))
            .collect(),
    };

    let mut phases = Vec::new();

    for (i, id) in ordered_ids.iter().enumerate() {
        let step = step_map.get(*id);
        let title = step
            .and_then(|s| s.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or(id);
        let size = step
            .and_then(|s| s.get("size"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");

        // Annotate parallel steps.
        let parallel_tag = parallel_of.get(*id).map(|gi| {
            // Check if the previous step is in the same group — if so, use "├" else "┌".
            let prev_same = i > 0
                && parallel_of
                    .get(ordered_ids[i - 1])
                    .is_some_and(|pg| pg == gi);
            let next_same = i + 1 < ordered_ids.len()
                && parallel_of
                    .get(ordered_ids[i + 1])
                    .is_some_and(|ng| ng == gi);
            match (prev_same, next_same) {
                (false, true) => "\u{250c} ",  // ┌ first in group
                (true, true) => "\u{251c} ",   // ├ middle
                (true, false) => "\u{2514} ",  // └ last in group
                (false, false) => "\u{2500} ", // ─ solo (shouldn't happen)
            }
        });

        let name = match parallel_tag {
            Some(tag) => format!("{tag}[{size}] {title}"),
            None => format!("[{size}] {title}"),
        };

        let (done, total) = step.copied().map(step_progress).unwrap_or((0, 1));

        phases.push(PlanPhase { name, done, total });
    }

    phases
}

fn print_json(docs: &[&Doc]) {
    let payload: Vec<JsonDoc<'_>> = docs.iter().map(|doc| to_json_doc(doc)).collect();
    match serde_json::to_string_pretty(&payload) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            eprintln!("Failed to serialize docs as JSON: {err}");
            std::process::exit(1);
        }
    }
}

fn to_json_doc(doc: &Doc) -> JsonDoc<'_> {
    JsonDoc {
        file: &doc.file,
        title: &doc.title,
        version: &doc.version,
        status: &doc.status,
        date: &doc.date,
        progress: plan_progress(&doc.phases),
        phases: doc
            .phases
            .iter()
            .map(|phase| JsonPlanPhase {
                name: &phase.name,
                done: phase.done,
                total: phase.total,
                status: phase_status(phase),
            })
            .collect(),
    }
}

fn plan_progress(phases: &[PlanPhase]) -> JsonProgress {
    JsonProgress {
        complete_phases: phases
            .iter()
            .filter(|p| p.total > 0 && p.done == p.total)
            .count(),
        total_phases: phases.len(),
        done_tasks: phases.iter().map(|p| p.done).sum(),
        total_tasks: phases.iter().map(|p| p.total).sum(),
    }
}

fn phase_status(phase: &PlanPhase) -> &'static str {
    if phase.total == 0 {
        "unknown"
    } else if phase.done == phase.total {
        "done"
    } else if phase.done > 0 {
        "partial"
    } else {
        "pending"
    }
}

fn step_progress(step: &serde_json::Value) -> (usize, usize) {
    let Some(budget) = step.get("validationBudget").and_then(|v| v.as_object()) else {
        return (0, 1);
    };

    if let (Some(total), Some(done)) = (
        budget.get("valReq").and_then(|v| v.as_u64()),
        budget.get("valDone").and_then(|v| v.as_u64()),
    ) {
        return (done as usize, total as usize);
    }

    if let (Some(total), Some(done)) = (
        budget.get("required").and_then(|v| v.as_u64()),
        budget.get("performed").and_then(|v| v.as_u64()),
    ) {
        return (done as usize, total as usize);
    }

    (0, 1)
}

fn truncate_title(s: &str, max: usize) -> String {
    // If there's a sentence boundary (.: ;) early enough, cut there.
    for (i, c) in s.char_indices() {
        if i > 0 && i <= max && matches!(c, ':' | ';') {
            let candidate = s[..i].trim();
            if candidate.len() >= max / 3 {
                return candidate.to_string();
            }
        }
    }

    if s.len() <= max {
        s.to_string()
    } else {
        // Break at a word boundary.
        let truncated = &s[..max - 3];
        match truncated.rfind(' ') {
            Some(pos) if pos > max / 2 => format!("{}...", &s[..pos]),
            _ => format!("{truncated}..."),
        }
    }
}

// ── Markdown frontmatter parsing ───────────────────────────────────

fn parse_frontmatter(content: &str, path: &Path) -> Option<Doc> {
    let content = content.trim_start();

    if !content.starts_with("---") {
        return None;
    }

    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];
    let fields = parse_yaml_fields(frontmatter);

    let title = fields.get("title")?.clone();

    Some(Doc {
        file: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        title,
        version: fields.get("version").cloned().unwrap_or_default(),
        status: fields
            .get("status")
            .cloned()
            .unwrap_or_else(|| "\u{2014}".into()),
        date: fields
            .get("date")
            .cloned()
            .unwrap_or_else(|| "\u{2014}".into()),
        phases: Vec::new(),
    })
}

fn parse_yaml_fields(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in text.lines() {
        if line.starts_with(' ') || line.starts_with('\t') || line.trim().is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim();

            if value == ">" || value == "|" || value.is_empty() {
                continue;
            }

            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);

            map.insert(key.to_string(), value.to_string());
        }
    }

    map
}

// ── Plan-phase extraction (markdown) ───────────────────────────────

fn parse_plan_phases(content: &str) -> Vec<PlanPhase> {
    let mut phases: Vec<PlanPhase> = Vec::new();
    let mut current_name: Option<String> = None;
    let mut done: usize = 0;
    let mut total: usize = 0;

    for line in content.lines() {
        // Detect "## Phase N — Title" or "## Phase N - Title" headings
        if line.starts_with("## Phase ") || line.starts_with("## phase ") {
            // Flush previous phase
            if let Some(name) = current_name.take() {
                phases.push(PlanPhase { name, done, total });
            }
            // Extract phase name: strip "## " prefix
            let heading = line.trim_start_matches('#').trim();
            current_name = Some(heading.to_string());
            done = 0;
            total = 0;
        } else if current_name.is_some() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
                total += 1;
                done += 1;
            } else if trimmed.starts_with("- [ ]") {
                total += 1;
            }
        }
    }

    // Flush last phase
    if let Some(name) = current_name.take() {
        phases.push(PlanPhase { name, done, total });
    }

    phases
}

// ── Table rendering ─────────────────────────────────────────────────

fn print_table(kind: DocKind, docs: &[&Doc]) {
    let has_phases = matches!(kind, DocKind::Plans) && docs.iter().any(|d| !d.phases.is_empty());

    let hdr = ("FILE", "TITLE", "VERSION", "STATUS", "DATE");

    let w_file = docs
        .iter()
        .map(|d| d.file.len())
        .max()
        .unwrap_or(0)
        .max(hdr.0.len());
    let w_title = docs
        .iter()
        .map(|d| d.title.len())
        .max()
        .unwrap_or(0)
        .max(hdr.1.len());
    let w_ver = docs
        .iter()
        .map(|d| d.version.len())
        .max()
        .unwrap_or(0)
        .max(hdr.2.len());
    let w_status = docs
        .iter()
        .map(|d| d.status.len())
        .max()
        .unwrap_or(0)
        .max(hdr.3.len());
    let w_date = docs
        .iter()
        .map(|d| d.date.len())
        .max()
        .unwrap_or(0)
        .max(hdr.4.len());

    // If we have phases, add a PROGRESS column
    let progress_hdr = "PROGRESS";
    let w_progress = if has_phases {
        docs.iter()
            .map(|d| format_progress_summary(&d.phases).len())
            .max()
            .unwrap_or(0)
            .max(progress_hdr.len())
    } else {
        0
    };

    // Header
    if has_phases {
        println!(
            "  {:<w_file$}  {:<w_title$}  {:<w_ver$}  {:<w_status$}  {:<w_date$}  {:<w_progress$}",
            bold(hdr.0),
            bold(hdr.1),
            bold(hdr.2),
            bold(hdr.3),
            bold(hdr.4),
            bold(progress_hdr),
        );
        println!(
            "  {}  {}  {}  {}  {}  {}",
            dim(&"\u{2500}".repeat(w_file)),
            dim(&"\u{2500}".repeat(w_title)),
            dim(&"\u{2500}".repeat(w_ver)),
            dim(&"\u{2500}".repeat(w_status)),
            dim(&"\u{2500}".repeat(w_date)),
            dim(&"\u{2500}".repeat(w_progress)),
        );
    } else {
        println!(
            "  {:<w_file$}  {:<w_title$}  {:<w_ver$}  {:<w_status$}  {:<w_date$}",
            bold(hdr.0),
            bold(hdr.1),
            bold(hdr.2),
            bold(hdr.3),
            bold(hdr.4),
        );
        println!(
            "  {}  {}  {}  {}  {}",
            dim(&"\u{2500}".repeat(w_file)),
            dim(&"\u{2500}".repeat(w_title)),
            dim(&"\u{2500}".repeat(w_ver)),
            dim(&"\u{2500}".repeat(w_status)),
            dim(&"\u{2500}".repeat(w_date)),
        );
    }

    // Rows
    for doc in docs {
        let status_display = status_color(&doc.status);
        let status_padding = w_status.saturating_sub(doc.status.len());

        if has_phases {
            let progress = format_progress_summary(&doc.phases);
            let progress_colored = color_progress_summary(&doc.phases);
            let progress_padding = w_progress.saturating_sub(progress.len());
            println!(
                "  {:<w_file$}  {:<w_title$}  {:<w_ver$}  {}{:>spad$}  {:<w_date$}  {}{:>ppad$}",
                doc.file,
                doc.title,
                doc.version,
                status_display,
                "",
                doc.date,
                progress_colored,
                "",
                spad = status_padding,
                ppad = progress_padding,
            );
        } else {
            println!(
                "  {:<w_file$}  {:<w_title$}  {:<w_ver$}  {}{:>pad$}  {:<w_date$}",
                doc.file,
                doc.title,
                doc.version,
                status_display,
                "",
                doc.date,
                pad = status_padding,
            );
        }
    }

    println!();

    // Phase details for plans
    if has_phases {
        for doc in docs {
            if doc.phases.is_empty() {
                continue;
            }
            println!("  {} {}", bold(&doc.title), dim("phases:"));
            for phase in &doc.phases {
                let bar = progress_bar(phase.done, phase.total, 16);
                let status_tag = if phase.total == 0 {
                    dim("--")
                } else if phase.done == phase.total {
                    green("done")
                } else if phase.done > 0 {
                    yellow("partial")
                } else {
                    dim("pending")
                };
                println!(
                    "    {bar}  {done}/{total}  {status}  {name}",
                    done = phase.done,
                    total = phase.total,
                    status = status_tag,
                    name = phase.name,
                );
            }
            println!();
        }
    }

    println!("  {} {}(s) found", docs.len(), kind.label());
}

fn format_progress_summary(phases: &[PlanPhase]) -> String {
    if phases.is_empty() {
        return "\u{2014}".to_string();
    }
    let total_done: usize = phases.iter().map(|p| p.done).sum();
    let total_all: usize = phases.iter().map(|p| p.total).sum();
    let n_phases = phases.len();
    let n_complete = phases
        .iter()
        .filter(|p| p.total > 0 && p.done == p.total)
        .count();
    format!("{n_complete}/{n_phases} phases  {total_done}/{total_all} tasks")
}

fn color_progress_summary(phases: &[PlanPhase]) -> String {
    if phases.is_empty() {
        return dim("\u{2014}");
    }
    let total_done: usize = phases.iter().map(|p| p.done).sum();
    let total_all: usize = phases.iter().map(|p| p.total).sum();
    let n_phases = phases.len();
    let n_complete = phases
        .iter()
        .filter(|p| p.total > 0 && p.done == p.total)
        .count();

    let phase_part = if n_complete == n_phases {
        green(&format!("{n_complete}/{n_phases} phases"))
    } else if n_complete > 0 {
        yellow(&format!("{n_complete}/{n_phases} phases"))
    } else {
        dim(&format!("{n_complete}/{n_phases} phases"))
    };

    let task_part = if total_done == total_all && total_all > 0 {
        green(&format!("{total_done}/{total_all} tasks"))
    } else if total_done > 0 {
        yellow(&format!("{total_done}/{total_all} tasks"))
    } else {
        dim(&format!("{total_done}/{total_all} tasks"))
    };

    format!("{phase_part}  {task_part}")
}

fn progress_bar(done: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return dim(&format!("[{}]", " ".repeat(width)));
    }
    let filled = (done * width) / total;
    let empty = width - filled;
    let bar_str = format!(
        "[{}{}]",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty)
    );
    if done == total {
        green(&bar_str)
    } else if done > 0 {
        yellow(&bar_str)
    } else {
        cyan(&bar_str)
    }
}
