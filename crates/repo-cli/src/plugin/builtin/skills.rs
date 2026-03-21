use crate::output::{bold, dim, green, red, yellow};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Embedded built-in assets ───────────────────────────────────────

struct DefaultAsset {
    filename: &'static str,
    content: &'static str,
}

const BUILTIN_SKILLS: &[DefaultAsset] = &[
    DefaultAsset {
        filename: "01KM17JDVNJ333TN3R5BGZB5QS-tsdoc-voice.md",
        content: include_str!("../../../defaults/skills/01KM17JDVNJ333TN3R5BGZB5QS-tsdoc-voice.md"),
    },
    DefaultAsset {
        filename: "01KM188YV2CJ26QH6KNH2NWG1Z-mental-model.md",
        content: include_str!(
            "../../../defaults/skills/01KM188YV2CJ26QH6KNH2NWG1Z-mental-model.md"
        ),
    },
    DefaultAsset {
        filename: "01KM18ZD23GC3TDVN7W0GX2000-adv-plan.md",
        content: include_str!("../../../defaults/skills/01KM18ZD23GC3TDVN7W0GX2000-adv-plan.md"),
    },
    DefaultAsset {
        filename: "01KM1A13V4FY0371Y0AB7FSGX9-purpose-md.md",
        content: include_str!("../../../defaults/skills/01KM1A13V4FY0371Y0AB7FSGX9-purpose-md.md"),
    },
    DefaultAsset {
        filename: "01KM1A156P4VEY0KT304QXA466-testing-standards.md",
        content: include_str!(
            "../../../defaults/skills/01KM1A156P4VEY0KT304QXA466-testing-standards.md"
        ),
    },
    DefaultAsset {
        filename: "01KM23VWVQWH62NBFF0TTFWVXR-doc-hygiene.md",
        content: include_str!(
            "../../../defaults/skills/01KM23VWVQWH62NBFF0TTFWVXR-doc-hygiene.md"
        ),
    },
];

const BUILTIN_REFERENCES: &[DefaultAsset] = &[
    DefaultAsset {
        filename: "01KM17JDVNJ333TN3R5BGZB5QS-tsdoc-spec.md",
        content: include_str!(
            "../../../defaults/references/01KM17JDVNJ333TN3R5BGZB5QS-tsdoc-spec.md"
        ),
    },
    DefaultAsset {
        filename: "01KM188YV2CJ26QH6KNH2NWG1Z-mental-model-schema.md",
        content: include_str!(
            "../../../defaults/references/01KM188YV2CJ26QH6KNH2NWG1Z-mental-model-schema.md"
        ),
    },
    DefaultAsset {
        filename: "01KM18ZD23GC3TDVN7W0GX2000-plan-schema-fields.md",
        content: include_str!(
            "../../../defaults/references/01KM18ZD23GC3TDVN7W0GX2000-plan-schema-fields.md"
        ),
    },
    DefaultAsset {
        filename: "01KM23VWVQWH62NBFF0TTFWVXR-detection-patterns.md",
        content: include_str!(
            "../../../defaults/references/01KM23VWVQWH62NBFF0TTFWVXR-detection-patterns.md"
        ),
    },
    DefaultAsset {
        filename: "01KM23VWVQWH62NBFF0TTFWVXR-sync-checks.md",
        content: include_str!(
            "../../../defaults/references/01KM23VWVQWH62NBFF0TTFWVXR-sync-checks.md"
        ),
    },
    DefaultAsset {
        filename: "01KM23VWVQWH62NBFF0TTFWVXR-report-template.md",
        content: include_str!(
            "../../../defaults/references/01KM23VWVQWH62NBFF0TTFWVXR-report-template.md"
        ),
    },
];

const BUILTIN_SCHEMAS: &[DefaultAsset] = &[DefaultAsset {
    filename: "01KM18ZD23GC3TDVN7W0GX2000-plan-schema.ts",
    content: include_str!("../../../defaults/schemas/01KM18ZD23GC3TDVN7W0GX2000-plan-schema.ts"),
}];

// All 10 default skills — superset of BUILTIN_SKILLS. Used by `repo skills deploy`
// to install directly into the agent skills ecosystem (.agents/skills/).
const ALL_DEFAULT_SKILLS: &[DefaultAsset] = &[
    DefaultAsset {
        filename: "01KM17JDVNJ333TN3R5BGZB5QS-tsdoc-voice.md",
        content: include_str!(
            "../../../defaults/skills/01KM17JDVNJ333TN3R5BGZB5QS-tsdoc-voice.md"
        ),
    },
    DefaultAsset {
        filename: "01KM188YV2CJ26QH6KNH2NWG1Z-mental-model.md",
        content: include_str!(
            "../../../defaults/skills/01KM188YV2CJ26QH6KNH2NWG1Z-mental-model.md"
        ),
    },
    DefaultAsset {
        filename: "01KM18ZD23GC3TDVN7W0GX2000-adv-plan.md",
        content: include_str!(
            "../../../defaults/skills/01KM18ZD23GC3TDVN7W0GX2000-adv-plan.md"
        ),
    },
    DefaultAsset {
        filename: "01KM1A13V4FY0371Y0AB7FSGX9-purpose-md.md",
        content: include_str!(
            "../../../defaults/skills/01KM1A13V4FY0371Y0AB7FSGX9-purpose-md.md"
        ),
    },
    DefaultAsset {
        filename: "01KM1A156P4VEY0KT304QXA466-testing-standards.md",
        content: include_str!(
            "../../../defaults/skills/01KM1A156P4VEY0KT304QXA466-testing-standards.md"
        ),
    },
    DefaultAsset {
        filename: "01KM1BKXK7ST4DT8P6YC1BTMRD-incremental-validation.md",
        content: include_str!(
            "../../../defaults/skills/01KM1BKXK7ST4DT8P6YC1BTMRD-incremental-validation.md"
        ),
    },
    DefaultAsset {
        filename: "01KM1BVKWT984AB0A4WPZRWWGX-review-plan.md",
        content: include_str!(
            "../../../defaults/skills/01KM1BVKWT984AB0A4WPZRWWGX-review-plan.md"
        ),
    },
    DefaultAsset {
        filename: "01KM1YWRFYBBT98WV14WXDKJM4-prompt-builder.md",
        content: include_str!(
            "../../../defaults/skills/01KM1YWRFYBBT98WV14WXDKJM4-prompt-builder.md"
        ),
    },
    DefaultAsset {
        filename: "01KM1Z6WK23PJQJ5PM9E9B07BC-behavioral-layer.md",
        content: include_str!(
            "../../../defaults/skills/01KM1Z6WK23PJQJ5PM9E9B07BC-behavioral-layer.md"
        ),
    },
    DefaultAsset {
        filename: "01KM23VWVQWH62NBFF0TTFWVXR-doc-hygiene.md",
        content: include_str!(
            "../../../defaults/skills/01KM23VWVQWH62NBFF0TTFWVXR-doc-hygiene.md"
        ),
    },
];

// ── Known agent configurations ───────────────────────────────────────────────

struct KnownAgent {
    /// Human-readable display name.
    name: &'static str,
    /// Config directory path relative to home (used for detection and symlink
    /// placement). E.g. `".claude"` → `~/.claude/skills/<name>`.
    config_dir: &'static str,
}

// Agents whose config directories are checked during `repo skills deploy`.
// Detection: agent is considered present if `~/<config_dir>` exists.
// Symlink: `<base>/<config_dir>/skills/<name>` → `../../.agents/skills/<name>`.
const KNOWN_AGENTS: &[KnownAgent] = &[
    KnownAgent { name: "Claude Code",  config_dir: ".claude"          },
    KnownAgent { name: "Codex",        config_dir: ".codex"           },
    KnownAgent { name: "Cursor",       config_dir: ".cursor"          },
    KnownAgent { name: "Amp",          config_dir: ".amp"             },
    KnownAgent { name: "Warp",         config_dir: ".warp"            },
    KnownAgent { name: "Cortex Code",  config_dir: ".cortex"          },
    KnownAgent { name: "Cline",        config_dir: ".cline"           },
    KnownAgent { name: "OpenCode",     config_dir: ".config/opencode" },
];

// ── Config model ────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SkillsConfig {
    #[serde(default)]
    pub skills: Vec<SkillEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    /// Skill name (must match the SKILL.md `name` field).
    pub name: String,

    /// Source repository or URL for installation.
    /// E.g. "obra/superpowers" or "https://github.com/vercel-labs/agent-skills"
    pub source: String,

    /// Specific skill within the source repo (for multi-skill repos).
    /// If omitted, installs all / matches by name.
    pub skill: Option<String>,

    /// Target agents. E.g. ["claude-code", "codex"]. Empty = all detected.
    #[serde(default)]
    pub agents: Vec<String>,

    /// Install scope: "project" (default) or "global".
    #[serde(default = "default_scope")]
    pub scope: String,

    /// Description (informational, not used for matching).
    pub description: Option<String>,
}

fn default_scope() -> String {
    "project".into()
}

impl SkillsConfig {
    pub fn load(repo_root: &Path) -> Option<Self> {
        let path = repo_root.join(".repo").join("skills.toml");
        let content = std::fs::read_to_string(&path).ok()?;
        match toml::from_str(&content) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {e}", path.display());
                None
            }
        }
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }
}

// ── Public entry point ──────────────────────────────────────────────

pub fn run(repo_root: &Path, args: &[&str]) {
    let subcommand = args.first().copied().filter(|a| !a.starts_with('-'));

    if args.iter().any(|a| *a == "--help" || *a == "-h") {
        print_help();
        return;
    }

    match subcommand {
        Some("init")   => cmd_init(repo_root),
        Some("export") => cmd_export(repo_root),
        Some("sync")   => cmd_sync(repo_root),
        Some("install") => cmd_install(repo_root),
        Some("fix")    => cmd_fix(repo_root),
        Some("deploy") => cmd_deploy(args),
        Some(other) => {
            eprintln!("Unknown skills subcommand: {other}");
            eprintln!("Run `repo skills --help` for usage.");
        }
        None => cmd_check(repo_root),
    }
}

fn print_help() {
    println!(
        "\
repo skills — Manage required agent skills

USAGE:
    repo skills [COMMAND]

COMMANDS:
    (none)      Check installed skills against .repo/skills.toml
    init        Create .repo/skills.toml and copy built-in skills, references, schemas
    export      Snapshot currently installed skills into .repo/skills.toml
    sync        Merge installed skills into .repo/skills.toml (keeps existing entries)
    install     Install missing skills declared in .repo/skills.toml
    fix         Remove unfixable entries (empty source, skill not found) from .repo/skills.toml
    deploy      Install all built-in skills into the agent skills ecosystem

OPTIONS:
    -h, --help  Print this help message

Skills are declared in .repo/skills.toml and installed via `npx skills add`.
The init command also copies built-in skills to .repo/skills/, references to
.repo/references/, and schemas to .repo/schemas/. Existing files are never
overwritten. The check command verifies each declared skill has a SKILL.md
in the project's .agents/skills/ directory.

Run `repo skills fix` to automatically remove entries that cannot be installed
(missing source field or skill not found at the declared source).

`repo skills deploy` writes all 10 built-in skills directly into ~/.agents/skills/
and creates agent-specific symlinks (e.g. ~/.claude/skills/) for every detected
agent. No external registry required — the skill content is embedded in the binary.

  --global, -g   Install to ~/.agents/skills/ (default and only scope)
  --force,  -f   Overwrite already-installed skills and existing symlinks"
    );
}

// ── init ────────────────────────────────────────────────────────────

fn cmd_init(repo_root: &Path) {
    let repo_dir = repo_root.join(".repo");

    // 1. Write skills.toml template.
    let toml_path = repo_dir.join("skills.toml");
    if toml_path.exists() {
        println!("  {} .repo/skills.toml already exists", dim("--"));
    } else {
        let template = include_str!("../../../defaults/skills.toml");
        if let Err(e) = std::fs::write(&toml_path, template) {
            eprintln!("Error writing {}: {e}", toml_path.display());
            std::process::exit(1);
        }
        println!("  {} wrote .repo/skills.toml", green("ok"));
    }

    // 2. Write built-in skills, references, and schemas.
    let groups: &[(&str, &[DefaultAsset])] = &[
        ("skills", BUILTIN_SKILLS),
        ("references", BUILTIN_REFERENCES),
        ("schemas", BUILTIN_SCHEMAS),
    ];

    for (dir_name, assets) in groups {
        let dir = repo_dir.join(dir_name);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("Failed to create {}: {e}", dir.display());
            std::process::exit(1);
        }

        let mut written = 0u32;
        let mut skipped = 0u32;

        for asset in *assets {
            let path = dir.join(asset.filename);
            if path.exists() {
                skipped += 1;
                continue;
            }
            if let Err(e) = std::fs::write(&path, asset.content) {
                eprintln!("  {} failed to write {}: {e}", red("!!"), path.display());
            } else {
                written += 1;
            }
        }

        if written > 0 {
            println!("  {} wrote {written} built-in {}", green("ok"), dir_name,);
        }
        if skipped > 0 {
            println!(
                "  {} {skipped} {} already existed (not overwritten)",
                dim("--"),
                dir_name,
            );
        }
    }

    println!();
    println!("  Edit .repo/skills.toml to declare required external skills.");
    println!("  Built-in skills, references, and schemas are ready to use.");
}

// ── export ──────────────────────────────────────────────────────────

fn cmd_export(repo_root: &Path) {
    let installed = scan_installed_skills(repo_root);

    if installed.is_empty() {
        println!("  No skills found in .agents/skills/");
        return;
    }

    let config = SkillsConfig { skills: installed };

    let path = repo_root.join(".repo").join("skills.toml");
    let content = format!(
        "# .repo/skills.toml — Exported from installed skills\n\
         # Generated by `repo skills export`\n\
         # Edit source fields as needed, then run `repo skills` to validate.\n\n\
         {}\n",
        config.to_toml()
    );

    if let Err(e) = std::fs::write(&path, &content) {
        eprintln!("Error writing {}: {e}", path.display());
        std::process::exit(1);
    }

    println!("  {} wrote .repo/skills.toml", green("ok"));
    println!();

    for skill in &config.skills {
        println!(
            "  {} {}  {}",
            green("ok"),
            skill.name,
            dim(skill.description.as_deref().unwrap_or("")),
        );
    }

    println!();
    println!("  {} skill(s) exported", config.skills.len());
}

// ── sync ────────────────────────────────────────────────────────

fn cmd_sync(repo_root: &Path) {
    let installed = scan_installed_skills(repo_root);
    let existing = SkillsConfig::load(repo_root);

    let mut merged: Vec<SkillEntry> = Vec::new();
    let mut added = 0u32;
    let mut kept = 0u32;
    let mut removed_names: Vec<String> = Vec::new();

    // Build a lookup of existing entries by name.
    let existing_by_name: std::collections::HashMap<String, SkillEntry> = existing
        .as_ref()
        .map(|cfg| {
            cfg.skills
                .iter()
                .map(|s| (s.name.clone(), s.clone()))
                .collect()
        })
        .unwrap_or_default();

    // For each installed skill, keep existing config or create new entry.
    for on_disk in &installed {
        if let Some(existing_entry) = existing_by_name.get(&on_disk.name) {
            // Preserve the user's config (source, agents, scope, etc.).
            // Update description from disk if the existing one is empty.
            let mut entry = existing_entry.clone();
            if entry.description.is_none() || entry.description.as_deref() == Some("") {
                entry.description = on_disk.description.clone();
            }
            merged.push(entry);
            kept += 1;
        } else {
            merged.push(on_disk.clone());
            added += 1;
        }
    }

    // Detect skills in config that are no longer on disk.
    if let Some(cfg) = &existing {
        let installed_names: Vec<&str> = installed.iter().map(|s| s.name.as_str()).collect();
        for entry in &cfg.skills {
            if !installed_names.contains(&entry.name.as_str()) {
                removed_names.push(entry.name.clone());
            }
        }
    }

    merged.sort_by(|a, b| a.name.cmp(&b.name));

    // Calculate column width for output.
    let w_name = merged
        .iter()
        .chain(removed_names.iter().map(|n| &existing_by_name[n]))
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0)
        .max(4);

    println!("{}", bold("Syncing skills"));
    println!();

    // Show what happened.
    for entry in &merged {
        let tag = if existing_by_name.contains_key(&entry.name) {
            dim("kept")
        } else {
            green("added")
        };
        println!(
            "  {} {:<w_name$}  {}",
            tag,
            entry.name,
            dim(entry.description.as_deref().unwrap_or("")),
        );
    }

    for name in &removed_names {
        println!(
            "  {} {:<w_name$}  {}",
            yellow("gone"),
            name,
            dim("no longer installed — removed from config"),
        );
    }

    // Write the merged config.
    let config = SkillsConfig { skills: merged };
    let path = repo_root.join(".repo").join("skills.toml");

    let header = if existing.is_some() {
        "# .repo/skills.toml — Synced with installed skills\n"
    } else {
        "# .repo/skills.toml — Created by repo skills sync\n"
    };
    let content = format!(
        "{header}\
         # Run `repo skills` to check, `repo skills install` to install missing.\n\
         # Fill in empty `source` fields so teammates can install too.\n\n\
         {}\n",
        config.to_toml()
    );

    if let Err(e) = std::fs::write(&path, &content) {
        eprintln!("Error writing {}: {e}", path.display());
        std::process::exit(1);
    }

    println!();
    println!(
        "  {} kept, {} added, {} removed",
        kept,
        green(&added.to_string()),
        if removed_names.is_empty() {
            "0".to_string()
        } else {
            yellow(&removed_names.len().to_string())
        },
    );
    println!("  {} wrote .repo/skills.toml", green("ok"));

    // Warn about empty sources.
    let empty_sources: Vec<&str> = config
        .skills
        .iter()
        .filter(|s| s.source.is_empty())
        .map(|s| s.name.as_str())
        .collect();
    if !empty_sources.is_empty() {
        println!();
        println!(
            "  {} {} skill(s) have empty source — fill in so teammates can install:",
            yellow("!!"),
            empty_sources.len(),
        );
        for name in &empty_sources {
            println!("    {}", dim(name));
        }
    }
}

// ── check ───────────────────────────────────────────────────────────

fn cmd_check(repo_root: &Path) {
    let config = match SkillsConfig::load(repo_root) {
        Some(cfg) => cfg,
        None => {
            println!("  {} no .repo/skills.toml found", dim("--"),);
            println!("  Run `repo skills init` to create one,");
            println!("  or `repo skills export` to snapshot installed skills.");
            return;
        }
    };

    if config.skills.is_empty() {
        println!("  No skills declared in .repo/skills.toml");
        return;
    }

    println!("{}", bold("Agent skills"));
    println!("  {}", dim("validating against .repo/skills.toml"));
    println!();

    let skills_dir = repo_root.join(".agents").join("skills");
    let mut pass = 0u32;
    let mut fail = 0u32;

    let w_name = config
        .skills
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0)
        .max(4);

    for entry in &config.skills {
        let skill_path = skills_dir.join(&entry.name);
        let skill_md = skill_path.join("SKILL.md");

        if skill_md.is_file() {
            println!(
                "  {} {:<w_name$}  {}",
                green("ok"),
                entry.name,
                dim(entry.description.as_deref().unwrap_or("")),
            );
            pass += 1;
        } else {
            println!(
                "  {} {:<w_name$}  {}",
                red("!!"),
                entry.name,
                red("not installed"),
            );

            // Show install command.
            match build_install_cmd(entry) {
                Some(install_cmd) => println!(
                    "    {:<w_name$}  {}",
                    "",
                    dim(&format!("install: {install_cmd}")),
                ),
                None => println!(
                    "    {:<w_name$}  {}",
                    "",
                    yellow("fill in 'source' in .repo/skills.toml to enable install"),
                ),
            }

            fail += 1;
        }
    }

    println!();
    println!(
        "  {} installed, {} missing",
        green(&pass.to_string()),
        if fail > 0 {
            red(&fail.to_string())
        } else {
            fail.to_string()
        },
    );

    if fail > 0 {
        println!();
        println!(
            "  Run {} to install missing skills.",
            dim("repo skills install"),
        );
        std::process::exit(1);
    }
}

// ── install ─────────────────────────────────────────────────────────

/// Outcome of a single install attempt.
#[derive(Debug)]
enum InstallOutcome {
    Installed,
    NoSource,
    NotFound { source: String },
    Failed { exit_status: String },
    Error { message: String },
}

fn run_install(entry: &SkillEntry) -> InstallOutcome {
    let cmd = match build_install_cmd(entry) {
        Some(c) => c,
        None => return InstallOutcome::NoSource,
    };

    let result = std::process::Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output();

    match result {
        Ok(out) if out.status.success() => InstallOutcome::Installed,
        Ok(out) => {
            // Combine stdout + stderr to detect "No matching skills found".
            let combined = format!(
                "{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr),
            );
            if combined.contains("No matching skills found") {
                InstallOutcome::NotFound {
                    source: entry.source.clone(),
                }
            } else {
                InstallOutcome::Failed {
                    exit_status: out.status.to_string(),
                }
            }
        }
        Err(e) => InstallOutcome::Error {
            message: e.to_string(),
        },
    }
}

fn cmd_install(repo_root: &Path) {
    let config = match SkillsConfig::load(repo_root) {
        Some(cfg) => cfg,
        None => {
            eprintln!("  No .repo/skills.toml found. Run `repo skills init` first.");
            std::process::exit(1);
        }
    };

    let skills_dir = repo_root.join(".agents").join("skills");

    let missing: Vec<&SkillEntry> = config
        .skills
        .iter()
        .filter(|s| !skills_dir.join(&s.name).join("SKILL.md").is_file())
        .collect();

    if missing.is_empty() {
        println!("  {} all skills are installed", green("ok"));
        return;
    }

    println!("{} {} missing skill(s)", bold("Installing"), missing.len());
    println!();

    let w_name = missing
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0)
        .max(4);

    let mut needs_fix: Vec<&str> = Vec::new();

    for entry in &missing {
        match run_install(entry) {
            InstallOutcome::Installed => {
                println!("  {} {}", green("ok"), entry.name);
            }
            InstallOutcome::NoSource => {
                println!(
                    "  {} {:<w_name$}  {}",
                    yellow("!!"),
                    entry.name,
                    yellow("no source — set 'source' in .repo/skills.toml or run: repo skills fix"),
                );
                needs_fix.push(&entry.name);
            }
            InstallOutcome::NotFound { ref source } => {
                println!(
                    "  {} {:<w_name$}  {}",
                    red("!!"),
                    entry.name,
                    red(&format!("skill not found in {source}")),
                );
                println!(
                    "    {:<w_name$}  {}",
                    "",
                    dim("fix: correct the 'source'/'skill' fields in .repo/skills.toml, or run: repo skills fix"),
                );
                needs_fix.push(&entry.name);
            }
            InstallOutcome::Failed { ref exit_status } => {
                eprintln!(
                    "  {} {:<w_name$}  {}",
                    red("!!"),
                    entry.name,
                    red(&format!("install failed ({exit_status})")),
                );
            }
            InstallOutcome::Error { ref message } => {
                eprintln!(
                    "  {} {:<w_name$}  {}",
                    red("!!"),
                    entry.name,
                    red(&format!("could not run install: {message}")),
                );
            }
        }
        println!();
    }

    if !needs_fix.is_empty() {
        println!(
            "  {} {} entry/entries cannot be installed as configured.",
            yellow("!!"),
            needs_fix.len(),
        );
        println!(
            "  Run {} to remove them from .repo/skills.toml.",
            bold("repo skills fix"),
        );
    }
}

// ── fix ─────────────────────────────────────────────────────────────

fn cmd_fix(repo_root: &Path) {
    let config = match SkillsConfig::load(repo_root) {
        Some(cfg) => cfg,
        None => {
            eprintln!("  No .repo/skills.toml found. Run `repo skills init` first.");
            std::process::exit(1);
        }
    };

    if config.skills.is_empty() {
        println!("  {} .repo/skills.toml is empty, nothing to fix", dim("--"));
        return;
    }

    println!("{}", bold("Checking skills for fixable issues"));
    println!();

    let w_name = config
        .skills
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0)
        .max(4);

    let mut keep: Vec<SkillEntry> = Vec::new();
    let mut removed: Vec<(String, &'static str)> = Vec::new();

    for entry in &config.skills {
        // Empty source — cannot install without user providing one.
        if entry.source.is_empty() {
            println!(
                "  {} {:<w_name$}  {}",
                yellow("rm"),
                entry.name,
                yellow("no source field — cannot install"),
            );
            removed.push((entry.name.clone(), "no source"));
            continue;
        }

        // Probe: run the install command and check if skill is not found.
        print!("  {} {:<w_name$}  checking... ", dim(".."), entry.name);
        // Flush stdout so the "checking..." appears before the (slow) npx call.
        use std::io::Write;
        let _ = std::io::stdout().flush();

        let outcome = run_install(entry);
        match outcome {
            InstallOutcome::Installed => {
                println!("{}", green("ok (now installed)"));
                keep.push(entry.clone());
            }
            InstallOutcome::NotFound { ref source } => {
                println!("{}", red(&format!("skill not found in {source}")));
                println!(
                    "    {:<w_name$}  {}",
                    "",
                    dim(&format!("removed — fix source/skill fields to re-add")),
                );
                removed.push((entry.name.clone(), "skill not found at source"));
            }
            InstallOutcome::NoSource => {
                // Already handled above; shouldn't reach here.
                println!("{}", yellow("no source"));
                removed.push((entry.name.clone(), "no source"));
            }
            InstallOutcome::Failed { ref exit_status } => {
                println!("{}", yellow(&format!("install failed ({exit_status}) — kept")));
                println!(
                    "    {:<w_name$}  {}",
                    "",
                    dim("kept — failure may be transient (network, auth). Re-run to retry."),
                );
                keep.push(entry.clone());
            }
            InstallOutcome::Error { ref message } => {
                println!("{}", yellow(&format!("could not run install: {message} — kept")));
                keep.push(entry.clone());
            }
        }
    }

    println!();

    if removed.is_empty() {
        println!("  {} nothing to fix — all entries look valid", green("ok"));
        return;
    }

    // Write the pruned config back.
    let pruned = SkillsConfig { skills: keep };
    let path = repo_root.join(".repo").join("skills.toml");
    let content = format!(
        "# .repo/skills.toml — Synced with installed skills\n\
         # Run `repo skills` to check, `repo skills install` to install missing.\n\
         # Fill in empty `source` fields so teammates can install too.\n\n\
         {}\n",
        pruned.to_toml()
    );

    if let Err(e) = std::fs::write(&path, &content) {
        eprintln!("  {} failed to write {}: {e}", red("!!"), path.display());
        std::process::exit(1);
    }

    println!(
        "  {} removed {} entry/entries from .repo/skills.toml:",
        green("ok"),
        removed.len(),
    );
    for (name, reason) in &removed {
        println!("    {} — {}", name, dim(reason));
    }
    println!();
    println!(
        "  Add corrected entries back manually, or re-run {} after editing.",
        dim("repo skills install"),
    );
}

// ── Helpers ─────────────────────────────────────────────────────────

fn build_install_cmd(entry: &SkillEntry) -> Option<String> {
    if entry.source.is_empty() {
        return None;
    }
    let mut cmd = format!("npx skills add {}", entry.source);

    // --skill flag.
    if let Some(ref skill) = entry.skill {
        cmd.push_str(&format!(" --skill {skill}"));
    }

    // --agent flags.
    for agent in &entry.agents {
        cmd.push_str(&format!(" -a {agent}"));
    }

    // --global flag.
    if entry.scope == "global" {
        cmd.push_str(" -g");
    }

    // Non-interactive.
    cmd.push_str(" -y");

    Some(cmd)
}

fn scan_installed_skills(repo_root: &Path) -> Vec<SkillEntry> {
    let skills_dir = repo_root.join(".agents").join("skills");
    let mut entries = Vec::new();

    let Ok(dir) = std::fs::read_dir(&skills_dir) else {
        return entries;
    };

    for entry in dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }

        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        // Parse SKILL.md frontmatter for description.
        let description = parse_skill_description(&skill_md);

        entries.push(SkillEntry {
            name,
            source: String::new(), // User must fill in the source.
            skill: None,
            agents: Vec::new(),
            scope: "project".into(),
            description,
        });
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

// ── deploy helpers ───────────────────────────────────────────────────────────

/// Returns the user's home directory from `$HOME` (Unix) or `%USERPROFILE%` (Windows).
fn get_home_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
}

/// Extracts the `name:` value from a skill's YAML frontmatter.
fn parse_skill_name(content: &str) -> Option<&str> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    for line in rest[..end].lines() {
        if let Some(val) = line.strip_prefix("name:") {
            return Some(val.trim());
        }
    }
    None
}

/// Builds the relative symlink target from `<base>/<config_dir>/skills/<name>`
/// back to `<base>/.agents/skills/<name>`.
///
/// Depth = number of path segments in `config_dir` + 1 (for the `skills` dir).
/// E.g. `.claude` (depth 2) → `../../.agents/skills/<name>`
///      `.config/opencode` (depth 3) → `../../../.agents/skills/<name>`
fn symlink_target(config_dir: &str, skill_name: &str) -> String {
    let depth = config_dir.split('/').count() + 1;
    format!("{}.agents/skills/{skill_name}", "../".repeat(depth))
}

// ── deploy ───────────────────────────────────────────────────────────────────

fn cmd_deploy(args: &[&str]) {
    let global = args.iter().any(|a| *a == "--global" || *a == "-g");
    let force  = args.iter().any(|a| *a == "--force"  || *a == "-f");

    let home = match get_home_dir() {
        Some(h) => h,
        None => {
            eprintln!("{} cannot determine home directory", red("!!"));
            std::process::exit(1);
        }
    };

    // For global scope the canonical base is ~/.agents/skills/.
    // For project scope we would use the repo root, but deploy is always global —
    // internal skills are user-level tools, not project-specific artefacts.
    // (Use --global / default behaviour is always global for this command.)
    let base = if global || true { home.clone() } else { home.clone() };
    let canonical_base = base.join(".agents").join("skills");

    // Detect which agents have their config directory present under $HOME.
    let detected: Vec<&KnownAgent> = KNOWN_AGENTS
        .iter()
        .filter(|a| home.join(a.config_dir).is_dir())
        .collect();

    println!("{}", bold("Deploying built-in skills"));
    println!();

    if detected.is_empty() {
        println!(
            "  {} no agent config dirs found under {}",
            yellow("!!"),
            dim(&home.display().to_string()),
        );
        println!("  Skills will be written to {} only.", dim("~/.agents/skills/"));
    } else {
        let names: Vec<&str> = detected.iter().map(|a| a.name).collect();
        println!("  {}  {}", dim("agents :"), names.join(", "));
    }
    println!(
        "  {}  ~/.agents/skills/",
        dim("install:"),
    );
    println!();

    let mut installed = 0u32;
    let mut skipped   = 0u32;
    let mut failed    = 0u32;

    let w = ALL_DEFAULT_SKILLS
        .iter()
        .filter_map(|a| parse_skill_name(a.content))
        .map(str::len)
        .max()
        .unwrap_or(0);

    for asset in ALL_DEFAULT_SKILLS {
        let skill_name = match parse_skill_name(asset.content) {
            Some(n) => n,
            None => {
                eprintln!("  {} could not parse name from {}", red("!!"), asset.filename);
                failed += 1;
                continue;
            }
        };

        let skill_dir = canonical_base.join(skill_name);
        let skill_md  = skill_dir.join("SKILL.md");

        // Skip already-installed unless --force.
        if skill_md.exists() && !force {
            println!("  {} {:<w$}  {}", dim("--"), skill_name, dim("already installed"));
            skipped += 1;
            continue;
        }

        // Write canonical SKILL.md.
        if let Err(e) = std::fs::create_dir_all(&skill_dir) {
            eprintln!("  {} {skill_name}: mkdir failed: {e}", red("!!"));
            failed += 1;
            continue;
        }
        if let Err(e) = std::fs::write(&skill_md, asset.content) {
            eprintln!("  {} {skill_name}: write failed: {e}", red("!!"));
            failed += 1;
            continue;
        }

        // Create per-agent symlinks.
        let mut linked:  Vec<&str> = Vec::new();
        let mut skipped_link: Vec<&str> = Vec::new();

        for agent in &detected {
            let agent_skills_dir = home.join(agent.config_dir).join("skills");

            if let Err(e) = std::fs::create_dir_all(&agent_skills_dir) {
                eprintln!(
                    "  {} {skill_name}: mkdir {}: {e}",
                    yellow("!!"),
                    agent_skills_dir.display(),
                );
                continue;
            }

            let link_path = agent_skills_dir.join(skill_name);
            let target    = symlink_target(agent.config_dir, skill_name);

            // Remove stale link/file when --force, otherwise skip.
            let exists = link_path.exists() || link_path.symlink_metadata().is_ok();
            if exists {
                if force {
                    let _ = std::fs::remove_file(&link_path);
                } else {
                    skipped_link.push(agent.name);
                    continue;
                }
            }

            #[cfg(unix)]
            let result = std::os::unix::fs::symlink(&target, &link_path);
            #[cfg(not(unix))]
            let result = std::fs::copy(&skill_md, link_path.join("SKILL.md")).map(|_| ());

            match result {
                Ok(()) => linked.push(agent.name),
                Err(e) => eprintln!(
                    "  {} {skill_name}: symlink for {}: {e}",
                    yellow("!!"),
                    agent.name,
                ),
            }
        }

        let link_info = if linked.is_empty() && skipped_link.is_empty() {
            dim("(no agents)")
        } else if linked.is_empty() {
            dim("(links already exist)")
        } else {
            dim(&linked.join(", "))
        };

        println!("  {} {:<w$}  {link_info}", green("ok"), skill_name);
        installed += 1;
    }

    println!();
    print!(
        "  {} installed, {} skipped",
        green(&installed.to_string()),
        skipped,
    );
    if failed > 0 {
        print!(", {} {}", red(&failed.to_string()), red("failed"));
    }
    println!();

    if skipped > 0 {
        println!(
            "  Run {} to overwrite existing installs.",
            dim("`repo skills deploy --force`"),
        );
    }
}

fn parse_skill_description(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let content = content.trim_start();

    if !content.starts_with("---") {
        return None;
    }

    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];

    for line in frontmatter.lines() {
        if let Some(rest) = line.strip_prefix("description:") {
            let val = rest.trim();
            let val = val
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(val);
            // Truncate long descriptions.
            if val.len() > 80 {
                return Some(format!("{}...", &val[..77]));
            }
            return Some(val.to_string());
        }
    }

    None
}
