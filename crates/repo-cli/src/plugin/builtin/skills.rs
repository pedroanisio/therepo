use crate::output::{bold, dim, green, red, yellow};
use crate::progress::Spinner;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
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

// ── Skill bundle types ───────────────────────────────────────────────────────

/// A single file to be written into a skill's subdirectory on deploy.
struct BundledFile {
    /// Target filename within the subdirectory (e.g. `"tsdoc-spec.md"`).
    filename: &'static str,
    content: &'static str,
}

/// Source of a skill's content: either plain text or a `.skill` ZIP archive.
///
/// **Plain** — the classic `SKILL.md`/`.skill` text format:
/// a YAML-frontmatter header followed by Markdown instructions.
///
/// **Zip** — the Anthropic distribution format used by claude.ai and the API.
/// The ZIP contains `SKILL.md` at the root plus optional `references/`,
/// `scripts/`, and `assets/` subdirectories. `cmd_deploy` extracts the archive
/// directly into `~/.agents/skills/<name>/` at install time.
enum SkillSource {
    /// Plain text (`.md` or plain `.skill`). Written directly as `SKILL.md`.
    Plain(DefaultAsset),
    /// ZIP archive (`.skill` with `PK` magic). Extracted in-place on deploy.
    Zip {
        /// Source filename — used only for error messages.
        filename: &'static str,
        /// Raw ZIP bytes embedded with `include_bytes!`.
        bytes: &'static [u8],
    },
}

/// A complete deployable skill.
///
/// For `SkillSource::Plain` sources, `references`/`scripts`/`examples` are
/// written from the embedded text arrays. For `SkillSource::Zip` sources,
/// the archive already contains all supporting files and these arrays must
/// be empty.
struct SkillBundle {
    source: SkillSource,
    /// Written into `~/.agents/skills/<name>/references/`.
    references: &'static [BundledFile],
    /// Written into `~/.agents/skills/<name>/scripts/` and made executable.
    scripts: &'static [BundledFile],
    /// Written into `~/.agents/skills/<name>/examples/`.
    examples: &'static [BundledFile],
}

// All 10 built-in skills with their associated supporting files.
// Used by `repo skills deploy` to produce self-contained skill directories,
// matching the pattern of e.g. `rust-best-practices` (SKILL.md + references/).
const ALL_SKILL_BUNDLES: &[SkillBundle] = &[
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM17JDVNJ333TN3R5BGZB5QS-tsdoc-voice.md",
            content: include_str!(
                "../../../defaults/skills/01KM17JDVNJ333TN3R5BGZB5QS-tsdoc-voice.md"
            ),
        }),
        references: &[BundledFile {
            filename: "tsdoc-spec.md",
            content: include_str!(
                "../../../defaults/references/01KM17JDVNJ333TN3R5BGZB5QS-tsdoc-spec.md"
            ),
        }],
        scripts: &[],
        examples: &[],
    },
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM188YV2CJ26QH6KNH2NWG1Z-mental-model.md",
            content: include_str!(
                "../../../defaults/skills/01KM188YV2CJ26QH6KNH2NWG1Z-mental-model.md"
            ),
        }),
        references: &[BundledFile {
            filename: "mental-model-schema.md",
            content: include_str!(
                "../../../defaults/references/01KM188YV2CJ26QH6KNH2NWG1Z-mental-model-schema.md"
            ),
        }],
        scripts: &[BundledFile {
            filename: "treemeta.sh",
            content: include_str!("../../../defaults/scripts/treemeta.sh"),
        }],
        examples: &[],
    },
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM18ZD23GC3TDVN7W0GX2000-adv-plan.md",
            content: include_str!(
                "../../../defaults/skills/01KM18ZD23GC3TDVN7W0GX2000-adv-plan.md"
            ),
        }),
        references: &[
            BundledFile {
                filename: "plan-schema-fields.md",
                content: include_str!(
                    "../../../defaults/references/01KM18ZD23GC3TDVN7W0GX2000-plan-schema-fields.md"
                ),
            },
            BundledFile {
                filename: "plan-schema.ts",
                content: include_str!(
                    "../../../defaults/schemas/01KM18ZD23GC3TDVN7W0GX2000-plan-schema.ts"
                ),
            },
        ],
        scripts: &[BundledFile {
            filename: "treemeta.sh",
            content: include_str!("../../../defaults/scripts/treemeta.sh"),
        }],
        examples: &[],
    },
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM1A13V4FY0371Y0AB7FSGX9-purpose-md.md",
            content: include_str!(
                "../../../defaults/skills/01KM1A13V4FY0371Y0AB7FSGX9-purpose-md.md"
            ),
        }),
        references: &[],
        scripts: &[],
        examples: &[],
    },
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM1A156P4VEY0KT304QXA466-testing-standards.md",
            content: include_str!(
                "../../../defaults/skills/01KM1A156P4VEY0KT304QXA466-testing-standards.md"
            ),
        }),
        references: &[],
        scripts: &[],
        examples: &[],
    },
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM1BKXK7ST4DT8P6YC1BTMRD-incremental-validation.md",
            content: include_str!(
                "../../../defaults/skills/01KM1BKXK7ST4DT8P6YC1BTMRD-incremental-validation.md"
            ),
        }),
        references: &[],
        scripts: &[],
        examples: &[],
    },
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM1BVKWT984AB0A4WPZRWWGX-review-plan.md",
            content: include_str!(
                "../../../defaults/skills/01KM1BVKWT984AB0A4WPZRWWGX-review-plan.md"
            ),
        }),
        // review-plan reads the same plan schema as adv-planning.
        references: &[
            BundledFile {
                filename: "plan-schema-fields.md",
                content: include_str!(
                    "../../../defaults/references/01KM18ZD23GC3TDVN7W0GX2000-plan-schema-fields.md"
                ),
            },
            BundledFile {
                filename: "plan-schema.ts",
                content: include_str!(
                    "../../../defaults/schemas/01KM18ZD23GC3TDVN7W0GX2000-plan-schema.ts"
                ),
            },
        ],
        scripts: &[],
        examples: &[],
    },
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM1YWRFYBBT98WV14WXDKJM4-prompt-builder.md",
            content: include_str!(
                "../../../defaults/skills/01KM1YWRFYBBT98WV14WXDKJM4-prompt-builder.md"
            ),
        }),
        references: &[
            BundledFile {
                filename: "schema-reference.md",
                content: include_str!(
                    "../../../defaults/references/01KM1YWRFYBBT98WV14WXDKJM4-schema-reference.md"
                ),
            },
            BundledFile {
                filename: "prompt-schema.ts",
                content: include_str!(
                    "../../../defaults/schemas/01KM1YWRFYBBT98WV14WXDKJM4-prompt-schema.ts"
                ),
            },
        ],
        scripts: &[],
        examples: &[],
    },
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM1Z6WK23PJQJ5PM9E9B07BC-behavioral-layer.md",
            content: include_str!(
                "../../../defaults/skills/01KM1Z6WK23PJQJ5PM9E9B07BC-behavioral-layer.md"
            ),
        }),
        references: &[BundledFile {
            filename: "trait-spec.md",
            content: include_str!(
                "../../../defaults/references/01KM1Z6WK23PJQJ5PM9E9B07BC-trait-spec.md"
            ),
        }],
        scripts: &[],
        examples: &[
            BundledFile {
                filename: "trait-example.md",
                content: include_str!(
                    "../../../defaults/examples/01KM1Z6WK23PJQJ5PM9E9B07BC-trait.md"
                ),
            },
            BundledFile {
                filename: "trait-template.md",
                content: include_str!(
                    "../../../defaults/templates/01KM1Z6WK23PJQJ5PM9E9B07BC-trait-template.md"
                ),
            },
        ],
    },
    SkillBundle {
        source: SkillSource::Plain(DefaultAsset {
            filename: "01KM23VWVQWH62NBFF0TTFWVXR-doc-hygiene.md",
            content: include_str!(
                "../../../defaults/skills/01KM23VWVQWH62NBFF0TTFWVXR-doc-hygiene.md"
            ),
        }),
        references: &[
            BundledFile {
                filename: "detection-patterns.md",
                content: include_str!(
                    "../../../defaults/references/01KM23VWVQWH62NBFF0TTFWVXR-detection-patterns.md"
                ),
            },
            BundledFile {
                filename: "sync-checks.md",
                content: include_str!(
                    "../../../defaults/references/01KM23VWVQWH62NBFF0TTFWVXR-sync-checks.md"
                ),
            },
            BundledFile {
                filename: "report-template.md",
                content: include_str!(
                    "../../../defaults/references/01KM23VWVQWH62NBFF0TTFWVXR-report-template.md"
                ),
            },
        ],
        scripts: &[],
        examples: &[],
    },
    // ── ZIP-packaged skills (.skill Anthropic distribution format) ────────────
    SkillBundle {
        source: SkillSource::Zip {
            filename: "cli-ux-patterns.skill",
            bytes: include_bytes!(
                "../../../defaults/skills/cli-ux-patterns.skill"
            ),
        },
        // ZIP already contains SKILL.md + references/ — no extra arrays needed.
        references: &[],
        scripts: &[],
        examples: &[],
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
    /// E.g. `obra/superpowers` or <https://github.com/vercel-labs/agent-skills>
    pub source: String,

    /// Specific skill within the source repo (for multi-skill repos).
    /// If omitted, installs all / matches by name.
    pub skill: Option<String>,

    /// Target agents. E.g. `["claude-code", "codex"]`. Empty = all detected.
    #[serde(default)]
    pub agents: Vec<String>,

    /// Install scope: "project" (default) or "global".
    #[serde(default = "default_scope")]
    pub scope: String,

    /// Description (informational, not used for matching).
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
struct SkillsCheckItem {
    name: String,
    installed: bool,
    description: Option<String>,
    install_command: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct SkillsCheckReport {
    installed: usize,
    missing: usize,
    items: Vec<SkillsCheckItem>,
}

#[derive(Debug, Serialize)]
struct SkillsExportReport {
    exported: usize,
    skills: Vec<SkillEntry>,
}

#[derive(Debug, Serialize)]
struct SkillsSyncReport {
    kept: u32,
    added: u32,
    removed: Vec<String>,
    empty_sources: Vec<String>,
    skills: Vec<SkillEntry>,
}

#[derive(Debug, Serialize)]
struct SkillsInstallItem {
    name: String,
    outcome: String,
    source: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct SkillsInstallReport {
    items: Vec<SkillsInstallItem>,
    needs_fix: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SkillsFixReport {
    removed: Vec<RemovedSkill>,
    kept: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RemovedSkill {
    name: String,
    reason: String,
}

#[derive(Debug, Serialize)]
struct JsonError {
    error: String,
}

fn default_scope() -> String {
    "project".into()
}

impl SkillsConfig {
    #[must_use]
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

    #[must_use]
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }
}

// ── Public entry point ──────────────────────────────────────────────

pub fn run(repo_root: &Path, args: &[&str]) {
    let subcommand = args.first().copied().filter(|a| !a.starts_with('-'));
    let json = args.contains(&"--json");

    if args.iter().any(|a| *a == "--help" || *a == "-h") {
        print_help();
        return;
    }

    match subcommand {
        Some("init")   => cmd_init(repo_root),
        Some("export") => cmd_export(repo_root, json),
        Some("sync")   => cmd_sync(repo_root, json),
        Some("install") => cmd_install(repo_root, json),
        Some("fix")    => cmd_fix(repo_root, json),
        Some("deploy") => cmd_deploy(args),
        Some(other) => {
            eprintln!("Unknown skills subcommand: {other}");
            eprintln!("Run `repo skills --help` for usage.");
        }
        None => cmd_check(repo_root, json),
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
    --json       Emit machine-readable JSON where supported
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

fn cmd_export(repo_root: &Path, json: bool) {
    let installed = scan_installed_skills(repo_root);

    if installed.is_empty() {
        if json {
            let report = SkillsExportReport {
                exported: 0,
                skills: Vec::new(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
            );
            return;
        }
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

    if json {
        let report = SkillsExportReport {
            exported: config.skills.len(),
            skills: config.skills,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
        );
        return;
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

#[expect(clippy::too_many_lines)]
fn cmd_sync(repo_root: &Path, json: bool) {
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
                entry.description.clone_from(&on_disk.description);
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

    let empty_sources: Vec<String> = merged
        .iter()
        .filter(|s| s.source.is_empty())
        .map(|s| s.name.clone())
        .collect();

    let json_report = if json {
        Some(SkillsSyncReport {
            kept,
            added,
            removed: removed_names.clone(),
            empty_sources,
            skills: merged.clone(),
        })
    } else {
        None
    };

    // Calculate column width for output.
    let w_name = merged
        .iter()
        .chain(removed_names.iter().map(|n| &existing_by_name[n]))
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0)
        .max(4);

    if !json {
        println!("{}", bold("Syncing skills"));
        println!();
    }

    // Show what happened.
    for entry in &merged {
        let tag = if existing_by_name.contains_key(&entry.name) {
            dim("kept")
        } else {
            green("added")
        };
        if !json {
            println!(
                "  {} {:<w_name$}  {}",
                tag,
                entry.name,
                dim(entry.description.as_deref().unwrap_or("")),
            );
        }
    }

    if !json {
        for name in &removed_names {
            println!(
                "  {} {:<w_name$}  {}",
                yellow("gone"),
                name,
                dim("no longer installed — removed from config"),
            );
        }
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

    if let Some(report) = json_report {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
        );
        return;
    }

    if !json {
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
}

// ── check ───────────────────────────────────────────────────────────

#[expect(clippy::too_many_lines)]
fn cmd_check(repo_root: &Path, json: bool) {
    let Some(config) = SkillsConfig::load(repo_root) else {
        if json {
            let report = JsonError {
                error: "no .repo/skills.toml found".into(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
            );
            return;
        }
        println!("  {} no .repo/skills.toml found", dim("--"),);
        println!("  Run `repo skills init` to create one,");
        println!("  or `repo skills export` to snapshot installed skills.");
        return;
    };

    if config.skills.is_empty() {
        if json {
            let report = SkillsCheckReport {
                installed: 0,
                missing: 0,
                items: Vec::new(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
            );
            return;
        }
        println!("  No skills declared in .repo/skills.toml");
        return;
    }

    if !json {
        println!("{}", bold("Agent skills"));
        println!("  {}", dim("validating against .repo/skills.toml"));
        println!();
    }

    let skills_dir = repo_root.join(".agents").join("skills");
    let mut pass = 0u32;
    let mut fail = 0u32;
    let mut items = Vec::new();

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
            items.push(SkillsCheckItem {
                name: entry.name.clone(),
                installed: true,
                description: entry.description.clone(),
                install_command: None,
                message: None,
            });
            if !json {
                println!(
                    "  {} {:<w_name$}  {}",
                    green("ok"),
                    entry.name,
                    dim(entry.description.as_deref().unwrap_or("")),
                );
            }
            pass += 1;
        } else {
            let install_command = build_install_cmd(entry);
            items.push(SkillsCheckItem {
                name: entry.name.clone(),
                installed: false,
                description: entry.description.clone(),
                install_command: install_command.clone(),
                message: Some(if install_command.is_some() {
                    "not installed".into()
                } else {
                    "fill in 'source' in .repo/skills.toml to enable install".into()
                }),
            });
            if !json {
                println!(
                    "  {} {:<w_name$}  {}",
                    red("!!"),
                    entry.name,
                    red("not installed"),
                );
            }

            if let Some(install_cmd) = install_command {
                if !json {
                    println!(
                        "    {:<w_name$}  {}",
                        "",
                        dim(&format!("install: {install_cmd}")),
                    );
                }
            } else if !json {
                println!(
                    "    {:<w_name$}  {}",
                    "",
                    yellow("fill in 'source' in .repo/skills.toml to enable install"),
                );
            }

            fail += 1;
        }
    }

    if json {
        let report = SkillsCheckReport {
            installed: pass as usize,
            missing: fail as usize,
            items,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
        );
        if fail > 0 {
            std::process::exit(1);
        }
        return;
    }

    if !json {
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
        }
    }

    if fail > 0 {
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
    let Some(cmd) = build_install_cmd(entry) else {
        return InstallOutcome::NoSource;
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

#[expect(clippy::too_many_lines)]
fn cmd_install(repo_root: &Path, json: bool) {
    let Some(config) = SkillsConfig::load(repo_root) else {
        if json {
            let report = JsonError {
                error: "no .repo/skills.toml found".into(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
            );
            std::process::exit(1);
        }
        eprintln!("  No .repo/skills.toml found. Run `repo skills init` first.");
        std::process::exit(1);
    };

    let skills_dir = repo_root.join(".agents").join("skills");

    let missing: Vec<&SkillEntry> = config
        .skills
        .iter()
        .filter(|s| !skills_dir.join(&s.name).join("SKILL.md").is_file())
        .collect();

    if missing.is_empty() {
        if json {
            let report = SkillsInstallReport {
                items: Vec::new(),
                needs_fix: Vec::new(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
            );
            return;
        }
        println!("  {} all skills are installed", green("ok"));
        return;
    }

    if !json {
        println!("{} {} missing skill(s)", bold("Installing"), missing.len());
        println!();
    }

    let w_name = missing
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0)
        .max(4);

    let mut needs_fix: Vec<&str> = Vec::new();
    let mut items = Vec::new();

    for entry in &missing {
        let mut spinner = Spinner::start(format!("installing {}", entry.name));
        let outcome = run_install(entry);
        spinner.finish("");

        match outcome {
            InstallOutcome::Installed => {
                items.push(SkillsInstallItem {
                    name: entry.name.clone(),
                    outcome: "installed".into(),
                    source: Some(entry.source.clone()),
                    message: None,
                });
                if !json {
                    println!("  {} {}", green("ok"), entry.name);
                }
            }
            InstallOutcome::NoSource => {
                items.push(SkillsInstallItem {
                    name: entry.name.clone(),
                    outcome: "no_source".into(),
                    source: None,
                    message: Some("set 'source' in .repo/skills.toml".into()),
                });
                if !json {
                    println!(
                        "  {} {:<w_name$}  {}",
                        yellow("!!"),
                        entry.name,
                        yellow("no source — set 'source' in .repo/skills.toml or run: repo skills fix"),
                    );
                }
                needs_fix.push(&entry.name);
            }
            InstallOutcome::NotFound { ref source } => {
                items.push(SkillsInstallItem {
                    name: entry.name.clone(),
                    outcome: "not_found".into(),
                    source: Some(source.clone()),
                    message: Some(format!("skill not found in {source}")),
                });
                if !json {
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
                }
                needs_fix.push(&entry.name);
            }
            InstallOutcome::Failed { ref exit_status } => {
                items.push(SkillsInstallItem {
                    name: entry.name.clone(),
                    outcome: "failed".into(),
                    source: Some(entry.source.clone()),
                    message: Some(format!("install failed ({exit_status})")),
                });
                if !json {
                    eprintln!(
                        "  {} {:<w_name$}  {}",
                        red("!!"),
                        entry.name,
                        red(&format!("install failed ({exit_status})")),
                    );
                }
            }
            InstallOutcome::Error { ref message } => {
                items.push(SkillsInstallItem {
                    name: entry.name.clone(),
                    outcome: "error".into(),
                    source: Some(entry.source.clone()),
                    message: Some(format!("could not run install: {message}")),
                });
                if !json {
                    eprintln!(
                        "  {} {:<w_name$}  {}",
                        red("!!"),
                        entry.name,
                        red(&format!("could not run install: {message}")),
                    );
                }
            }
        }
        if !json {
            println!();
        }
    }

    if json {
        let report = SkillsInstallReport {
            items,
            needs_fix: needs_fix.iter().map(|name| (*name).to_string()).collect(),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
        );
        return;
    }

    if !json && !needs_fix.is_empty() {
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

#[expect(clippy::too_many_lines)]
fn cmd_fix(repo_root: &Path, json: bool) {
    let Some(config) = SkillsConfig::load(repo_root) else {
        if json {
            let report = JsonError {
                error: "no .repo/skills.toml found".into(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
            );
            std::process::exit(1);
        }
        eprintln!("  No .repo/skills.toml found. Run `repo skills init` first.");
        std::process::exit(1);
    };

    if config.skills.is_empty() {
        if json {
            let report = SkillsFixReport {
                removed: Vec::new(),
                kept: Vec::new(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
            );
            return;
        }
        println!("  {} .repo/skills.toml is empty, nothing to fix", dim("--"));
        return;
    }

    if !json {
        println!("{}", bold("Checking skills for fixable issues"));
        println!();
    }

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
            if !json {
                println!(
                    "  {} {:<w_name$}  {}",
                    yellow("rm"),
                    entry.name,
                    yellow("no source field — cannot install"),
                );
            }
            removed.push((entry.name.clone(), "no source"));
            continue;
        }

        // Probe: run the install command and check if skill is not found.
        let mut spinner = Spinner::start(format!("checking {}", entry.name));
        let outcome = run_install(entry);
        spinner.finish("");
        match outcome {
            InstallOutcome::Installed => {
                if !json {
                    println!("{}", green("ok (now installed)"));
                }
                keep.push(entry.clone());
            }
            InstallOutcome::NotFound { ref source } => {
                if !json {
                    println!("{}", red(&format!("skill not found in {source}")));
                    println!(
                        "    {:<w_name$}  {}",
                        "",
                        dim("removed — fix source/skill fields to re-add"),
                    );
                }
                removed.push((entry.name.clone(), "skill not found at source"));
            }
            InstallOutcome::NoSource => {
                if !json {
                    println!("{}", yellow("no source"));
                }
                removed.push((entry.name.clone(), "no source"));
            }
            InstallOutcome::Failed { ref exit_status } => {
                if !json {
                    println!("{}", yellow(&format!("install failed ({exit_status}) — kept")));
                    println!(
                        "    {:<w_name$}  {}",
                        "",
                        dim("kept — failure may be transient (network, auth). Re-run to retry."),
                    );
                }
                keep.push(entry.clone());
            }
            InstallOutcome::Error { ref message } => {
                if !json {
                    println!("{}", yellow(&format!("could not run install: {message} — kept")));
                }
                keep.push(entry.clone());
            }
        }
    }

    if json {
        let report = SkillsFixReport {
            removed: removed
                .iter()
                .map(|(name, reason)| RemovedSkill {
                    name: name.clone(),
                    reason: (*reason).to_string(),
                })
                .collect(),
            kept: keep.iter().map(|entry| entry.name.clone()).collect(),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
        );
        return;
    }

    if removed.is_empty() {
        if !json {
            println!();
            println!("  {} nothing to fix — all entries look valid", green("ok"));
        }
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

    if !json {
        println!();
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
}

// ── Helpers ─────────────────────────────────────────────────────────

fn build_install_cmd(entry: &SkillEntry) -> Option<String> {
    if entry.source.is_empty() {
        return None;
    }
    let mut cmd = format!("npx skills add {}", entry.source);

    // --skill flag.
    if let Some(ref skill) = entry.skill {
        let _ = write!(cmd, " --skill {skill}");
    }

    // --agent flags.
    for agent in &entry.agents {
        let _ = write!(cmd, " -a {agent}");
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

/// Returns the skill `name` from whichever source type a bundle uses.
fn skill_name_from_bundle(bundle: &SkillBundle) -> Option<String> {
    match &bundle.source {
        SkillSource::Plain(asset) => {
            parse_skill_name(asset.content).map(str::to_owned)
        }
        SkillSource::Zip { bytes, .. } => {
            // Read SKILL.md from the ZIP and parse the name field.
            let cursor = std::io::Cursor::new(*bytes);
            let mut archive = zip::ZipArchive::new(cursor).ok()?;
            let mut skill_md = archive.by_name("SKILL.md").ok()?;
            let mut content = String::new();
            std::io::Read::read_to_string(&mut skill_md, &mut content).ok()?;
            parse_skill_name(&content).map(str::to_owned)
        }
    }
}

/// Extract a `.skill` ZIP archive into `~/.agents/skills/<name>/`.
///
/// Returns `(skill_name, ref_count, script_count, example_count)` on success.
fn deploy_skill_zip(
    bytes: &[u8],
    source_filename: &str,
    canonical_base: &std::path::Path,
    force: bool,
) -> Result<(String, u32, u32, u32), String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("invalid ZIP in {source_filename}: {e}"))?;

    // Read SKILL.md first to extract the skill name.
    let skill_name = {
        let mut skill_md_file = archive
            .by_name("SKILL.md")
            .map_err(|_| format!("{source_filename}: no SKILL.md found in ZIP"))?;
        let mut content = String::new();
        std::io::Read::read_to_string(&mut skill_md_file, &mut content)
            .map_err(|e| format!("{source_filename}: read SKILL.md: {e}"))?;
        parse_skill_name(&content)
            .ok_or_else(|| format!("{source_filename}: no name: field in SKILL.md frontmatter"))?
            .to_owned()
    };

    let skill_dir = canonical_base.join(&skill_name);

    // Skip if already installed and not --force.
    if skill_dir.join("SKILL.md").exists() && !force {
        return Err(format!("__skip__{skill_name}"));
    }

    std::fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("{skill_name}: mkdir failed: {e}"))?;

    let mut refs = 0u32;
    let mut scripts = 0u32;
    let mut examples = 0u32;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("{skill_name}: zip entry {i}: {e}"))?;

        if entry.is_dir() {
            continue;
        }

        let entry_path = entry
            .enclosed_name()
            .ok_or_else(|| format!("{skill_name}: unsafe path in ZIP"))?
            .clone();

        let dest = skill_dir.join(&entry_path);

        if dest.exists() && !force {
            continue;
        }

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{skill_name}: mkdir {}: {e}", parent.display()))?;
        }

        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut content)
            .map_err(|e| format!("{skill_name}: read {}: {e}", entry_path.display()))?;

        std::fs::write(&dest, &content)
            .map_err(|e| format!("{skill_name}: write {}: {e}", dest.display()))?;

        // Count by subdirectory.
        let top = entry_path.components().next().map(|c| c.as_os_str().to_string_lossy().into_owned());
        match top.as_deref() {
            Some("references") => refs += 1,
            Some("scripts")    => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
                }
                scripts += 1;
            }
            Some("examples")   => examples += 1,
            _ => {} // SKILL.md or other root files
        }
    }

    Ok((skill_name, refs, scripts, examples))
}

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

/// Write `files` into `parent/subdir/`, creating the directory if needed.
/// Returns the count of files successfully written.
fn write_bundle_subdir(
    parent: &std::path::Path,
    subdir: &str,
    files: &[BundledFile],
    force: bool,
    executable: bool,
) -> u32 {
    if files.is_empty() {
        return 0;
    }
    let dir = parent.join(subdir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("  {} mkdir {}: {e}", yellow("!!"), dir.display());
        return 0;
    }
    let mut written = 0u32;
    for file in files {
        let path = dir.join(file.filename);
        if path.exists() && !force {
            continue;
        }
        if let Err(e) = std::fs::write(&path, file.content) {
            eprintln!("  {} write {}: {e}", yellow("!!"), path.display());
            continue;
        }
        #[cfg(unix)]
        if executable {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
        }
        written += 1;
    }
    written
}

#[allow(clippy::too_many_lines)]
fn cmd_deploy(args: &[&str]) {
    let force = args.iter().any(|a| *a == "--force" || *a == "-f");

    let Some(home) = get_home_dir() else {
        eprintln!("{} cannot determine home directory", red("!!"));
        std::process::exit(1);
    };

    let canonical_base = home.join(".agents").join("skills");

    // Detect agents whose config dir exists under $HOME.
    let detected: Vec<&KnownAgent> = KNOWN_AGENTS
        .iter()
        .filter(|a| home.join(a.config_dir).is_dir())
        .collect();

    println!("{}", bold("Deploying built-in skills"));
    println!();

    if detected.is_empty() {
        println!(
            "  {} no agent config dirs found — writing to {} only",
            yellow("!!"),
            dim("~/.agents/skills/"),
        );
    } else {
        let names: Vec<&str> = detected.iter().map(|a| a.name).collect();
        println!("  {}  {}", dim("agents :"), names.join(", "));
    }
    println!("  {}  ~/.agents/skills/", dim("install:"));
    println!();

    let mut installed = 0u32;
    let mut skipped   = 0u32;
    let mut failed    = 0u32;

    // Pre-compute display width from skill names across all source types.
    let w = ALL_SKILL_BUNDLES
        .iter()
        .filter_map(skill_name_from_bundle)
        .map(|n| n.len())
        .max()
        .unwrap_or(0);

    for bundle in ALL_SKILL_BUNDLES {
        // Resolve skill name and install, branching on source type.
        let (skill_name, refs, scripts, examples) = match &bundle.source {
            SkillSource::Plain(asset) => {
                let Some(name) = parse_skill_name(asset.content) else {
                    eprintln!("  {} could not parse name from {}", red("!!"), asset.filename);
                    failed += 1;
                    continue;
                };
                let skill_dir = canonical_base.join(name);
                let skill_md  = skill_dir.join("SKILL.md");
                if skill_md.exists() && !force {
                    println!("  {} {:<w$}  {}", dim("--"), name, dim("already installed"));
                    skipped += 1;
                    continue;
                }
                if let Err(e) = std::fs::create_dir_all(&skill_dir) {
                    eprintln!("  {} {name}: mkdir failed: {e}", red("!!"));
                    failed += 1;
                    continue;
                }
                if let Err(e) = std::fs::write(&skill_md, asset.content) {
                    eprintln!("  {} {name}: write SKILL.md failed: {e}", red("!!"));
                    failed += 1;
                    continue;
                }
                let r = write_bundle_subdir(&skill_dir, "references", bundle.references, force, false);
                let s = write_bundle_subdir(&skill_dir, "scripts",    bundle.scripts,    force, true);
                let e = write_bundle_subdir(&skill_dir, "examples",   bundle.examples,   force, false);
                (name.to_owned(), r, s, e)
            }
            SkillSource::Zip { filename, bytes } => {
                match deploy_skill_zip(bytes, filename, &canonical_base, force) {
                    Ok(result) => result,
                    Err(e) if e.starts_with("__skip__") => {
                        let name = &e["__skip__".len()..];
                        println!("  {} {:<w$}  {}", dim("--"), name, dim("already installed"));
                        skipped += 1;
                        continue;
                    }
                    Err(e) => {
                        eprintln!("  {} {filename}: {e}", red("!!"));
                        failed += 1;
                        continue;
                    }
                }
            }
        };
        let skill_name = skill_name.as_str();

        // Per-agent symlinks.
        let mut linked: Vec<&str> = Vec::new();

        for agent in &detected {
            let agent_skills_dir = home.join(agent.config_dir).join("skills");
            if let Err(e) = std::fs::create_dir_all(&agent_skills_dir) {
                eprintln!("  {} {skill_name}: mkdir {}: {e}", yellow("!!"), agent_skills_dir.display());
                continue;
            }
            let link_path = agent_skills_dir.join(skill_name);
            let target    = symlink_target(agent.config_dir, skill_name);

            let exists = link_path.exists() || link_path.symlink_metadata().is_ok();
            if exists {
                if force { let _ = std::fs::remove_file(&link_path); }
                else { continue; }
            }

            #[cfg(unix)]
            let result = std::os::unix::fs::symlink(&target, &link_path);
            #[cfg(not(unix))]
            let result = std::fs::copy(skill_dir.join("SKILL.md"), link_path.join("SKILL.md")).map(|_| ());

            match result {
                Ok(()) => linked.push(agent.name),
                Err(e) => eprintln!("  {} {skill_name}: symlink {}: {e}", yellow("!!"), agent.name),
            }
        }

        // Build extras annotation: show bundled subdirs when non-empty.
        let mut extras: Vec<String> = Vec::new();
        if refs     > 0 { extras.push(format!("{refs}ref")); }
        if scripts  > 0 { extras.push(format!("{scripts}script")); }
        if examples > 0 { extras.push(format!("{examples}example")); }

        let agents_str = if linked.is_empty() { String::new() } else { linked.join(", ") };
        let extras_str = if extras.is_empty() {
            String::new()
        } else {
            format!("  {}", dim(&format!("[{}]", extras.join(" "))))
        };

        println!("  {} {:<w$}  {}{extras_str}", green("ok"), skill_name, dim(&agents_str));
        installed += 1;
    }

    println!();
    print!("  {} installed, {} skipped", green(&installed.to_string()), skipped);
    if failed > 0 { print!(", {} {}", red(&failed.to_string()), red("failed")); }
    println!();
    if skipped > 0 {
        println!("  Run {} to overwrite.", dim("`repo skills deploy --force`"));
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
        let dir = std::env::temp_dir().join(format!("therepo-skills-{label}-{nanos}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    mod build_install_cmd {
        use super::*;

        #[test]
        fn returns_none_when_source_is_empty() {
            let entry = SkillEntry {
                name: "testing-standards".into(),
                source: String::new(),
                skill: None,
                agents: Vec::new(),
                scope: "project".into(),
                description: None,
            };

            assert_eq!(super::build_install_cmd(&entry), None);
        }

        #[test]
        fn includes_skill_agents_global_and_non_interactive_flags() {
            let entry = SkillEntry {
                name: "testing-standards".into(),
                source: "supercent-io/skills-template".into(),
                skill: Some("testing-standards".into()),
                agents: vec!["claude".into(), "codex".into()],
                scope: "global".into(),
                description: None,
            };

            assert_eq!(
                super::build_install_cmd(&entry).as_deref(),
                Some(
                    "npx skills add supercent-io/skills-template --skill testing-standards -a claude -a codex -g -y"
                )
            );
        }
    }

    mod scan_installed_skills {
        use super::*;

        #[test]
        fn ignores_non_skill_entries_and_sorts_results() {
            let repo_root = temp_dir("scan-installed");
            let skills_dir = repo_root.join(".agents").join("skills");
            fs::create_dir_all(&skills_dir).unwrap();

            fs::write(skills_dir.join("README.txt"), "ignored").unwrap();

            let beta = skills_dir.join("beta");
            fs::create_dir_all(&beta).unwrap();
            fs::write(beta.join("SKILL.md"), "---\ndescription: Beta skill\n---\n").unwrap();

            let alpha = skills_dir.join("alpha");
            fs::create_dir_all(&alpha).unwrap();
            fs::write(alpha.join("SKILL.md"), "---\ndescription: Alpha skill\n---\n").unwrap();

            let empty = skills_dir.join("empty-dir");
            fs::create_dir_all(&empty).unwrap();

            let entries = super::scan_installed_skills(&repo_root);

            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].name, "alpha");
            assert_eq!(entries[0].description.as_deref(), Some("Alpha skill"));
            assert_eq!(entries[1].name, "beta");
            assert_eq!(entries[1].description.as_deref(), Some("Beta skill"));

            fs::remove_dir_all(repo_root).ok();
        }
    }

    mod parse_skill_name {
        #[test]
        fn returns_none_without_frontmatter() {
            assert_eq!(super::parse_skill_name("# no frontmatter"), None);
        }

        #[test]
        fn returns_name_from_frontmatter() {
            assert_eq!(
                super::parse_skill_name("---\nname: testing-standards\n---\nbody"),
                Some("testing-standards")
            );
        }
    }

    mod symlink_target {
        #[test]
        fn computes_depth_for_single_segment_config_dir() {
            assert_eq!(
                super::symlink_target(".claude", "testing-standards"),
                "../../.agents/skills/testing-standards"
            );
        }

        #[test]
        fn computes_depth_for_nested_config_dir() {
            assert_eq!(
                super::symlink_target(".config/opencode", "testing-standards"),
                "../../../.agents/skills/testing-standards"
            );
        }
    }

    mod write_bundle_subdir {
        use super::*;

        #[test]
        fn skips_existing_files_without_force_and_overwrites_with_force() {
            let dir = temp_dir("write-bundle");
            let parent = dir.join("skill");
            fs::create_dir_all(&parent).unwrap();

            let files = [BundledFile {
                filename: "helper.sh",
                content: "echo first\n",
            }];

            assert_eq!(super::write_bundle_subdir(&parent, "scripts", &files, false, false), 1);
            assert_eq!(super::write_bundle_subdir(&parent, "scripts", &files, false, false), 0);

            let updated = [BundledFile {
                filename: "helper.sh",
                content: "echo second\n",
            }];
            assert_eq!(
                super::write_bundle_subdir(&parent, "scripts", &updated, true, false),
                1
            );
            assert_eq!(
                fs::read_to_string(parent.join("scripts").join("helper.sh")).unwrap(),
                "echo second\n"
            );

            fs::remove_dir_all(dir).ok();
        }
    }

    mod parse_skill_description {
        use super::*;

        #[test]
        fn returns_none_without_frontmatter() {
            let dir = temp_dir("description-none");
            let path = dir.join("SKILL.md");
            fs::write(&path, "# no frontmatter\n").unwrap();

            assert_eq!(super::parse_skill_description(&path), None);

            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn strips_quotes_and_truncates_long_descriptions() {
            let dir = temp_dir("description-truncate");
            let path = dir.join("SKILL.md");
            let description = "x".repeat(90);
            fs::write(
                &path,
                format!("---\ndescription: \"{description}\"\n---\nbody\n"),
            )
            .unwrap();

            let parsed = super::parse_skill_description(&path).unwrap();
            assert_eq!(parsed.len(), 80);
            assert!(parsed.ends_with("..."));

            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn returns_none_when_no_description_field() {
            let dir = temp_dir("description-missing-field");
            let path = dir.join("SKILL.md");
            fs::write(&path, "---\nname: my-skill\nauthor: Alice\n---\nbody\n").unwrap();

            assert_eq!(super::parse_skill_description(&path), None);

            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn returns_unquoted_description() {
            let dir = temp_dir("description-unquoted");
            let path = dir.join("SKILL.md");
            fs::write(&path, "---\nname: my-skill\ndescription: A short description\n---\nbody\n")
                .unwrap();

            assert_eq!(
                super::parse_skill_description(&path).as_deref(),
                Some("A short description")
            );

            fs::remove_dir_all(dir).ok();
        }
    }

    mod skills_config {
        use super::*;

        #[test]
        fn load_returns_none_for_missing_file() {
            let dir = temp_dir("config-missing");
            assert!(SkillsConfig::load(&dir).is_none());
            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn load_returns_none_for_invalid_toml() {
            let dir = temp_dir("config-invalid");
            let repo_dir = dir.join(".repo");
            fs::create_dir_all(&repo_dir).unwrap();
            fs::write(repo_dir.join("skills.toml"), "NOT VALID TOML ][").unwrap();

            assert!(SkillsConfig::load(&dir).is_none());

            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn load_parses_skill_entries() {
            let dir = temp_dir("config-valid");
            let repo_dir = dir.join(".repo");
            fs::create_dir_all(&repo_dir).unwrap();
            let content = "[[skills]]\nname = \"my-skill\"\nsource = \"owner/repo\"\nscope = \"project\"\n";
            fs::write(repo_dir.join("skills.toml"), content).unwrap();

            let cfg = SkillsConfig::load(&dir).unwrap();
            assert_eq!(cfg.skills.len(), 1);
            assert_eq!(cfg.skills[0].name, "my-skill");
            assert_eq!(cfg.skills[0].source, "owner/repo");

            fs::remove_dir_all(dir).ok();
        }

        #[test]
        fn to_toml_round_trips() {
            let cfg = SkillsConfig {
                skills: vec![SkillEntry {
                    name: "my-skill".into(),
                    source: "owner/repo".into(),
                    skill: None,
                    agents: vec![],
                    scope: "project".into(),
                    description: None,
                }],
            };
            let toml_str = cfg.to_toml();
            let parsed: SkillsConfig = toml::from_str(&toml_str).unwrap();
            assert_eq!(parsed.skills.len(), 1);
            assert_eq!(parsed.skills[0].name, "my-skill");
            assert_eq!(parsed.skills[0].scope, "project");
        }
    }

    mod skill_name_from_bundle {
        use super::*;

        #[test]
        fn returns_name_from_plain_bundle() {
            let plain = ALL_SKILL_BUNDLES
                .iter()
                .find(|b| matches!(b.source, SkillSource::Plain(_)));
            let bundle = plain.expect("at least one plain bundle must exist");
            let name = skill_name_from_bundle(bundle);
            assert!(name.is_some(), "expected name from plain bundle frontmatter");
        }

        #[test]
        fn returns_name_from_zip_bundle() {
            let zip = ALL_SKILL_BUNDLES
                .iter()
                .find(|b| matches!(b.source, SkillSource::Zip { .. }));
            if let Some(bundle) = zip {
                let name = skill_name_from_bundle(bundle);
                assert!(name.is_some(), "expected name from ZIP bundle SKILL.md");
            }
            // If no ZIP bundles exist the test is vacuously satisfied.
        }
    }

    mod get_home_dir {
        use super::*;

        #[test]
        fn returns_some_in_test_environment() {
            assert!(
                get_home_dir().is_some(),
                "HOME or USERPROFILE must be set in the test environment"
            );
        }
    }

    mod build_install_cmd_extra {
        use super::*;

        #[test]
        fn source_only_produces_minimal_command() {
            let entry = SkillEntry {
                name: "my-skill".into(),
                source: "owner/repo".into(),
                skill: None,
                agents: vec![],
                scope: "project".into(),
                description: None,
            };
            assert_eq!(
                build_install_cmd(&entry).as_deref(),
                Some("npx skills add owner/repo -y")
            );
        }
    }
}
