use crate::output::{bold, cyan, dim, green, status_color, yellow};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, IsTerminal, Write as _};
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SortMode {
    Date,
    Status,
    Title,
    Progress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DetailsMode {
    None,
    Incomplete,
    All,
}

struct ListOptions {
    query: Option<String>,
    status_filter: Option<String>,
    json_output: bool,
    sort: SortMode,
    limit: Option<usize>,
    details: DetailsMode,
    interactive: bool,
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
    #[must_use]
    pub fn subdir(self) -> &'static str {
        match self {
            Self::Plans => "plans",
            Self::Designs => "designs",
            Self::Adrs => "adrs",
            Self::References => "references",
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Plans => "plan",
            Self::Designs => "design",
            Self::Adrs => "ADR",
            Self::References => "reference",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
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
    let subcommand = args.iter().copied().find(|arg| !arg.starts_with('-'));
    let json_output = args.contains(&"--json");

    if args.iter().any(|a| *a == "--help" || *a == "-h") {
        if let Some(kind) = subcommand.and_then(DocKind::parse) {
            print_subcommand_help(kind);
        } else {
            print_help();
        }
        return;
    }

    match subcommand {
        Some(sub) => {
            if let Some(kind) = DocKind::parse(sub) {
                let remaining: Vec<&str> = args
                    .iter()
                    .copied()
                    .skip_while(|arg| *arg != sub)
                    .skip(1)
                    .collect();
                list_kind(repo_root, kind, &remaining);
            } else {
                eprintln!("Unknown docs subcommand: {sub}");
                eprintln!("Run `repo docs --help` for usage.");
                std::process::exit(1);
            }
        }
        None => list_all(repo_root, json_output),
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
    <QUERY>            Show one document by filename, stem, or title prefix
    --status <STATUS>  Filter by status (e.g. proposal, draft, active, accepted)
    --sort <SORT>      Sort by date, status, title, or progress
    --limit <N>        Limit the number of listed documents
    --details <MODE>   Expand phase details: none, incomplete, or all
    --interactive      Choose one document interactively from a TTY
    --json             Emit machine-readable JSON instead of a table
    -h, --help         Print this help message

Scans {location} for plan documents.",
        sub = kind.subdir(),
        label = kind.label(),
        location = location,
    );
}

pub fn list_all(repo_root: &Path, json_output: bool) {
    if json_output {
        let payload: Vec<serde_json::Value> = ALL_KINDS
            .iter()
            .map(|kind| {
                let docs = resolve_docs(repo_root, *kind).unwrap_or_default();
                serde_json::json!({
                    "kind": kind.subdir(),
                    "count": docs.len(),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

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

    let options = match parse_list_options(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
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

    let mut filtered: Vec<&Doc> = docs
        .iter()
        .filter(|d| {
            if let Some(ref filter) = options.status_filter {
                d.status.to_lowercase() == *filter
            } else {
                true
            }
        })
        .collect();

    sort_docs(&mut filtered, options.sort);

    if let Some(query) = options.query.as_deref() {
        if let Some(doc) = find_doc(&filtered, query) {
            filtered = vec![doc];
        } else {
            eprintln!("No {} matched `{query}`.", kind.label());
            eprintln!("Run `repo docs {} --help` for usage.", kind.subdir());
            std::process::exit(1);
        }
    } else if options.interactive {
        let Some(doc) = pick_doc_interactively(kind, &filtered) else {
            return;
        };
        filtered = vec![doc];
    }

    if let Some(limit) = options.limit {
        filtered.truncate(limit);
    }

    if filtered.is_empty() {
        if options.json_output {
            println!("[]");
            return;
        }
        println!("No {}s match the given filter.", kind.label());
        return;
    }

    if options.json_output {
        print_json(&filtered);
        return;
    }

    let details = if options.query.is_some() || options.interactive {
        DetailsMode::All
    } else {
        options.details
    };

    print_table(kind, &filtered, details);
}

fn parse_list_options(args: &[&str]) -> Result<ListOptions, String> {
    let mut query = None;
    let mut status_filter = None;
    let mut json_output = false;
    let mut sort = SortMode::Date;
    let mut limit = None;
    let mut details = DetailsMode::None;
    let mut interactive = false;

    let mut i = 0usize;
    while i < args.len() {
        match args[i] {
            "--json" => {
                json_output = true;
                i += 1;
            }
            "--interactive" => {
                interactive = true;
                i += 1;
            }
            "--status" => {
                let value = args.get(i + 1).ok_or("Missing value for --status. Usage: --status <STATUS>")?;
                status_filter = Some((*value).to_lowercase());
                i += 2;
            }
            "--sort" => {
                let value = args.get(i + 1).ok_or("Missing value for --sort. Usage: --sort <date|status|title|progress>")?;
                sort = match *value {
                    "date" => SortMode::Date,
                    "status" => SortMode::Status,
                    "title" => SortMode::Title,
                    "progress" => SortMode::Progress,
                    other => return Err(format!("Unknown sort mode: {other}")),
                };
                i += 2;
            }
            "--limit" => {
                let value = args.get(i + 1).ok_or("Missing value for --limit. Usage: --limit <N>")?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid limit: {value}"))?;
                limit = Some(parsed);
                i += 2;
            }
            "--details" => {
                let value = args.get(i + 1).ok_or("Missing value for --details. Usage: --details <none|incomplete|all>")?;
                details = match *value {
                    "none" => DetailsMode::None,
                    "incomplete" => DetailsMode::Incomplete,
                    "all" => DetailsMode::All,
                    other => return Err(format!("Unknown details mode: {other}")),
                };
                i += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("Unknown docs option: {value}"));
            }
            value => {
                if query.is_some() {
                    return Err("Only one docs query is supported at a time.".into());
                }
                query = Some(value.to_string());
                i += 1;
            }
        }
    }

    if interactive && json_output {
        return Err("`--interactive` cannot be combined with `--json`.".into());
    }
    if interactive && query.is_some() {
        return Err("`--interactive` cannot be combined with a docs query.".into());
    }

    Ok(ListOptions {
        query,
        status_filter,
        json_output,
        sort,
        limit,
        details,
        interactive,
    })
}

fn sort_docs(docs: &mut Vec<&Doc>, sort: SortMode) {
    match sort {
        SortMode::Date => docs.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title))),
        SortMode::Status => docs.sort_by(|a, b| a.status.cmp(&b.status).then_with(|| a.title.cmp(&b.title))),
        SortMode::Title => docs.sort_by(|a, b| a.title.cmp(&b.title)),
        SortMode::Progress => docs.sort_by(|a, b| {
            plan_score(b)
                .cmp(&plan_score(a))
                .then_with(|| b.date.cmp(&a.date))
                .then_with(|| a.title.cmp(&b.title))
        }),
    }
}

fn find_doc<'a>(docs: &[&'a Doc], query: &str) -> Option<&'a Doc> {
    let needle = query.to_lowercase();
    docs.iter()
        .copied()
        .find(|doc| doc.file.eq_ignore_ascii_case(query))
        .or_else(|| {
            docs.iter().copied().find(|doc| {
                Path::new(&doc.file)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| stem.eq_ignore_ascii_case(query))
            })
        })
        .or_else(|| {
            docs.iter().copied().find(|doc| {
                doc.title.to_lowercase().starts_with(&needle) || doc.file.to_lowercase().starts_with(&needle)
            })
        })
}

fn pick_doc_interactively<'a>(kind: DocKind, docs: &[&'a Doc]) -> Option<&'a Doc> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        eprintln!("`--interactive` requires a TTY.");
        std::process::exit(1);
    }

    println!("{}", bold(&format!("Select a {}:", kind.label())));
    for (index, doc) in docs.iter().enumerate() {
        println!(
            "  {}. {} {}",
            index + 1,
            doc.file,
            dim(&format!("({})", doc.title))
        );
    }
    print!("> ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("Failed to read selection.");
        std::process::exit(1);
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let choice = trimmed
        .parse::<usize>()
        .ok()
        .and_then(|index| docs.get(index.saturating_sub(1)).copied());
    if choice.is_none() {
        eprintln!("Invalid selection: {trimmed}");
        std::process::exit(1);
    }
    choice
}

fn should_expand_doc(details: DetailsMode, doc: &Doc) -> bool {
    match details {
        DetailsMode::None => false,
        DetailsMode::All => true,
        DetailsMode::Incomplete => {
            let progress = plan_progress(&doc.phases);
            progress.total_phases > 0 && progress.complete_phases < progress.total_phases
        }
    }
}

fn plan_score(doc: &Doc) -> (usize, usize) {
    let progress = plan_progress(&doc.phases);
    (progress.complete_phases, progress.done_tasks)
}

// ── Resolve docs per kind ──────────────────────────────────────────

fn resolve_docs(repo_root: &Path, kind: DocKind) -> Result<Vec<Doc>, String> {
    if let DocKind::Plans = kind {
        let storage = repo_root.join(".repo").join("storage");
        if !storage.is_dir() {
            return Ok(Vec::new());
        }
        Ok(scan_storage_plans(&storage))
    } else {
        let dir = repo_root.join("_docs").join(kind.subdir());
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        scan_docs(&dir)
    }
}

// ── Scanning _docs/ (designs, adrs, references) ────────────────────

/// # Errors
///
/// Returns an error when the target directory cannot be read or an entry
/// cannot be loaded from disk.
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

fn scan_storage_plans(storage_dir: &Path) -> Vec<Doc> {
    let mut docs = Vec::new();
    walk_storage_dir(storage_dir, &mut docs);
    docs.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));
    docs
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
        .or_else(|| (!plan_id.is_empty()).then(|| plan_id.replace('-', " ")))
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
        .map_or_else(|| "\u{2014}".into(), |d| {
            // Extract just the date part from ISO 8601.
            if d.len() >= 10 {
                d[..10].to_string()
            } else {
                d.to_string()
            }
        });

    // Extract steps as pseudo-phases grouped by execution order.
    let mut phases = extract_json_phases(obj);
    let status = derive_plan_status(obj, &phases);

    // If the plan is explicitly marked complete at the metadata level, reflect
    // that in per-phase progress — individual step valDone fields are often
    // left at 0 even after the plan finishes.
    if status == "complete" {
        for phase in &mut phases {
            if phase.done < phase.total {
                phase.done = phase.total;
            }
        }
    }

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
    // "complete" always wins — even if an explicit status is set.
    if !phases.is_empty() {
        let all_complete = phases.iter().all(|p| p.total > 0 && p.done == p.total);
        if all_complete {
            return "complete".into();
        }
    }

    // Explicit metadata.status overrides all derived logic.
    if let Some(explicit) = obj
        .get("metadata")
        .and_then(|m| m.get("status"))
        .and_then(|v| v.as_str())
    {
        return explicit.to_string();
    }

    if !phases.is_empty() {
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
    let Some(steps) = obj.get("steps").and_then(|v| v.as_array()) else {
        return Vec::new();
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

        let (done, total) = step.copied().map_or((0, 1), step_progress);

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
        budget.get("valReq").and_then(serde_json::Value::as_u64),
        budget.get("valDone").and_then(serde_json::Value::as_u64),
    ) {
        return (
            usize::try_from(done).unwrap_or(usize::MAX),
            usize::try_from(total).unwrap_or(usize::MAX),
        );
    }

    if let (Some(total), Some(done)) = (
        budget.get("required").and_then(serde_json::Value::as_u64),
        budget.get("performed").and_then(serde_json::Value::as_u64),
    ) {
        return (
            usize::try_from(done).unwrap_or(usize::MAX),
            usize::try_from(total).unwrap_or(usize::MAX),
        );
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

            map.insert(key.clone(), value.to_string());
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

#[expect(clippy::too_many_lines)]
fn print_table(kind: DocKind, docs: &[&Doc], details: DetailsMode) {
    let has_phases = matches!(kind, DocKind::Plans) && docs.iter().any(|d| !d.phases.is_empty());

    let hdr_file = "FILE";
    let hdr_title = "TITLE";
    let hdr_version = "VERSION";
    let hdr_status = "STATUS";
    let hdr_date = "DATE";

    let w_file = docs
        .iter()
        .map(|d| d.file.len())
        .max()
        .unwrap_or(0)
        .max(hdr_file.len());
    let w_title = docs
        .iter()
        .map(|d| d.title.len())
        .max()
        .unwrap_or(0)
        .max(hdr_title.len());
    let w_ver = docs
        .iter()
        .map(|d| d.version.len())
        .max()
        .unwrap_or(0)
        .max(if matches!(kind, DocKind::Plans) { 0 } else { hdr_version.len() });
    let w_status = docs
        .iter()
        .map(|d| d.status.len())
        .max()
        .unwrap_or(0)
        .max(hdr_status.len());
    let w_date = docs
        .iter()
        .map(|d| d.date.len())
        .max()
        .unwrap_or(0)
        .max(hdr_date.len());

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
    if has_phases && matches!(kind, DocKind::Plans) {
        println!(
            "  {:<w_file$}  {:<w_title$}  {:<w_status$}  {:<w_date$}  {:<w_progress$}",
            bold(hdr_file),
            bold(hdr_title),
            bold(hdr_status),
            bold(hdr_date),
            bold(progress_hdr),
        );
        println!(
            "  {}  {}  {}  {}  {}",
            dim(&"\u{2500}".repeat(w_file)),
            dim(&"\u{2500}".repeat(w_title)),
            dim(&"\u{2500}".repeat(w_status)),
            dim(&"\u{2500}".repeat(w_date)),
            dim(&"\u{2500}".repeat(w_progress)),
        );
    } else if has_phases {
        println!(
            "  {:<w_file$}  {:<w_title$}  {:<w_ver$}  {:<w_status$}  {:<w_date$}  {:<w_progress$}",
            bold(hdr_file),
            bold(hdr_title),
            bold(hdr_version),
            bold(hdr_status),
            bold(hdr_date),
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
            bold(hdr_file),
            bold(hdr_title),
            bold(hdr_version),
            bold(hdr_status),
            bold(hdr_date),
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

        if has_phases && matches!(kind, DocKind::Plans) {
            let progress = format_progress_summary(&doc.phases);
            let progress_colored = color_progress_summary(&doc.phases);
            let progress_padding = w_progress.saturating_sub(progress.len());
            println!(
                "  {:<w_file$}  {:<w_title$}  {}{:>spad$}  {:<w_date$}  {}{:>ppad$}",
                doc.file,
                doc.title,
                status_display,
                "",
                doc.date,
                progress_colored,
                "",
                spad = status_padding,
                ppad = progress_padding,
            );
        } else if has_phases {
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
    if has_phases && details != DetailsMode::None {
        for doc in docs {
            if doc.phases.is_empty() || !should_expand_doc(details, doc) {
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
    if matches!(kind, DocKind::Plans) && details == DetailsMode::None {
        println!("  Run {} to inspect one plan.", dim("repo docs plans <query>"));
        println!(
            "  Run {} to expand active plans.",
            dim("repo docs plans --details incomplete")
        );
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("therepo-docs-{label}-{nanos}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(root: &std::path::Path, relative: &str, content: &str) {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    mod doc_kind {
        use super::*;

        #[test]
        fn parse_accepts_known_names_and_aliases() {
            assert!(matches!(DocKind::parse("plans"), Some(DocKind::Plans)));
            assert!(matches!(DocKind::parse("designs"), Some(DocKind::Designs)));
            assert!(matches!(DocKind::parse("adrs"), Some(DocKind::Adrs)));
            assert!(matches!(DocKind::parse("refs"), Some(DocKind::References)));
            assert!(matches!(
                DocKind::parse("references"),
                Some(DocKind::References)
            ));
            assert!(DocKind::parse("unknown").is_none());
        }
    }

    mod parsing {
        use super::*;

        #[test]
        fn parse_frontmatter_extracts_fields_and_defaults() {
            let path = std::path::Path::new("2026-03-21-example.md");
            let doc = parse_frontmatter(
                "---\n\
                 title: \"Example Plan\"\n\
                 version: '1.2.3'\n\
                 status: accepted\n\
                 date: 2026-03-21\n\
                 ---\n\
                 body\n",
                path,
            )
            .expect("expected frontmatter");

            assert_eq!(doc.file, "2026-03-21-example.md");
            assert_eq!(doc.title, "Example Plan");
            assert_eq!(doc.version, "1.2.3");
            assert_eq!(doc.status, "accepted");
            assert_eq!(doc.date, "2026-03-21");
            assert!(doc.phases.is_empty());
        }

        #[test]
        fn parse_frontmatter_rejects_missing_title() {
            let path = std::path::Path::new("missing-title.md");
            assert!(parse_frontmatter("---\nstatus: draft\n---\nbody\n", path).is_none());
        }

        #[test]
        fn parse_plan_phases_groups_headings_and_counts_tasks() {
            let phases = parse_plan_phases(
                "\
                 # Intro\n\
                 ## Phase 1 - Setup\n\
                 - [x] one\n\
                 - [ ] two\n\
                 ## Phase 2 - Finish\n\
                 - [x] done\n",
            );

            assert_eq!(phases.len(), 2);
            assert_eq!(phases[0].name, "Phase 1 - Setup");
            assert_eq!(phases[0].done, 1);
            assert_eq!(phases[0].total, 2);
            assert_eq!(phases[1].name, "Phase 2 - Finish");
            assert_eq!(phases[1].done, 1);
            assert_eq!(phases[1].total, 1);
        }

        #[test]
        fn truncate_title_prefers_sentence_boundaries_and_word_wrap() {
            assert_eq!(
                truncate_title("Short title", 72),
                "Short title".to_string()
            );
            assert_eq!(
                truncate_title(
                    "A concise outcome: keep the first clause when truncating the title",
                    40
                ),
                "A concise outcome".to_string()
            );
        }
    }

    mod json {
        use super::*;

        #[test]
        fn parse_plan_json_uses_progress_and_status_from_steps() {
            let path = std::path::Path::new("plan.json");
            let doc = parse_plan_json(
                r#"{
                    "schemaVersion": "1",
                    "metadata": {
                        "planId": "release-checklist",
                        "version": "2.0.0",
                        "updatedAt": "2026-03-21T12:34:56Z"
                    },
                    "problem": {
                        "successOutcome": "Ship a tighter release flow without breaking installs"
                    },
                    "steps": [
                        {
                            "id": "setup",
                            "title": "Setup",
                            "size": "S",
                            "validationBudget": { "valReq": 2, "valDone": 2 }
                        },
                        {
                            "id": "ship",
                            "title": "Ship",
                            "size": "M",
                            "validationBudget": { "required": 3, "performed": 1 }
                        }
                    ],
                    "executionOrder": {
                        "sequence": ["setup", "ship"],
                        "parallelizableGroups": [["setup", "ship"]]
                    }
                }"#,
                path,
            )
            .expect("expected json doc");

            assert_eq!(doc.file, "plan.json");
            assert_eq!(
                doc.title,
                "Ship a tighter release flow without breaking installs"
            );
            assert_eq!(doc.version, "2.0.0");
            assert_eq!(doc.status, "active");
            assert_eq!(doc.date, "2026-03-21");
            assert_eq!(doc.phases.len(), 2);
            assert!(doc.phases[0].name.starts_with("┌ [S] Setup"));
            assert!(doc.phases[1].name.starts_with("└ [M] Ship"));
            assert_eq!(doc.phases[0].done, 2);
            assert_eq!(doc.phases[0].total, 2);
        }

        #[test]
        fn derive_plan_status_prefers_phase_completion_over_history() {
            let mut obj = serde_json::Map::new();
            obj.insert("metadata".into(), serde_json::json!({"versionHistory": [1]}));
            let phases = vec![PlanPhase {
                name: "setup".into(),
                done: 1,
                total: 1,
            }];

            assert_eq!(derive_plan_status(&obj, &phases), "complete");
        }
    }

    mod scanning {
        use super::*;

        #[test]
        fn scan_docs_sorts_markdown_docs_by_date_then_title() {
            let dir = temp_dir("scan-docs");
            write_file(
                &dir,
                "b.md",
                "---\n\
                 title: Beta\n\
                 version: 1.0.0\n\
                 status: draft\n\
                 date: 2026-03-20\n\
                 ---\n",
            );
            write_file(
                &dir,
                "a.md",
                "---\n\
                 title: Alpha\n\
                 version: 1.0.0\n\
                 status: accepted\n\
                 date: 2026-03-21\n\
                 ---\n",
            );
            write_file(&dir, "ignore.txt", "ignored");

            let docs = scan_docs(&dir).expect("expected docs");

            assert_eq!(docs.len(), 2);
            assert_eq!(docs[0].file, "a.md");
            assert_eq!(docs[0].title, "Alpha");
            assert_eq!(docs[1].file, "b.md");
            assert_eq!(docs[1].title, "Beta");

            fs::remove_dir_all(dir).ok();
        }
    }

    mod progress {
        use super::*;

        fn phase(done: usize, total: usize) -> PlanPhase {
            PlanPhase { name: "p".into(), done, total }
        }

        // ── phase_status ────────────────────────────────────────────

        #[test]
        fn phase_status_unknown_when_no_tasks() {
            assert_eq!(phase_status(&phase(0, 0)), "unknown");
        }

        #[test]
        fn phase_status_done_when_all_complete() {
            assert_eq!(phase_status(&phase(3, 3)), "done");
        }

        #[test]
        fn phase_status_partial_when_some_done() {
            assert_eq!(phase_status(&phase(1, 3)), "partial");
        }

        #[test]
        fn phase_status_pending_when_none_done() {
            assert_eq!(phase_status(&phase(0, 3)), "pending");
        }

        // ── step_progress ───────────────────────────────────────────

        #[test]
        fn step_progress_returns_zero_one_when_no_budget() {
            let step = serde_json::json!({ "id": "a" });
            assert_eq!(step_progress(&step), (0, 1));
        }

        #[test]
        fn step_progress_reads_val_req_and_val_done() {
            let step = serde_json::json!({
                "validationBudget": { "valReq": 4, "valDone": 2 }
            });
            assert_eq!(step_progress(&step), (2, 4));
        }

        #[test]
        fn step_progress_reads_required_and_performed() {
            let step = serde_json::json!({
                "validationBudget": { "required": 3, "performed": 1 }
            });
            assert_eq!(step_progress(&step), (1, 3));
        }

        // ── plan_progress ───────────────────────────────────────────

        #[test]
        fn plan_progress_sums_across_phases() {
            let phases = vec![phase(2, 3), phase(1, 1)];
            let p = plan_progress(&phases);
            assert_eq!(p.total_phases, 2);
            assert_eq!(p.complete_phases, 1);
            assert_eq!(p.done_tasks, 3);
            assert_eq!(p.total_tasks, 4);
        }

        // ── format_progress_summary ─────────────────────────────────

        #[test]
        fn format_progress_summary_returns_dash_for_empty() {
            assert_eq!(format_progress_summary(&[]), "\u{2014}");
        }

        #[test]
        fn format_progress_summary_formats_counts() {
            let phases = vec![phase(2, 2), phase(0, 1)];
            let s = format_progress_summary(&phases);
            assert!(s.contains("1/2 phases"), "got: {s}");
            assert!(s.contains("2/3 tasks"), "got: {s}");
        }

        // ── progress_bar ────────────────────────────────────────────

        #[test]
        fn progress_bar_returns_empty_bar_for_zero_total() {
            let bar = progress_bar(0, 0, 5);
            // Should contain spaces (dimmed empty bar), no filled blocks.
            assert!(bar.contains('['), "got: {bar}");
        }

        #[test]
        fn progress_bar_fully_filled_for_complete() {
            let bar = progress_bar(4, 4, 4);
            assert!(bar.contains('\u{2588}'), "expected filled blocks in: {bar}");
        }

        #[test]
        fn progress_bar_partial_for_in_progress() {
            let bar = progress_bar(2, 4, 4);
            assert!(bar.contains('\u{2588}'), "expected some filled: {bar}");
            assert!(bar.contains('\u{2591}'), "expected some empty: {bar}");
        }

        #[test]
        fn progress_bar_all_empty_for_zero_done() {
            let bar = progress_bar(0, 4, 4);
            assert!(bar.contains('\u{2591}'), "expected empty blocks in: {bar}");
        }

        // ── truncate_title word-boundary ────────────────────────────

        #[test]
        fn truncate_title_breaks_at_word_boundary() {
            let long = "one two three four five six seven eight nine ten eleven";
            let truncated = truncate_title(long, 20);
            assert!(truncated.ends_with("..."), "got: {truncated}");
            assert!(truncated.len() <= 23, "got: {truncated}"); // max + "..."
        }

        // ── color_progress_summary ──────────────────────────────────

        #[test]
        fn color_progress_summary_returns_dash_for_empty_phases() {
            let result = color_progress_summary(&[]);
            assert!(result.contains('\u{2014}'), "expected em-dash in: {result}");
        }

        #[test]
        fn color_progress_summary_all_complete() {
            let phases = vec![phase(2, 2), phase(1, 1)];
            let result = color_progress_summary(&phases);
            assert!(result.contains("2/2 phases"), "got: {result}");
            assert!(result.contains("3/3 tasks"), "got: {result}");
        }

        #[test]
        fn color_progress_summary_partial_phases() {
            let phases = vec![phase(1, 2), phase(0, 3)];
            let result = color_progress_summary(&phases);
            assert!(result.contains("0/2 phases"), "got: {result}");
            assert!(result.contains("1/5 tasks"), "got: {result}");
        }

        #[test]
        fn color_progress_summary_zero_done() {
            let phases = vec![phase(0, 2), phase(0, 3)];
            let result = color_progress_summary(&phases);
            assert!(result.contains("0/2 phases"), "got: {result}");
        }
    }
}
