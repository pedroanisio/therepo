use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Health configuration loaded from `.repo/health.toml`.
///
/// Specifies required tools, their expected versions, and how
/// privilege escalation works in this environment.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HealthConfig {
    #[serde(default)]
    pub environment: EnvironmentConfig,

    /// Required tools. Key is the display name (e.g. "node", "rustc").
    #[serde(default)]
    pub tools: BTreeMap<String, ToolRequirement>,

    /// Custom health checks. Key is the check name.
    #[serde(default)]
    pub checks: BTreeMap<String, CustomCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomCheck {
    /// Shell command to run. Exit 0 = pass, non-zero = fail.
    pub command: String,

    /// Human-readable description shown in output.
    #[serde(default)]
    pub description: String,

    /// Severity: "error" (default, fails health check) or "warning".
    #[serde(default = "default_error")]
    pub severity: String,

    /// Hint shown on failure (e.g. how to fix).
    pub hint: Option<String>,
}

fn default_error() -> String {
    "error".into()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Privilege escalation method: "sudo", "doas", "pkexec", "none", or "auto".
    #[serde(default = "default_privilege")]
    pub privilege: String,

    /// Allowed runtime cages. Empty = any. E.g. ["host", "docker"].
    #[serde(default)]
    pub allowed_runtimes: Vec<String>,

    /// Required shell (must be in /etc/shells). E.g. "zsh".
    pub required_shell: Option<String>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            privilege: default_privilege(),
            allowed_runtimes: Vec::new(),
            required_shell: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolRequirement {
    /// Whether this tool is required or optional.
    #[serde(default = "default_true")]
    pub required: bool,

    /// Minimum version (inclusive). Compared numerically per segment.
    /// E.g. "1.70.0".
    pub min_version: Option<String>,

    /// Exact version constraint. E.g. "20.11.0".
    pub exact_version: Option<String>,

    /// The binary name to probe (defaults to the tool key).
    pub command: Option<String>,

    /// Arguments to get the version. Defaults to ["--version"].
    pub version_args: Option<Vec<String>>,

    /// URL to the source of truth for latest version / releases.
    pub url: Option<String>,

    /// Install command or instructions. Shown when the tool is missing
    /// or fails a version check.
    pub install: Option<String>,

    /// Command to check the latest available version.
    /// E.g. "npm" for `npm view <pkg> version`.
    pub latest_cmd: Option<String>,

    /// Arguments for the latest-version command.
    /// E.g. ["view", "skills", "version"].
    pub latest_args: Option<Vec<String>>,
}

fn default_privilege() -> String {
    "auto".into()
}

fn default_true() -> bool {
    true
}

impl HealthConfig {
    /// Load from `.repo/health.toml`. Returns `None` if the file doesn't exist.
    pub fn load(repo_root: &Path) -> Option<Self> {
        let path = repo_root.join(".repo").join("health.toml");
        let content = std::fs::read_to_string(&path).ok()?;
        match toml::from_str(&content) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {e}", path.display());
                None
            }
        }
    }

    /// Serialize to TOML string.
    pub fn to_toml(&self) -> String {
        // toml::to_string_pretty unwraps safely for our types.
        toml::to_string_pretty(self).unwrap_or_default()
    }
}

/// Generate a blank config template with comments.
pub fn blank_template() -> &'static str {
    include_str!("../../../defaults/health.toml")
}

struct ToolMeta {
    url: Option<String>,
    install: Option<String>,
    latest_cmd: Option<String>,
    latest_args: Option<Vec<String>>,
}

/// Well-known tool metadata (url, install hint, latest-version check).
fn tool_metadata(name: &str) -> ToolMeta {
    let npm_latest = |pkg: &str| -> (Option<String>, Option<Vec<String>>) {
        (
            Some("npm".into()),
            Some(vec!["view".into(), pkg.into(), "version".into()]),
        )
    };

    match name {
        "git" => ToolMeta {
            url: Some("https://git-scm.com/downloads".into()),
            install: Some("apt install git".into()),
            latest_cmd: None,
            latest_args: None,
        },
        "rustc" | "cargo" | "clippy" | "rustfmt" => ToolMeta {
            url: Some("https://github.com/rust-lang/rust/releases".into()),
            install: Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".into()),
            latest_cmd: Some("rustup".into()),
            latest_args: Some(vec!["check".into()]),
        },
        "node" => {
            let (lc, la) = npm_latest("node");
            ToolMeta {
                url: Some("https://nodejs.org/en/download".into()),
                install: Some(
                    "curl -fsSL https://fnm.vercel.app/install | bash && fnm install --lts".into(),
                ),
                latest_cmd: lc,
                latest_args: la,
            }
        }
        "npm" => {
            let (lc, la) = npm_latest("npm");
            ToolMeta {
                url: Some(
                    "https://docs.npmjs.com/downloading-and-installing-node-js-and-npm".into(),
                ),
                install: None,
                latest_cmd: lc,
                latest_args: la,
            }
        }
        "pnpm" => {
            let (lc, la) = npm_latest("pnpm");
            ToolMeta {
                url: Some("https://pnpm.io/installation".into()),
                install: Some("npm install -g pnpm".into()),
                latest_cmd: lc,
                latest_args: la,
            }
        }
        "bun" => ToolMeta {
            url: Some("https://bun.sh".into()),
            install: Some("curl -fsSL https://bun.sh/install | bash".into()),
            latest_cmd: None,
            latest_args: None,
        },
        "skills" => {
            let (lc, la) = npm_latest("skills");
            ToolMeta {
                url: Some("https://www.npmjs.com/package/skills".into()),
                install: Some("npm install -g skills".into()),
                latest_cmd: lc,
                latest_args: la,
            }
        }
        "python" => ToolMeta {
            url: Some("https://www.python.org/downloads/".into()),
            install: Some("apt install python3".into()),
            latest_cmd: None,
            latest_args: None,
        },
        "uv" => ToolMeta {
            url: Some("https://docs.astral.sh/uv/getting-started/installation/".into()),
            install: Some("curl -LsSf https://astral.sh/uv/install.sh | sh".into()),
            latest_cmd: Some("uv".into()),
            latest_args: Some(vec!["self".into(), "update".into(), "--dry-run".into()]),
        },
        "docker" => ToolMeta {
            url: Some("https://docs.docker.com/get-docker/".into()),
            install: Some("curl -fsSL https://get.docker.com | sh".into()),
            latest_cmd: None,
            latest_args: None,
        },
        "make" => ToolMeta {
            url: Some("https://www.gnu.org/software/make/".into()),
            install: Some("apt install make".into()),
            latest_cmd: None,
            latest_args: None,
        },
        "cmake" => ToolMeta {
            url: Some("https://cmake.org/download/".into()),
            install: Some("apt install cmake".into()),
            latest_cmd: None,
            latest_args: None,
        },
        "go" => ToolMeta {
            url: Some("https://go.dev/dl/".into()),
            install: Some("https://go.dev/doc/install".into()),
            latest_cmd: None,
            latest_args: None,
        },
        "zsh" => ToolMeta {
            url: Some("https://www.zsh.org/".into()),
            install: Some("apt install zsh".into()),
            latest_cmd: None,
            latest_args: None,
        },
        "bash" => ToolMeta {
            url: Some("https://www.gnu.org/software/bash/".into()),
            install: None,
            latest_cmd: None,
            latest_args: None,
        },
        _ => ToolMeta {
            url: None,
            install: None,
            latest_cmd: None,
            latest_args: None,
        },
    }
}

/// Build a `HealthConfig` by probing the current environment.
pub fn snapshot_current(checks: &[(String, Option<String>)], cage: &str) -> HealthConfig {
    let mut tools = BTreeMap::new();

    for (name, version) in checks {
        if let Some(ver) = version {
            let clean = extract_version_number(ver);
            let meta = tool_metadata(name);
            tools.insert(
                name.clone(),
                ToolRequirement {
                    required: true,
                    min_version: if clean.is_empty() { None } else { Some(clean) },
                    exact_version: None,
                    command: None,
                    version_args: None,
                    url: meta.url,
                    install: meta.install,
                    latest_cmd: meta.latest_cmd,
                    latest_args: meta.latest_args,
                },
            );
        }
    }

    // Detect privilege escalation.
    let privilege = detect_privilege();

    HealthConfig {
        environment: EnvironmentConfig {
            privilege,
            allowed_runtimes: vec![cage.to_string()],
            required_shell: std::env::var("SHELL").ok().and_then(|s| {
                std::path::Path::new(&s)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            }),
        },
        tools,
        checks: BTreeMap::new(),
    }
}

/// Try to extract a semver-like version from a raw string.
/// E.g. "rustc 1.94.0 (4a4ef493e 2026-03-02)" -> "1.94.0"
///      "v20.11.0" -> "20.11.0"
///      "Python 3.12.3" -> "3.12.3"
fn extract_version_number(raw: &str) -> String {
    for word in raw.split_whitespace() {
        let word = word.strip_prefix('v').unwrap_or(word);
        // Check if it looks like a version: starts with digit, contains a dot.
        if word.chars().next().is_some_and(|c| c.is_ascii_digit()) && word.contains('.') {
            // Take only the version part (strip trailing parens, commas, etc.).
            let clean: String = word
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if clean.contains('.') {
                return clean;
            }
        }
    }
    String::new()
}

fn detect_privilege() -> String {
    if which_exists("sudo") {
        "sudo".into()
    } else if which_exists("doas") {
        "doas".into()
    } else if which_exists("pkexec") {
        "pkexec".into()
    } else {
        "none".into()
    }
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Check the latest available version for a tool.
/// Uses `latest_cmd`/`latest_args` from config, falling back to built-in knowledge.
/// Returns `None` if no check is available or it fails.
pub fn check_latest_version(name: &str, cfg: Option<&HealthConfig>) -> Option<String> {
    // Try config first.
    let (cmd, args) = if let Some(req) = cfg.and_then(|c| c.tools.get(name)) {
        if let Some(ref lc) = req.latest_cmd {
            let la: Vec<String> = req
                .latest_args
                .clone()
                .unwrap_or_else(|| vec!["--version".into()]);
            (lc.clone(), la)
        } else {
            // Fall back to built-in.
            let meta = tool_metadata(name);
            (meta.latest_cmd?, meta.latest_args.unwrap_or_default())
        }
    } else {
        let meta = tool_metadata(name);
        (meta.latest_cmd?, meta.latest_args.unwrap_or_default())
    };

    let output = std::process::Command::new(&cmd).args(&args).output().ok()?;

    let raw = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr).into_owned()
    } else {
        String::from_utf8_lossy(&output.stdout).into_owned()
    };

    // For `rustup check`, parse lines like "stable - Up to date : 1.94.0 (...)"
    // or "stable - Update available : 1.94.0 -> 1.95.0 (...)"
    if cmd == "rustup" {
        for line in raw.lines() {
            if line.starts_with("stable") {
                if let Some(arrow_pos) = line.find("->") {
                    // "1.94.0 -> 1.95.0 (hash date)"
                    let after = &line[arrow_pos + 2..];
                    let ver = extract_version_number(after.trim());
                    if !ver.is_empty() {
                        return Some(ver);
                    }
                } else if line.contains("Up to date") {
                    return None; // already latest
                }
            }
        }
        return None;
    }

    // For npm view output, it's just a version number.
    let ver = extract_version_number(raw.trim());
    if ver.is_empty() { None } else { Some(ver) }
}
