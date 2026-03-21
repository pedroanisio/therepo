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
    /// Privilege escalation method: `sudo`, `doas`, `pkexec`, `none`, or `auto`.
    #[serde(default = "default_privilege")]
    pub privilege: String,

    /// Allowed runtime cages. Empty = any. E.g. `["host", "docker"]`.
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

    /// Arguments to get the version. Defaults to `["--version"]`.
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
    /// E.g. `["view", "skills", "version"]`.
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
    #[must_use]
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
    #[must_use]
    pub fn to_toml(&self) -> String {
        // toml::to_string_pretty unwraps safely for our types.
        toml::to_string_pretty(self).unwrap_or_default()
    }
}

/// Generate a blank config template with comments.
#[must_use]
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
#[expect(clippy::too_many_lines)]
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
#[must_use]
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
#[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(name: &str) -> PathBuf {
        let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("therepo-health-{name}-{nanos}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_health_file(dir: &Path, content: &str) {
        let repo_dir = dir.join(".repo");
        fs::create_dir_all(&repo_dir).unwrap();
        let path = repo_dir.join("health.toml");
        fs::File::create(&path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
    }

    fn script_output(script: &str, name: &str) -> PathBuf {
        let path = temp_dir(name).join("latest.sh");
        fs::File::create(&path)
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        path
    }

    #[test]
    fn load_round_trips_serialized_config() {
        let dir = temp_dir("load-roundtrip");
        write_health_file(
            &dir,
            r#"
[environment]
privilege = "doas"
allowed_runtimes = ["host", "docker"]
required_shell = "zsh"

[tools.rustc]
required = true
min_version = "1.85.0"
url = "https://example.com/rust"
install = "rustup"

[checks.build]
command = "cargo test"
description = "Run the test suite"
severity = "warning"
hint = "Fix the tests first"
"#,
        );

        let cfg = HealthConfig::load(&dir).expect("config should load");

        assert_eq!(cfg.environment.privilege, "doas");
        assert_eq!(cfg.environment.allowed_runtimes, vec!["host", "docker"]);
        assert_eq!(cfg.environment.required_shell.as_deref(), Some("zsh"));
        assert_eq!(cfg.tools.get("rustc").and_then(|t| t.min_version.as_deref()), Some("1.85.0"));
        assert_eq!(cfg.checks.get("build").map(|c| c.severity.as_str()), Some("warning"));

        let rendered = cfg.to_toml();
        let parsed: HealthConfig = toml::from_str(&rendered).expect("rendered config should parse");
        assert_eq!(parsed.environment.privilege, "doas");
        assert_eq!(parsed.tools.get("rustc").and_then(|t| t.install.as_deref()), Some("rustup"));

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn load_returns_none_for_missing_file() {
        let dir = temp_dir("load-missing");

        assert!(HealthConfig::load(&dir).is_none());

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn extract_version_number_returns_clean_semver_fragment() {
        assert_eq!(
            extract_version_number("rustc 1.94.0 (4a4ef493e 2026-03-02)"),
            "1.94.0"
        );
        assert_eq!(extract_version_number("v20.11.0"), "20.11.0");
        assert_eq!(extract_version_number("Python 3.12.3"), "3.12.3");
        assert_eq!(extract_version_number("no version present"), "");
    }

    #[test]
    fn check_latest_version_uses_stdout_when_present() {
        let script = script_output("#!/usr/bin/env sh\necho 2.3.4\n", "latest-stdout");
        let mut tools = BTreeMap::new();
        tools.insert(
            "demo".to_string(),
            ToolRequirement {
                required: true,
                min_version: None,
                exact_version: None,
                command: None,
                version_args: None,
                url: None,
                install: None,
                latest_cmd: Some("sh".into()),
                latest_args: Some(vec![script.to_string_lossy().into_owned()]),
            },
        );
        let cfg = HealthConfig {
            environment: EnvironmentConfig::default(),
            tools,
            checks: BTreeMap::new(),
        };

        assert_eq!(check_latest_version("demo", Some(&cfg)), Some("2.3.4".into()));

        fs::remove_dir_all(script.parent().unwrap()).ok();
    }

    #[test]
    fn check_latest_version_uses_stderr_when_stdout_is_empty() {
        let script = script_output("#!/usr/bin/env sh\necho 3.4.5 1>&2\n", "latest-stderr");
        let mut tools = BTreeMap::new();
        tools.insert(
            "demo".to_string(),
            ToolRequirement {
                required: true,
                min_version: None,
                exact_version: None,
                command: None,
                version_args: None,
                url: None,
                install: None,
                latest_cmd: Some("sh".into()),
                latest_args: Some(vec![script.to_string_lossy().into_owned()]),
            },
        );
        let cfg = HealthConfig {
            environment: EnvironmentConfig::default(),
            tools,
            checks: BTreeMap::new(),
        };

        assert_eq!(check_latest_version("demo", Some(&cfg)), Some("3.4.5".into()));

        fs::remove_dir_all(script.parent().unwrap()).ok();
    }

    #[test]
    fn check_latest_version_returns_none_when_no_version_is_found() {
        let script = script_output("#!/usr/bin/env sh\necho not-a-version\n", "latest-empty");
        let mut tools = BTreeMap::new();
        tools.insert(
            "demo".to_string(),
            ToolRequirement {
                required: true,
                min_version: None,
                exact_version: None,
                command: None,
                version_args: None,
                url: None,
                install: None,
                latest_cmd: Some("sh".into()),
                latest_args: Some(vec![script.to_string_lossy().into_owned()]),
            },
        );
        let cfg = HealthConfig {
            environment: EnvironmentConfig::default(),
            tools,
            checks: BTreeMap::new(),
        };

        assert_eq!(check_latest_version("demo", Some(&cfg)), None);

        fs::remove_dir_all(script.parent().unwrap()).ok();
    }

    #[test]
    fn snapshot_current_captures_versions_and_cage() {
        let cfg = snapshot_current(
            &[
                ("rustc".to_string(), Some("rustc 1.95.1 (abc 2026-03-01)".to_string())),
                ("node".to_string(), Some("v20.11.0".to_string())),
                ("empty".to_string(), Some("nonsense".to_string())),
            ],
            "docker",
        );

        assert_eq!(cfg.environment.allowed_runtimes, vec!["docker"]);
        assert!(cfg.tools.contains_key("rustc"));
        assert_eq!(cfg.tools.get("rustc").and_then(|t| t.min_version.as_deref()), Some("1.95.1"));
        assert_eq!(cfg.tools.get("node").and_then(|t| t.min_version.as_deref()), Some("20.11.0"));
        assert_eq!(cfg.tools.get("empty").and_then(|t| t.min_version.as_deref()), None);
    }
}
