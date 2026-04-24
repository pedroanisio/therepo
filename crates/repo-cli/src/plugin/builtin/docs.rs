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

pub fn run(repo_root: &Path, args: &[&str]) -> i32 {
    let subcommand = args.iter().copied().find(|arg| !arg.starts_with('-'));
    let json_output = args.contains(&"--json");

    if args.iter().any(|a| *a == "--help" || *a == "-h") {
        if let Some(kind) = subcommand.and_then(DocKind::parse) {
            print_subcommand_help(kind);
        } else {
            print_help();
        }
        return 0;
    }

    if let Some(sub) = subcommand {
        if let Some(kind) = DocKind::parse(sub) {
            let remaining: Vec<&str> = args
                .iter()
                .copied()
                .skip_while(|arg| *arg != sub)
                .skip(1)
                .collect();
            list_kind(repo_root, kind, &remaining)
        } else {
            eprintln!("Unknown docs subcommand: {sub}");
            eprintln!("Run `repo docs --help` for usage.");
            1
        }
    } else {
        list_all(repo_root, json_output);
        0
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

fn list_kind(repo_root: &Path, kind: DocKind, args: &[&str]) -> i32 {
    let docs = match resolve_docs(repo_root, kind) {
        Ok(docs) => docs,
        Err(e) => {
            eprintln!("Error scanning {}: {e}", kind.subdir());
            return 1;
        }
    };

    let options = match parse_list_options(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };

    if docs.is_empty() {
        let location = match kind {
            DocKind::Plans => ".repo/storage/".to_string(),
            _ => format!("_docs/{}/", kind.subdir()),
        };
        println!("No {}s found in {}", kind.label(), location);
        return 0;
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
            return 1;
        }
    } else if options.interactive {
        let Some(doc) = (match pick_doc_interactively(kind, &filtered) {
            Ok(doc) => doc,
            Err(message) => {
                eprintln!("{message}");
                return 1;
            }
        }) else {
            return 0;
        };
        filtered = vec![doc];
    }

    if let Some(limit) = options.limit {
        filtered.truncate(limit);
    }

    if filtered.is_empty() {
        if options.json_output {
            println!("[]");
            return 0;
        }
        println!("No {}s match the given filter.", kind.label());
        return 0;
    }

    if options.json_output {
        return match print_json(&filtered) {
            Ok(()) => 0,
            Err(message) => {
                eprintln!("{message}");
                1
            }
        };
    }

    let details = if options.query.is_some() || options.interactive {
        DetailsMode::All
    } else {
        options.details
    };

    print_table(kind, &filtered, details);
    0
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

fn pick_doc_interactively<'a>(kind: DocKind, docs: &[&'a Doc]) -> Result<Option<&'a Doc>, String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("`--interactive` requires a TTY.".into());
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
        return Err("Failed to read selection.".into());
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let choice = trimmed
        .parse::<usize>()
        .ok()
        .and_then(|index| docs.get(index.saturating_sub(1)).copied());
    if choice.is_none() {
        return Err(format!("Invalid selection: {trimmed}"));
    }
    Ok(choice)
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

fn print_json(docs: &[&Doc]) -> Result<(), String> {
    let payload: Vec<JsonDoc<'_>> = docs.iter().map(|doc| to_json_doc(doc)).collect();
    match serde_json::to_string_pretty(&payload) {
        Ok(json) => {
            println!("{json}");
            Ok(())
        }
        Err(err) => Err(format!("Failed to serialize docs as JSON: {err}")),
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

    // ── parse_list_options ──────────────────────────────────────────

    mod parse_options {
        use super::*;

        #[test]
        fn empty_args_returns_defaults() {
            let opts = parse_list_options(&[]).unwrap();
            assert!(opts.query.is_none());
            assert!(opts.status_filter.is_none());
            assert!(!opts.json_output);
            assert_eq!(opts.sort, SortMode::Date);
            assert!(opts.limit.is_none());
            assert_eq!(opts.details, DetailsMode::None);
            assert!(!opts.interactive);
        }

        #[test]
        fn json_flag() {
            let opts = parse_list_options(&["--json"]).unwrap();
            assert!(opts.json_output);
        }

        #[test]
        fn interactive_flag() {
            let opts = parse_list_options(&["--interactive"]).unwrap();
            assert!(opts.interactive);
        }

        #[test]
        fn status_filter() {
            let opts = parse_list_options(&["--status", "Draft"]).unwrap();
            assert_eq!(opts.status_filter.as_deref(), Some("draft"));
        }

        #[test]
        fn status_missing_value() {
            assert!(parse_list_options(&["--status"]).is_err());
        }

        #[test]
        fn sort_all_modes() {
            for (input, expected) in [
                ("date", SortMode::Date),
                ("status", SortMode::Status),
                ("title", SortMode::Title),
                ("progress", SortMode::Progress),
            ] {
                let opts = parse_list_options(&["--sort", input]).unwrap();
                assert_eq!(opts.sort, expected);
            }
        }

        #[test]
        fn sort_invalid() {
            assert!(parse_list_options(&["--sort", "unknown"]).is_err());
        }

        #[test]
        fn sort_missing_value() {
            assert!(parse_list_options(&["--sort"]).is_err());
        }

        #[test]
        fn limit_valid() {
            let opts = parse_list_options(&["--limit", "5"]).unwrap();
            assert_eq!(opts.limit, Some(5));
        }

        #[test]
        fn limit_invalid() {
            assert!(parse_list_options(&["--limit", "abc"]).is_err());
        }

        #[test]
        fn limit_missing_value() {
            assert!(parse_list_options(&["--limit"]).is_err());
        }

        #[test]
        fn details_all_modes() {
            for (input, expected) in [
                ("none", DetailsMode::None),
                ("incomplete", DetailsMode::Incomplete),
                ("all", DetailsMode::All),
            ] {
                let opts = parse_list_options(&["--details", input]).unwrap();
                assert_eq!(opts.details, expected);
            }
        }

        #[test]
        fn details_invalid() {
            assert!(parse_list_options(&["--details", "bad"]).is_err());
        }

        #[test]
        fn details_missing_value() {
            assert!(parse_list_options(&["--details"]).is_err());
        }

        #[test]
        fn unknown_flag_rejected() {
            assert!(parse_list_options(&["--foo"]).is_err());
        }

        #[test]
        fn duplicate_queries_rejected() {
            assert!(parse_list_options(&["query1", "query2"]).is_err());
        }

        #[test]
        fn interactive_with_json_rejected() {
            assert!(parse_list_options(&["--interactive", "--json"]).is_err());
        }

        #[test]
        fn interactive_with_query_rejected() {
            assert!(parse_list_options(&["--interactive", "myquery"]).is_err());
        }

        #[test]
        fn query_is_captured() {
            let opts = parse_list_options(&["my-plan"]).unwrap();
            assert_eq!(opts.query.as_deref(), Some("my-plan"));
        }
    }

    // ── extract_json_phases ────────────────────────────────────────

    mod extract_phases {
        use super::*;

        #[test]
        fn no_steps_returns_empty() {
            let obj: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            assert!(extract_json_phases(&obj).is_empty());
        }

        #[test]
        fn empty_steps_returns_empty() {
            let obj: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(r#"{"steps": []}"#).unwrap();
            assert!(extract_json_phases(&obj).is_empty());
        }

        #[test]
        fn steps_without_execution_order_uses_step_order() {
            let obj: serde_json::Map<String, serde_json::Value> = serde_json::from_str(
                r#"{
                    "steps": [
                        {"id": "a", "title": "Alpha", "size": "S", "validationBudget": {"valReq": 2, "valDone": 1}},
                        {"id": "b", "title": "Beta", "size": "M"}
                    ]
                }"#,
            )
            .unwrap();
            let phases = extract_json_phases(&obj);
            assert_eq!(phases.len(), 2);
            assert!(phases[0].name.contains("Alpha"));
            assert!(phases[0].name.contains("[S]"));
            assert_eq!(phases[0].done, 1);
            assert_eq!(phases[0].total, 2);
            assert!(phases[1].name.contains("Beta"));
            assert_eq!(phases[1].done, 0);
            assert_eq!(phases[1].total, 1);
        }

        #[test]
        fn parallel_groups_annotated() {
            let obj: serde_json::Map<String, serde_json::Value> = serde_json::from_str(
                r#"{
                    "steps": [
                        {"id": "a", "title": "A", "size": "S"},
                        {"id": "b", "title": "B", "size": "M"},
                        {"id": "c", "title": "C", "size": "L"}
                    ],
                    "executionOrder": {
                        "sequence": ["a", "b", "c"],
                        "parallelizableGroups": [["a", "b"]]
                    }
                }"#,
            )
            .unwrap();
            let phases = extract_json_phases(&obj);
            assert_eq!(phases.len(), 3);
            assert!(phases[0].name.starts_with("\u{250c}"), "first in group: {}", phases[0].name);
            assert!(phases[1].name.starts_with("\u{2514}"), "last in group: {}", phases[1].name);
            // c is not in a parallel group
            assert!(phases[2].name.starts_with("[L]"), "solo: {}", phases[2].name);
        }

        #[test]
        fn three_way_parallel_group() {
            let obj: serde_json::Map<String, serde_json::Value> = serde_json::from_str(
                r#"{
                    "steps": [
                        {"id": "a", "title": "A", "size": "S"},
                        {"id": "b", "title": "B", "size": "M"},
                        {"id": "c", "title": "C", "size": "L"}
                    ],
                    "executionOrder": {
                        "sequence": ["a", "b", "c"],
                        "parallelizableGroups": [["a", "b", "c"]]
                    }
                }"#,
            )
            .unwrap();
            let phases = extract_json_phases(&obj);
            assert!(phases[0].name.starts_with("\u{250c}"), "first: {}", phases[0].name);
            assert!(phases[1].name.starts_with("\u{251c}"), "middle: {}", phases[1].name);
            assert!(phases[2].name.starts_with("\u{2514}"), "last: {}", phases[2].name);
        }
    }

    // ── sort_docs ──────────────────────────────────────────────────

    mod sort {
        use super::*;

        fn doc(title: &str, date: &str, status: &str, done: usize, total: usize) -> Doc {
            Doc {
                file: format!("{title}.md"),
                title: title.into(),
                version: String::new(),
                status: status.into(),
                date: date.into(),
                phases: vec![PlanPhase { name: "p".into(), done, total }],
            }
        }

        #[test]
        fn sort_by_date() {
            let a = doc("Alpha", "2026-01-01", "draft", 0, 1);
            let b = doc("Beta", "2026-02-01", "draft", 0, 1);
            let mut docs: Vec<&Doc> = vec![&a, &b];
            sort_docs(&mut docs, SortMode::Date);
            assert_eq!(docs[0].title, "Beta");
            assert_eq!(docs[1].title, "Alpha");
        }

        #[test]
        fn sort_by_status() {
            let a = doc("Alpha", "2026-01-01", "draft", 0, 1);
            let b = doc("Beta", "2026-01-01", "accepted", 0, 1);
            let mut docs: Vec<&Doc> = vec![&a, &b];
            sort_docs(&mut docs, SortMode::Status);
            assert_eq!(docs[0].title, "Beta");  // "accepted" < "draft"
            assert_eq!(docs[1].title, "Alpha");
        }

        #[test]
        fn sort_by_title() {
            let a = doc("Zulu", "2026-01-01", "draft", 0, 1);
            let b = doc("Alpha", "2026-01-01", "draft", 0, 1);
            let mut docs: Vec<&Doc> = vec![&a, &b];
            sort_docs(&mut docs, SortMode::Title);
            assert_eq!(docs[0].title, "Alpha");
            assert_eq!(docs[1].title, "Zulu");
        }

        #[test]
        fn sort_by_progress() {
            let a = doc("Low", "2026-01-01", "active", 0, 3);
            let b = doc("High", "2026-01-01", "active", 3, 3);
            let mut docs: Vec<&Doc> = vec![&a, &b];
            sort_docs(&mut docs, SortMode::Progress);
            assert_eq!(docs[0].title, "High");
            assert_eq!(docs[1].title, "Low");
        }
    }

    // ── find_doc ───────────────────────────────────────────────────

    mod find {
        use super::*;

        fn doc(file: &str, title: &str) -> Doc {
            Doc {
                file: file.into(),
                title: title.into(),
                version: String::new(),
                status: "draft".into(),
                date: "2026-01-01".into(),
                phases: Vec::new(),
            }
        }

        #[test]
        fn exact_filename_match() {
            let d = doc("plan.md", "My Plan");
            let docs: Vec<&Doc> = vec![&d];
            assert_eq!(find_doc(&docs, "plan.md").unwrap().title, "My Plan");
        }

        #[test]
        fn stem_match() {
            let d = doc("release-plan.md", "Release Plan");
            let docs: Vec<&Doc> = vec![&d];
            assert_eq!(find_doc(&docs, "release-plan").unwrap().title, "Release Plan");
        }

        #[test]
        fn prefix_match_title() {
            let d = doc("plan.md", "Release Checklist v2");
            let docs: Vec<&Doc> = vec![&d];
            assert_eq!(find_doc(&docs, "release").unwrap().title, "Release Checklist v2");
        }

        #[test]
        fn prefix_match_file() {
            let d = doc("release-v2.md", "Some Title");
            let docs: Vec<&Doc> = vec![&d];
            assert_eq!(find_doc(&docs, "release").unwrap().title, "Some Title");
        }

        #[test]
        fn no_match_returns_none() {
            let d = doc("plan.md", "My Plan");
            let docs: Vec<&Doc> = vec![&d];
            assert!(find_doc(&docs, "nonexistent").is_none());
        }
    }

    // ── should_expand_doc ──────────────────────────────────────────

    mod expand {
        use super::*;

        fn doc_with_phases(done: usize, total: usize) -> Doc {
            Doc {
                file: "test.md".into(),
                title: "Test".into(),
                version: String::new(),
                status: "active".into(),
                date: "2026-01-01".into(),
                phases: vec![PlanPhase { name: "p".into(), done, total }],
            }
        }

        #[test]
        fn none_never_expands() {
            assert!(!should_expand_doc(DetailsMode::None, &doc_with_phases(0, 3)));
        }

        #[test]
        fn all_always_expands() {
            assert!(should_expand_doc(DetailsMode::All, &doc_with_phases(3, 3)));
        }

        #[test]
        fn incomplete_expands_when_not_done() {
            assert!(should_expand_doc(DetailsMode::Incomplete, &doc_with_phases(1, 3)));
        }

        #[test]
        fn incomplete_does_not_expand_when_complete() {
            assert!(!should_expand_doc(DetailsMode::Incomplete, &doc_with_phases(3, 3)));
        }
    }

    // ── to_json_doc ────────────────────────────────────────────────

    mod json_doc {
        use super::*;

        #[test]
        fn round_trip_preserves_fields() {
            let doc = Doc {
                file: "test.json".into(),
                title: "Test Plan".into(),
                version: "1.0.0".into(),
                status: "active".into(),
                date: "2026-04-01".into(),
                phases: vec![
                    PlanPhase { name: "Setup".into(), done: 2, total: 2 },
                    PlanPhase { name: "Ship".into(), done: 0, total: 3 },
                ],
            };
            let json_doc = to_json_doc(&doc);
            assert_eq!(json_doc.file, "test.json");
            assert_eq!(json_doc.title, "Test Plan");
            assert_eq!(json_doc.version, "1.0.0");
            assert_eq!(json_doc.status, "active");
            assert_eq!(json_doc.date, "2026-04-01");
            assert_eq!(json_doc.progress.complete_phases, 1);
            assert_eq!(json_doc.progress.total_phases, 2);
            assert_eq!(json_doc.progress.done_tasks, 2);
            assert_eq!(json_doc.progress.total_tasks, 5);
            assert_eq!(json_doc.phases.len(), 2);
            assert_eq!(json_doc.phases[0].status, "done");
            assert_eq!(json_doc.phases[1].status, "pending");
        }
    }

    // ── parse_yaml_fields ──────────────────────────────────────────

    mod yaml_fields {
        use super::*;

        #[test]
        fn unquoted_value() {
            let fields = parse_yaml_fields("title: My Title");
            assert_eq!(fields.get("title").unwrap(), "My Title");
        }

        #[test]
        fn double_quoted_value() {
            let fields = parse_yaml_fields("title: \"My Title\"");
            assert_eq!(fields.get("title").unwrap(), "My Title");
        }

        #[test]
        fn single_quoted_value() {
            let fields = parse_yaml_fields("version: '1.0.0'");
            assert_eq!(fields.get("version").unwrap(), "1.0.0");
        }

        #[test]
        fn skips_indented_lines() {
            let fields = parse_yaml_fields("title: Hello\n  indented: value");
            assert_eq!(fields.len(), 1);
            assert!(!fields.contains_key("indented"));
        }

        #[test]
        fn skips_empty_lines() {
            let fields = parse_yaml_fields("title: Hello\n\nstatus: draft");
            assert_eq!(fields.len(), 2);
        }

        #[test]
        fn skips_multiline_markers() {
            let fields = parse_yaml_fields("description: >\ntitle: Hello\nblock: |");
            // description and block should be skipped (> and | values)
            assert_eq!(fields.len(), 1);
            assert_eq!(fields.get("title").unwrap(), "Hello");
        }

        #[test]
        fn keys_lowercased() {
            let fields = parse_yaml_fields("Title: Hello");
            assert_eq!(fields.get("title").unwrap(), "Hello");
        }
    }

    // ── try_parse_plan_file ────────────────────────────────────────

    mod plan_file {
        use super::*;

        #[test]
        fn md_with_frontmatter_and_phases() {
            let dir = temp_dir("plan-md");
            write_file(
                &dir,
                "plan.md",
                "---\ntitle: My Plan\nversion: 1.0.0\nstatus: active\ndate: 2026-01-01\n---\n\
                 ## Phase 1 - Setup\n- [x] done\n- [ ] todo\n",
            );
            let doc = try_parse_plan_file(&dir.join("plan.md")).unwrap();
            assert_eq!(doc.title, "My Plan");
            assert_eq!(doc.phases.len(), 1);
            assert_eq!(doc.phases[0].done, 1);
            assert_eq!(doc.phases[0].total, 2);
            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn md_with_phases_but_no_frontmatter() {
            let dir = temp_dir("plan-no-fm");
            write_file(
                &dir,
                "my-plan.md",
                "# Title\n## Phase 1 - Setup\n- [x] done\n",
            );
            let doc = try_parse_plan_file(&dir.join("my-plan.md")).unwrap();
            assert_eq!(doc.title, "my plan");
            assert_eq!(doc.phases.len(), 1);
            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn md_without_frontmatter_or_phases_returns_none() {
            let dir = temp_dir("plan-none");
            write_file(&dir, "notes.md", "# Just some notes\nHello world.\n");
            assert!(try_parse_plan_file(&dir.join("notes.md")).is_none());
            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn json_plan_file() {
            let dir = temp_dir("plan-json");
            write_file(
                &dir,
                "plan.json",
                r#"{
                    "schemaVersion": "1",
                    "metadata": { "planId": "test", "version": "1.0.0", "updatedAt": "2026-01-01T00:00:00Z" },
                    "problem": { "successOutcome": "Test outcome" },
                    "steps": [{"id": "a", "title": "A", "size": "S"}]
                }"#,
            );
            let doc = try_parse_plan_file(&dir.join("plan.json")).unwrap();
            assert_eq!(doc.title, "Test outcome");
            assert_eq!(doc.version, "1.0.0");
            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn unknown_extension_returns_none() {
            let dir = temp_dir("plan-txt");
            write_file(&dir, "plan.txt", "hello");
            assert!(try_parse_plan_file(&dir.join("plan.txt")).is_none());
            fs::remove_dir_all(dir).ok();
        }
    }

    // ── scan_docs (with frontmatter and without) ───────────────────

    mod scan {
        use super::*;

        #[test]
        fn scan_docs_includes_md_without_frontmatter() {
            let dir = temp_dir("scan-nofm");
            write_file(&dir, "no-frontmatter.md", "# Just a heading\nSome body text.\n");
            let docs = scan_docs(&dir).unwrap();
            assert_eq!(docs.len(), 1);
            assert_eq!(docs[0].file, "no-frontmatter.md");
            assert_eq!(docs[0].title, "no frontmatter");
            assert_eq!(docs[0].status, "\u{2014}");
            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn scan_docs_empty_dir() {
            let dir = temp_dir("scan-empty");
            let docs = scan_docs(&dir).unwrap();
            assert!(docs.is_empty());
            fs::remove_dir_all(dir).ok();
        }
    }

    // ── resolve_docs ───────────────────────────────────────────────

    mod resolve {
        use super::*;

        #[test]
        fn plans_uses_storage_dir() {
            let root = temp_dir("resolve-plans");
            // No .repo/storage/ directory -> empty
            let docs = resolve_docs(&root, DocKind::Plans).unwrap();
            assert!(docs.is_empty());
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn designs_uses_docs_dir() {
            let root = temp_dir("resolve-designs");
            // No _docs/designs/ directory -> empty
            let docs = resolve_docs(&root, DocKind::Designs).unwrap();
            assert!(docs.is_empty());
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn plans_with_storage_files() {
            let root = temp_dir("resolve-plans-files");
            write_file(
                &root,
                ".repo/storage/plan.json",
                r#"{
                    "schemaVersion": "1",
                    "metadata": { "planId": "test", "version": "1.0.0", "updatedAt": "2026-01-01T00:00:00Z" },
                    "problem": { "successOutcome": "Test" },
                    "steps": [{"id": "a", "title": "A", "size": "S"}]
                }"#,
            );
            let docs = resolve_docs(&root, DocKind::Plans).unwrap();
            assert_eq!(docs.len(), 1);
            fs::remove_dir_all(root).ok();
        }
    }

    // ── walk_storage_dir ───────────────────────────────────────────

    mod walk {
        use super::*;

        #[test]
        fn nested_dirs_are_traversed() {
            let root = temp_dir("walk-nested");
            write_file(
                &root,
                "sub/plan.json",
                r#"{
                    "schemaVersion": "1",
                    "metadata": { "planId": "nested", "version": "1.0.0", "updatedAt": "2026-01-01T00:00:00Z" },
                    "problem": { "successOutcome": "Nested" },
                    "steps": [{"id": "a", "title": "A", "size": "S"}]
                }"#,
            );
            let mut docs = Vec::new();
            walk_storage_dir(&root, &mut docs);
            assert_eq!(docs.len(), 1);
            assert_eq!(docs[0].title, "Nested");
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn unreadable_dir_does_not_panic() {
            let dir = std::path::PathBuf::from("/nonexistent-path-for-test");
            let mut docs = Vec::new();
            walk_storage_dir(&dir, &mut docs);
            assert!(docs.is_empty());
        }
    }

    // ── print_json ─────────────────────────────────────────────────

    mod print_json_tests {
        use super::*;

        #[test]
        fn produces_valid_json() {
            let doc = Doc {
                file: "test.md".into(),
                title: "Test".into(),
                version: "1.0.0".into(),
                status: "draft".into(),
                date: "2026-01-01".into(),
                phases: Vec::new(),
            };
            let docs: Vec<&Doc> = vec![&doc];
            // Should not return an error
            assert!(print_json(&docs).is_ok());
        }
    }

    // ── scan_storage_plans ─────────────────────────────────────────

    mod storage_plans {
        use super::*;

        #[test]
        fn collects_and_sorts_plans() {
            let dir = temp_dir("storage-plans");
            write_file(
                &dir,
                "a.json",
                r#"{
                    "schemaVersion": "1",
                    "metadata": { "planId": "alpha", "version": "1.0.0", "updatedAt": "2026-01-01T00:00:00Z" },
                    "problem": { "successOutcome": "Alpha" },
                    "steps": [{"id": "a", "title": "A", "size": "S"}]
                }"#,
            );
            write_file(
                &dir,
                "b.json",
                r#"{
                    "schemaVersion": "1",
                    "metadata": { "planId": "beta", "version": "1.0.0", "updatedAt": "2026-02-01T00:00:00Z" },
                    "problem": { "successOutcome": "Beta" },
                    "steps": [{"id": "b", "title": "B", "size": "M"}]
                }"#,
            );
            let plans = scan_storage_plans(&dir);
            assert_eq!(plans.len(), 2);
            // Sorted by date descending
            assert_eq!(plans[0].title, "Beta");
            assert_eq!(plans[1].title, "Alpha");
            fs::remove_dir_all(dir).ok();
        }
    }

    // ── print_table (no-panic smoke tests) ─────────────────────────

    mod table {
        use super::*;

        fn make_doc(file: &str, title: &str, status: &str, phases: Vec<PlanPhase>) -> Doc {
            Doc {
                file: file.into(),
                title: title.into(),
                version: "1.0.0".into(),
                status: status.into(),
                date: "2026-01-01".into(),
                phases,
            }
        }

        #[test]
        fn plans_with_phases_does_not_panic() {
            let doc = make_doc(
                "plan.json",
                "Test Plan",
                "active",
                vec![
                    PlanPhase { name: "Setup".into(), done: 1, total: 2 },
                    PlanPhase { name: "Ship".into(), done: 0, total: 3 },
                ],
            );
            let docs: Vec<&Doc> = vec![&doc];
            print_table(DocKind::Plans, &docs, DetailsMode::None);
        }

        #[test]
        fn plans_with_details_all_does_not_panic() {
            let doc = make_doc(
                "plan.json",
                "Test Plan",
                "active",
                vec![
                    PlanPhase { name: "Setup".into(), done: 2, total: 2 },
                    PlanPhase { name: "Ship".into(), done: 1, total: 3 },
                ],
            );
            let docs: Vec<&Doc> = vec![&doc];
            print_table(DocKind::Plans, &docs, DetailsMode::All);
        }

        #[test]
        fn plans_with_details_incomplete_does_not_panic() {
            let doc = make_doc(
                "plan.json",
                "Test Plan",
                "active",
                vec![PlanPhase { name: "Setup".into(), done: 0, total: 2 }],
            );
            let docs: Vec<&Doc> = vec![&doc];
            print_table(DocKind::Plans, &docs, DetailsMode::Incomplete);
        }

        #[test]
        fn non_plan_without_phases_does_not_panic() {
            let doc = make_doc("design.md", "My Design", "accepted", Vec::new());
            let docs: Vec<&Doc> = vec![&doc];
            print_table(DocKind::Designs, &docs, DetailsMode::None);
        }

        #[test]
        fn non_plan_with_phases_does_not_panic() {
            let doc = make_doc(
                "adr.md",
                "My ADR",
                "proposed",
                vec![PlanPhase { name: "Review".into(), done: 0, total: 1 }],
            );
            let docs: Vec<&Doc> = vec![&doc];
            print_table(DocKind::Adrs, &docs, DetailsMode::All);
        }

        #[test]
        fn plans_no_phases_uses_basic_layout() {
            let doc = make_doc("plan.json", "Empty Plan", "proposal", Vec::new());
            let docs: Vec<&Doc> = vec![&doc];
            print_table(DocKind::Plans, &docs, DetailsMode::None);
        }

        #[test]
        fn empty_docs_does_not_panic() {
            let docs: Vec<&Doc> = vec![];
            print_table(DocKind::References, &docs, DetailsMode::None);
        }

        #[test]
        fn details_all_with_complete_phases() {
            let doc = make_doc(
                "plan.json",
                "Done Plan",
                "complete",
                vec![PlanPhase { name: "Only".into(), done: 5, total: 5 }],
            );
            let docs: Vec<&Doc> = vec![&doc];
            print_table(DocKind::Plans, &docs, DetailsMode::All);
        }

        #[test]
        fn details_all_with_zero_total_phase() {
            let doc = make_doc(
                "plan.json",
                "Unknown Plan",
                "active",
                vec![PlanPhase { name: "Empty".into(), done: 0, total: 0 }],
            );
            let docs: Vec<&Doc> = vec![&doc];
            print_table(DocKind::Plans, &docs, DetailsMode::All);
        }
    }

    // ── list_all (smoke) ───────────────────────────────────────────

    mod list_all_tests {
        use super::*;

        #[test]
        fn empty_root_text_does_not_panic() {
            let root = temp_dir("list-all-text");
            list_all(&root, false);
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn empty_root_json_does_not_panic() {
            let root = temp_dir("list-all-json");
            list_all(&root, true);
            fs::remove_dir_all(root).ok();
        }
    }

    // ── list_kind (integration) ────────────────────────────────────

    mod list_kind_tests {
        use super::*;

        #[test]
        fn empty_docs_returns_zero() {
            let root = temp_dir("list-kind-empty");
            assert_eq!(list_kind(&root, DocKind::Designs, &[]), 0);
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn with_docs_and_status_filter() {
            let root = temp_dir("list-kind-filter");
            write_file(
                &root,
                "_docs/designs/a.md",
                "---\ntitle: Alpha\nversion: 1.0.0\nstatus: draft\ndate: 2026-01-01\n---\nbody\n",
            );
            write_file(
                &root,
                "_docs/designs/b.md",
                "---\ntitle: Beta\nversion: 1.0.0\nstatus: accepted\ndate: 2026-01-01\n---\nbody\n",
            );
            // Only draft
            assert_eq!(list_kind(&root, DocKind::Designs, &["--status", "draft"]), 0);
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn with_json_output() {
            let root = temp_dir("list-kind-json");
            write_file(
                &root,
                "_docs/adrs/adr.md",
                "---\ntitle: My ADR\nversion: 1.0.0\nstatus: accepted\ndate: 2026-01-01\n---\nbody\n",
            );
            assert_eq!(list_kind(&root, DocKind::Adrs, &["--json"]), 0);
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn query_no_match_returns_one() {
            let root = temp_dir("list-kind-nomatch");
            write_file(
                &root,
                "_docs/designs/a.md",
                "---\ntitle: Alpha\nversion: 1.0.0\nstatus: draft\ndate: 2026-01-01\n---\nbody\n",
            );
            assert_eq!(list_kind(&root, DocKind::Designs, &["nonexistent"]), 1);
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn query_match_returns_zero() {
            let root = temp_dir("list-kind-match");
            write_file(
                &root,
                "_docs/designs/alpha.md",
                "---\ntitle: Alpha Design\nversion: 1.0.0\nstatus: draft\ndate: 2026-01-01\n---\nbody\n",
            );
            assert_eq!(list_kind(&root, DocKind::Designs, &["alpha"]), 0);
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn invalid_option_returns_one() {
            let root = temp_dir("list-kind-badopt");
            assert_eq!(list_kind(&root, DocKind::Designs, &["--badopt"]), 1);
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn status_filter_no_match_prints_message() {
            let root = temp_dir("list-kind-status-nomatch");
            write_file(
                &root,
                "_docs/designs/a.md",
                "---\ntitle: Alpha\nversion: 1.0.0\nstatus: draft\ndate: 2026-01-01\n---\nbody\n",
            );
            // Filter by "accepted" but only draft exists
            assert_eq!(list_kind(&root, DocKind::Designs, &["--status", "accepted"]), 0);
            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn limit_option_works() {
            let root = temp_dir("list-kind-limit");
            write_file(
                &root,
                "_docs/designs/a.md",
                "---\ntitle: Alpha\nversion: 1.0.0\nstatus: draft\ndate: 2026-01-01\n---\nbody\n",
            );
            write_file(
                &root,
                "_docs/designs/b.md",
                "---\ntitle: Beta\nversion: 1.0.0\nstatus: draft\ndate: 2026-02-01\n---\nbody\n",
            );
            assert_eq!(list_kind(&root, DocKind::Designs, &["--limit", "1"]), 0);
            fs::remove_dir_all(root).ok();
        }
    }
}
