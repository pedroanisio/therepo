use crate::output::{bold, cyan, dim, green};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ── Prompt model ────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
    pub builtin: bool,
}

#[derive(Debug, Serialize)]
struct PromptInitReport {
    written: usize,
    skipped: usize,
}

// ── Embedded defaults ───────────────────────────────────────────────

struct DefaultPrompt {
    filename: &'static str,
    content: &'static str,
}

const DEFAULTS: &[DefaultPrompt] = &[
    DefaultPrompt {
        filename: "assess-corpus.md",
        content: include_str!("../../../defaults/prompts/assess-corpus.md"),
    },
    DefaultPrompt {
        filename: "feedback-processor.md",
        content: include_str!("../../../defaults/prompts/feedback-processor.md"),
    },
    DefaultPrompt {
        filename: "format-plan.md",
        content: include_str!("../../../defaults/prompts/format-plan.md"),
    },
    DefaultPrompt {
        filename: "review-cycle.md",
        content: include_str!("../../../defaults/prompts/review-cycle.md"),
    },
    DefaultPrompt {
        filename: "review-internal.md",
        content: include_str!("../../../defaults/prompts/review-internal.md"),
    },
    DefaultPrompt {
        filename: "validate-plan.md",
        content: include_str!("../../../defaults/prompts/validate-plan.md"),
    },
    DefaultPrompt {
        filename: "audit-repo.md",
        content: include_str!("../../../defaults/prompts/audit-repo.md"),
    },
];

fn load_defaults() -> Vec<Prompt> {
    DEFAULTS
        .iter()
        .map(|d| {
            let path = Path::new(d.filename);
            let mut p = parse_prompt(d.content, path);
            p.builtin = true;
            p
        })
        .collect()
}

// ── Commands ────────────────────────────────────────────────────────

#[must_use]
pub fn run(repo_root: &Path, args: &[&str]) -> i32 {
    let subcommand = args.first().copied().filter(|a| !a.starts_with('-'));
    let json = args.contains(&"--json");

    if args.iter().any(|a| *a == "--help" || *a == "-h") {
        print_help();
        return 0;
    }

    let prompts_dir = repo_root.join(".repo").join("prompts");

    match subcommand {
        Some("init") => cmd_init(&prompts_dir, json),
        Some("list") | None => cmd_list(&prompts_dir, args, json),
        Some(name) => cmd_show(&prompts_dir, name),
    }
}

fn print_help() {
    println!(
        "\
repo prompt — Reusable prompt snippets for AI agents and workflows

USAGE:
    repo prompt [COMMAND] [OPTIONS]

COMMANDS:
    <name>      Output the named prompt snippet to stdout
    list        List all available prompts (default when no args)
    init        Write built-in defaults to .repo/prompts/ for customization

OPTIONS:
    --tag <TAG>  Filter by tag (e.g. plan, format, review)
    --json       Emit machine-readable JSON for `list`
    -h, --help   Print this help message

Built-in prompts are always available. User prompts in .repo/prompts/
override built-ins by name. Run `repo prompt init` to copy defaults to
disk for customization.

SNIPPET FORMAT:
    Prompt snippets are markdown files with optional YAML frontmatter:

        ---
        name: my-prompt
        description: What this prompt does
        tags: [review, format]
        ---

        <prompt body here>

    Files without frontmatter use the filename (minus .md) as the name."
    );
}

// ── init ────────────────────────────────────────────────────────────

fn cmd_init(prompts_dir: &Path, json: bool) -> i32 {
    if let Err(e) = fs::create_dir_all(prompts_dir) {
        eprintln!("Failed to create {}: {e}", prompts_dir.display());
        return 1;
    }

    let mut written = 0;
    let mut skipped = 0;
    let mut failed = false;

    for d in DEFAULTS {
        let path = prompts_dir.join(d.filename);
        if path.exists() {
            skipped += 1;
            continue;
        }
        if let Err(e) = fs::write(&path, d.content) {
            eprintln!("Failed to write {}: {e}", path.display());
            failed = true;
        } else {
            written += 1;
        }
    }

    if json {
        let report = PromptInitReport { written, skipped };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
        );
        return i32::from(failed);
    }

    if written > 0 {
        println!(
            "{} Wrote {written} default prompt(s) to .repo/prompts/",
            green("✓")
        );
    }
    if skipped > 0 {
        println!("  {} {skipped} already existed (not overwritten)", dim("↳"));
    }
    if written == 0 && skipped > 0 {
        println!("  All defaults already present. Edit them in .repo/prompts/");
    }

    i32::from(failed)
}

// ── list ────────────────────────────────────────────────────────────

fn cmd_list(prompts_dir: &Path, args: &[&str], json: bool) -> i32 {
    let all_prompts = load_merged(prompts_dir);
    if all_prompts.is_empty() {
        if json {
            println!("[]");
            return 0;
        }
        println!("No prompts available.");
        return 0;
    }

    let filter = tag_filter(args);
    let filtered = list_prompts(prompts_dir, filter.as_deref());

    if filtered.is_empty() {
        if json {
            println!("[]");
            return 0;
        }
        println!("No prompts match the given filter.");
        return 0;
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&filtered).unwrap_or_else(|_| "[]".to_string())
        );
        return 0;
    }

    let refs = filtered.iter().collect::<Vec<_>>();
    print_table(&refs);
    0
}

// ── show ────────────────────────────────────────────────────────────

fn cmd_show(prompts_dir: &Path, name: &str) -> i32 {
    let prompts = load_merged(prompts_dir);

    // Match by name (exact), then by prefix.
    let found = prompts
        .iter()
        .find(|p| p.name == name)
        .or_else(|| prompts.iter().find(|p| p.name.starts_with(name)));

    if let Some(prompt) = found {
        println!("{}", prompt.body);
        0
    } else {
        eprintln!("Unknown prompt: {name}");
        eprintln!();

        let suggestions: Vec<&str> = prompts
            .iter()
            .filter(|p| p.name.contains(name) || p.tags.iter().any(|t| t.contains(name)))
            .map(|p| p.name.as_str())
            .collect();

        if suggestions.is_empty() {
            eprintln!("Run `repo prompt list` to see available prompts.");
        } else {
            eprintln!("Did you mean:");
            for s in &suggestions {
                eprintln!("  {s}");
            }
        }
        1
    }
}

// ── Merge: defaults + user overrides ────────────────────────────────

fn load_merged(prompts_dir: &Path) -> Vec<Prompt> {
    let mut defaults = load_defaults();

    // Load user prompts from disk (if directory exists).
    let user_prompts = if prompts_dir.is_dir() {
        scan_prompts(prompts_dir).unwrap_or_default()
    } else {
        Vec::new()
    };

    // User prompts override defaults by name.
    for user in user_prompts {
        if let Some(pos) = defaults.iter().position(|d| d.name == user.name) {
            defaults[pos] = user;
        } else {
            defaults.push(user);
        }
    }

    defaults.sort_by(|a, b| a.name.cmp(&b.name));
    defaults
}

#[must_use]
pub fn list_prompts(prompts_dir: &Path, tag: Option<&str>) -> Vec<Prompt> {
    let prompts = load_merged(prompts_dir);

    let Some(tag) = tag.map(str::to_lowercase) else {
        return prompts;
    };

    prompts
        .into_iter()
        .filter(|prompt| prompt.tags.iter().any(|value| value.to_lowercase() == tag))
        .collect()
}

fn tag_filter(args: &[&str]) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == "--tag")
        .map(|w| w[1].to_lowercase())
}

// ── Scanning & parsing ──────────────────────────────────────────────

fn scan_prompts(dir: &Path) -> Result<Vec<Prompt>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;

    let mut prompts = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        prompts.push(parse_prompt(&content, &path));
    }

    prompts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(prompts)
}

fn parse_prompt(content: &str, path: &Path) -> Prompt {
    let filename = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Prompt {
            name: filename,
            description: String::new(),
            tags: Vec::new(),
            body: content.to_string(),
            builtin: false,
        };
    }

    let rest = &trimmed[3..];
    let Some(end) = rest.find("\n---") else {
        return Prompt {
            name: filename,
            description: String::new(),
            tags: Vec::new(),
            body: content.to_string(),
            builtin: false,
        };
    };

    let frontmatter = &rest[..end];
    let body = rest[end + 4..].trim_start().to_string();
    let fields = parse_yaml_fields(frontmatter);

    let name = fields.get("name").cloned().unwrap_or(filename);
    let description = fields.get("description").cloned().unwrap_or_default();
    let tags = fields
        .get("tags")
        .map(|t| parse_tags(t))
        .unwrap_or_default();

    Prompt {
        name,
        description,
        tags,
        body,
        builtin: false,
    }
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

fn parse_tags(value: &str) -> Vec<String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .unwrap_or(value);

    inner
        .split(',')
        .map(|t| t.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

// ── Table rendering ─────────────────────────────────────────────────

fn print_table(prompts: &[&Prompt]) {
    let hdr_name = "NAME";
    let hdr_tags = "TAGS";
    let hdr_desc = "DESCRIPTION";
    let hdr_src = "SRC";

    let w_name = prompts
        .iter()
        .map(|p| p.name.len())
        .max()
        .unwrap_or(0)
        .max(hdr_name.len());
    let w_tags = prompts
        .iter()
        .map(|p| p.tags.join(", ").len())
        .max()
        .unwrap_or(0)
        .max(hdr_tags.len());
    let w_src = 8_usize.max(hdr_src.len());
    let w_desc = prompts
        .iter()
        .map(|p| p.description.len())
        .max()
        .unwrap_or(0)
        .max(hdr_desc.len());

    println!(
        "  {:<w_name$}  {:<w_tags$}  {:<w_src$}  {:<w_desc$}",
        bold(hdr_name),
        bold(hdr_tags),
        bold(hdr_src),
        bold(hdr_desc),
    );

    println!(
        "  {}  {}  {}  {}",
        dim(&"\u{2500}".repeat(w_name)),
        dim(&"\u{2500}".repeat(w_tags)),
        dim(&"\u{2500}".repeat(w_src)),
        dim(&"\u{2500}".repeat(w_desc)),
    );

    for p in prompts {
        let tags_display = if p.tags.is_empty() {
            dim("\u{2014}")
        } else {
            cyan(&p.tags.join(", "))
        };
        let tags_len = if p.tags.is_empty() {
            1
        } else {
            p.tags.join(", ").len()
        };
        let tags_padding = w_tags.saturating_sub(tags_len);

        let src = if p.builtin {
            dim("built-in")
        } else {
            "custom".to_string()
        };
        let src_len = if p.builtin { 8 } else { 6 };
        let src_padding = w_src.saturating_sub(src_len);

        println!(
            "  {:<w_name$}  {}{:>tpad$}  {}{:>spad$}  {}",
            p.name,
            tags_display,
            "",
            src,
            "",
            p.description,
            tpad = tags_padding,
            spad = src_padding,
        );
    }

    println!();
    let builtin_count = prompts.iter().filter(|p| p.builtin).count();
    let custom_count = prompts.len() - builtin_count;
    let mut parts = Vec::new();
    if builtin_count > 0 {
        parts.push(format!("{builtin_count} built-in"));
    }
    if custom_count > 0 {
        parts.push(format!("{custom_count} custom"));
    }
    println!("  {} prompt(s)  {}", prompts.len(), dim(&parts.join(", ")));
    println!("  Run {} to output a snippet.", dim("repo prompt <name>"));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression tests that catch documentation drift when built-in
    /// prompts are added or removed.
    mod asset_count_drift {
        use super::*;

        /// The prompt count stated in AGENTS.md must match the actual
        /// number of built-in prompts embedded in this module.
        /// If this test fails, update the stale count in AGENTS.md.
        #[test]
        fn prompt_count_matches_agents_md() {
            let expected = DEFAULTS.len();
            let agents_md = include_str!("../../../../../AGENTS.md");
            let needle = format!("{expected} built-in prompts");
            assert!(
                agents_md.contains(&needle),
                "AGENTS.md does not contain \"{needle}\". \
                 Update the prompt count in AGENTS.md to match the \
                 {expected} prompts defined in DEFAULTS."
            );
        }
    }

    mod parse_prompt_tests {
        use super::*;
        use std::path::Path;

        #[test]
        fn all_defaults_parse_successfully() {
            for d in DEFAULTS {
                let p = parse_prompt(d.content, Path::new(d.filename));
                assert!(
                    !p.name.is_empty(),
                    "prompt {:?} parsed with empty name",
                    d.filename
                );
            }
        }
    }

    // ── Helper ─────────────────────────────────────────────────────

    fn temp_dir(label: &str) -> std::path::PathBuf {
        use std::time::SystemTime;
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        let unique = std::process::id();
        let dir = std::env::temp_dir().join(format!("therepo-prompt-{label}-{nanos}-{unique}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    // ── load_defaults ──────────────────────────────────────────────

    #[test]
    fn load_defaults_returns_all_builtins() {
        let defaults = load_defaults();
        assert_eq!(defaults.len(), DEFAULTS.len());
        for p in &defaults {
            assert!(p.builtin, "all defaults must be builtin");
            assert!(!p.name.is_empty());
        }
    }

    // ── tag_filter ─────────────────────────────────────────────────

    #[test]
    fn tag_filter_extracts_value() {
        let args: Vec<&str> = vec!["list", "--tag", "review"];
        assert_eq!(tag_filter(&args), Some("review".to_string()));
    }

    #[test]
    fn tag_filter_returns_none_when_absent() {
        let args: Vec<&str> = vec!["list", "--json"];
        assert_eq!(tag_filter(&args), None);
    }

    #[test]
    fn tag_filter_lowercases_value() {
        let args: Vec<&str> = vec!["--tag", "PLAN"];
        assert_eq!(tag_filter(&args), Some("plan".to_string()));
    }

    // ── print_help ─────────────────────────────────────────────────

    #[test]
    fn print_help_does_not_panic() {
        // Just verify it doesn't panic; output goes to stdout.
        print_help();
    }

    // ── print_table ────────────────────────────────────────────────

    #[test]
    fn print_table_does_not_panic_on_empty() {
        let prompts: Vec<&Prompt> = vec![];
        print_table(&prompts);
    }

    #[test]
    fn print_table_does_not_panic_with_entries() {
        let p1 = Prompt {
            name: "test-prompt".to_string(),
            description: "A test".to_string(),
            tags: vec!["review".to_string()],
            body: "body".to_string(),
            builtin: true,
        };
        let p2 = Prompt {
            name: "custom-one".to_string(),
            description: String::new(),
            tags: vec![],
            body: "body2".to_string(),
            builtin: false,
        };
        print_table(&[&p1, &p2]);
    }

    // ── scan_prompts ───────────────────────────────────────────────

    #[test]
    fn scan_prompts_reads_md_files() {
        let dir = temp_dir("scan-md");
        fs::write(
            dir.join("my-prompt.md"),
            "---\nname: my-prompt\ndescription: hello\ntags: [a, b]\n---\nbody here",
        )
        .unwrap();
        // Non-md file should be ignored
        fs::write(dir.join("ignored.txt"), "not a prompt").unwrap();

        let result = scan_prompts(&dir).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-prompt");
        assert_eq!(result[0].tags, vec!["a", "b"]);
    }

    #[test]
    fn scan_prompts_nonexistent_dir() {
        let dir = std::env::temp_dir().join("therepo-prompt-does-not-exist-99999");
        let result = scan_prompts(&dir);
        assert!(result.is_err());
    }

    // ── load_merged ────────────────────────────────────────────────

    #[test]
    fn load_merged_includes_defaults_when_no_user_dir() {
        let dir = std::env::temp_dir().join("therepo-prompt-no-dir-merge");
        // Dir does not exist, so only defaults
        let prompts = load_merged(&dir);
        assert_eq!(prompts.len(), DEFAULTS.len());
    }

    #[test]
    fn load_merged_user_override_replaces_builtin() {
        let dir = temp_dir("merge-override");
        // Pick a known default name
        let defaults = load_defaults();
        let known_name = &defaults[0].name;
        let content = format!(
            "---\nname: {known_name}\ndescription: user version\ntags: [custom]\n---\nuser body"
        );
        // Filename must end in .md for scan
        fs::write(dir.join(format!("{known_name}.md")), &content).unwrap();

        let merged = load_merged(&dir);
        let found = merged.iter().find(|p| &p.name == known_name).unwrap();
        assert_eq!(found.description, "user version");
        assert!(!found.builtin, "user override should not be builtin");
    }

    #[test]
    fn load_merged_user_adds_new_prompt() {
        let dir = temp_dir("merge-add");
        fs::write(
            dir.join("brand-new.md"),
            "---\nname: brand-new\ndescription: new\n---\nnew body",
        )
        .unwrap();

        let merged = load_merged(&dir);
        assert!(merged.len() > DEFAULTS.len());
        assert!(merged.iter().any(|p| p.name == "brand-new"));
    }

    // ── cmd_init ───────────────────────────────────────────────────

    #[test]
    fn cmd_init_writes_defaults_text_mode() {
        let dir = temp_dir("init-text");
        let prompts_dir = dir.join("prompts");
        let code = cmd_init(&prompts_dir, false);
        assert_eq!(code, 0);

        // All default files should exist
        for d in DEFAULTS {
            assert!(
                prompts_dir.join(d.filename).exists(),
                "missing {}",
                d.filename
            );
        }
    }

    #[test]
    fn cmd_init_skips_existing_files() {
        let dir = temp_dir("init-skip");
        let prompts_dir = dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();

        // Pre-create one file
        fs::write(prompts_dir.join(DEFAULTS[0].filename), "custom content").unwrap();

        let code = cmd_init(&prompts_dir, false);
        assert_eq!(code, 0);

        // The pre-existing file should NOT be overwritten
        let content = fs::read_to_string(prompts_dir.join(DEFAULTS[0].filename)).unwrap();
        assert_eq!(content, "custom content");
    }

    #[test]
    fn cmd_init_json_mode() {
        let dir = temp_dir("init-json");
        let prompts_dir = dir.join("prompts");
        let code = cmd_init(&prompts_dir, true);
        assert_eq!(code, 0);
    }

    // ── cmd_list ───────────────────────────────────────────────────

    #[test]
    fn cmd_list_json_returns_zero() {
        let dir = temp_dir("list-json");
        let code = cmd_list(&dir, &["list", "--json"], true);
        assert_eq!(code, 0);
    }

    #[test]
    fn cmd_list_text_returns_zero() {
        let dir = temp_dir("list-text");
        let code = cmd_list(&dir, &["list"], false);
        assert_eq!(code, 0);
    }

    #[test]
    fn cmd_list_with_tag_filter_json() {
        let dir = temp_dir("list-tag-json");
        // Use a tag that won't match anything
        let code = cmd_list(&dir, &["list", "--tag", "nonexistent-xyz", "--json"], true);
        assert_eq!(code, 0);
    }

    #[test]
    fn cmd_list_with_tag_filter_text() {
        let dir = temp_dir("list-tag-text");
        let code = cmd_list(&dir, &["list", "--tag", "nonexistent-xyz"], false);
        assert_eq!(code, 0);
    }

    // ── cmd_show ───────────────────────────────────────────────────

    #[test]
    fn cmd_show_known_prompt() {
        let dir = temp_dir("show-known");
        let defaults = load_defaults();
        let code = cmd_show(&dir, &defaults[0].name);
        assert_eq!(code, 0);
    }

    #[test]
    fn cmd_show_unknown_prompt() {
        let dir = temp_dir("show-unknown");
        let code = cmd_show(&dir, "does-not-exist-xyz");
        assert_eq!(code, 1);
    }

    #[test]
    fn cmd_show_prefix_match() {
        let dir = temp_dir("show-prefix");
        let defaults = load_defaults();
        // Use first 3 chars of a known prompt name as prefix
        let prefix = &defaults[0].name[..3.min(defaults[0].name.len())];
        let code = cmd_show(&dir, prefix);
        // Should find via prefix match
        assert_eq!(code, 0);
    }

    // ── run() dispatch ─────────────────────────────────────────────

    #[test]
    fn run_help_flag() {
        let dir = temp_dir("run-help");
        assert_eq!(run(&dir, &["--help"]), 0);
    }

    #[test]
    fn run_short_help_flag() {
        let dir = temp_dir("run-help-h");
        assert_eq!(run(&dir, &["-h"]), 0);
    }

    #[test]
    fn run_no_args_defaults_to_list() {
        let dir = temp_dir("run-noargs");
        let code = run(&dir, &[]);
        assert_eq!(code, 0);
    }

    #[test]
    fn run_list_subcommand() {
        let dir = temp_dir("run-list");
        assert_eq!(run(&dir, &["list"]), 0);
    }

    #[test]
    fn run_init_subcommand() {
        let dir = temp_dir("run-init");
        assert_eq!(run(&dir, &["init"]), 0);
    }

    #[test]
    fn run_unknown_name_returns_one() {
        let dir = temp_dir("run-unknown");
        assert_eq!(run(&dir, &["totally-nonexistent-prompt-xyz"]), 1);
    }

    #[test]
    fn run_known_prompt_by_name() {
        let dir = temp_dir("run-known");
        let defaults = load_defaults();
        assert_eq!(run(&dir, &[defaults[0].name.as_str()]), 0);
    }

    #[test]
    fn run_list_json() {
        let dir = temp_dir("run-list-json");
        assert_eq!(run(&dir, &["list", "--json"]), 0);
    }

    // ── parse_yaml_fields ──────────────────────────────────────────

    #[test]
    fn parse_yaml_fields_handles_quoted_values() {
        let yaml = "name: \"hello\"\ndesc: 'world'";
        let fields = parse_yaml_fields(yaml);
        assert_eq!(fields.get("name").unwrap(), "hello");
        assert_eq!(fields.get("desc").unwrap(), "world");
    }

    #[test]
    fn parse_yaml_fields_skips_multiline_markers() {
        let yaml = "description: >\nname: test";
        let fields = parse_yaml_fields(yaml);
        assert!(!fields.contains_key("description"));
        assert_eq!(fields.get("name").unwrap(), "test");
    }

    #[test]
    fn parse_yaml_fields_skips_indented_lines() {
        let yaml = "name: top\n  indented: skip\n\t tabbed: skip";
        let fields = parse_yaml_fields(yaml);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields.get("name").unwrap(), "top");
    }

    // ── list_prompts ───────────────────────────────────────────────

    #[test]
    fn list_prompts_no_tag_returns_all() {
        let dir = temp_dir("list-notag");
        let result = list_prompts(&dir, None);
        assert_eq!(result.len(), DEFAULTS.len());
    }

    #[test]
    fn list_prompts_with_nonexistent_tag() {
        let dir = temp_dir("list-badtag");
        let result = list_prompts(&dir, Some("nonexistent-xyz-tag"));
        assert!(result.is_empty());
    }
}
