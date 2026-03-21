use crate::output::{bold, cyan, dim, green};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ── Prompt model ────────────────────────────────────────────────────

pub struct Prompt {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
    pub builtin: bool,
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

pub fn run(repo_root: &Path, args: &[&str]) {
    let subcommand = args.first().copied().filter(|a| !a.starts_with('-'));

    if args.iter().any(|a| *a == "--help" || *a == "-h") {
        print_help();
        return;
    }

    let prompts_dir = repo_root.join(".repo").join("prompts");

    match subcommand {
        Some("init") => cmd_init(&prompts_dir),
        Some("list") => cmd_list(&prompts_dir, args),
        Some(name) => cmd_show(&prompts_dir, name),
        None => cmd_list(&prompts_dir, args),
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

fn cmd_init(prompts_dir: &Path) {
    if let Err(e) = fs::create_dir_all(prompts_dir) {
        eprintln!("Failed to create {}: {e}", prompts_dir.display());
        std::process::exit(1);
    }

    let mut written = 0;
    let mut skipped = 0;

    for d in DEFAULTS {
        let path = prompts_dir.join(d.filename);
        if path.exists() {
            skipped += 1;
            continue;
        }
        if let Err(e) = fs::write(&path, d.content) {
            eprintln!("Failed to write {}: {e}", path.display());
        } else {
            written += 1;
        }
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
}

// ── list ────────────────────────────────────────────────────────────

fn cmd_list(prompts_dir: &Path, args: &[&str]) {
    let prompts = load_merged(prompts_dir);

    if prompts.is_empty() {
        println!("No prompts available.");
        return;
    }

    // Optional --tag filter.
    let tag_filter = args
        .windows(2)
        .find(|w| w[0] == "--tag")
        .map(|w| w[1].to_lowercase());

    let filtered: Vec<&Prompt> = prompts
        .iter()
        .filter(|p| {
            if let Some(ref tag) = tag_filter {
                p.tags.iter().any(|t| t.to_lowercase() == *tag)
            } else {
                true
            }
        })
        .collect();

    if filtered.is_empty() {
        println!("No prompts match the given filter.");
        return;
    }

    print_table(&filtered);
}

// ── show ────────────────────────────────────────────────────────────

fn cmd_show(prompts_dir: &Path, name: &str) {
    let prompts = load_merged(prompts_dir);

    // Match by name (exact), then by prefix.
    let found = prompts
        .iter()
        .find(|p| p.name == name)
        .or_else(|| prompts.iter().find(|p| p.name.starts_with(name)));

    match found {
        Some(prompt) => {
            println!("{}", prompt.body);
        }
        None => {
            eprintln!("Unknown prompt: {name}");
            eprintln!();

            let suggestions: Vec<&str> = prompts
                .iter()
                .filter(|p| p.name.contains(name) || p.tags.iter().any(|t| t.contains(name)))
                .map(|p| p.name.as_str())
                .collect();

            if !suggestions.is_empty() {
                eprintln!("Did you mean:");
                for s in &suggestions {
                    eprintln!("  {s}");
                }
            } else {
                eprintln!("Run `repo prompt list` to see available prompts.");
            }
            std::process::exit(1);
        }
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

            map.insert(key.to_string(), value.to_string());
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
