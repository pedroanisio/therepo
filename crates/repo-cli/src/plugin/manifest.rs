//! Deserialisation types for `plugin.toml` manifests.
//!
//! A `plugin.toml` sits at `.repo/plugins/<name>/plugin.toml` and declares
//! what a plugin provides, how to invoke it, and optional configuration for
//! its command, validation, and hook capabilities.
//!
//! # Example `plugin.toml`
//! ```toml
//! [plugin]
//! name = "my-linter"
//! version = "0.2.0"
//! description = "Run custom lint checks"
//! provides = ["command", "validation"]
//!
//! [command]
//! name = "lint"
//! help = "Run my-linter against the repo"
//! ```

use serde::Deserialize;
use std::path::Path;

/// Top-level structure of a `plugin.toml` manifest.
#[derive(Debug, Deserialize)]
// Optional config sections unused until Phase 2 plugin dispatch is implemented.
#[allow(dead_code)]
pub struct Manifest {
    pub plugin: PluginMeta,
    pub command: Option<CommandConfig>,
    pub validation: Option<ValidationConfig>,
    pub hooks: Option<HooksConfig>,
}

#[derive(Debug, Deserialize)]
// `command_path` unused until Phase 2 external plugin dispatch is implemented.
#[allow(dead_code)]
pub struct PluginMeta {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    pub command_path: Option<String>,
    #[serde(default)]
    pub provides: Vec<String>,
}

#[derive(Debug, Deserialize)]
// Aliases and help text used in Phase 2 when command dispatch is implemented.
#[allow(dead_code)]
pub struct CommandConfig {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub help: String,
}

#[derive(Debug, Deserialize)]
// Fields used in Phase 2 when the validation framework is implemented.
#[allow(dead_code)]
pub struct ValidationConfig {
    pub name: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
// Trigger list used in Phase 2 when hook execution is implemented.
#[allow(dead_code)]
pub struct HooksConfig {
    #[serde(default)]
    pub triggers: Vec<String>,
}

fn default_severity() -> String {
    "error".into()
}

/// Load and parse a `plugin.toml` file. Returns `None` if it doesn't exist
/// or fails to parse.
#[must_use]
pub fn load(path: &Path) -> Option<Manifest> {
    let content = std::fs::read_to_string(path).ok()?;
    match toml::from_str(&content) {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("Warning: failed to parse {}: {e}", path.display());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_file(label: &str, content: &str) -> std::path::PathBuf {
        let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("therepo-manifest-{label}-{nanos}-{unique}.toml"));
        fs::write(&path, content).unwrap();
        path
    }

    mod load {
        use super::*;

        #[test]
        fn parses_complete_manifest() {
            let path = temp_file(
                "valid",
                concat!(
                    "[plugin]\n",
                    "name = \"my-linter\"\n",
                    "version = \"0.2.0\"\n",
                    "description = \"Run custom lint checks\"\n",
                    "command_path = \"bin/my-linter\"\n",
                    "provides = [\"command\", \"validation\"]\n\n",
                    "[command]\n",
                    "name = \"lint\"\n",
                    "aliases = [\"check\", \"verify\"]\n",
                    "help = \"Run my-linter against the repo\"\n\n",
                    "[validation]\n",
                    "name = \"lint\"\n",
                    "severity = \"warning\"\n",
                    "description = \"Validate repository style\"\n\n",
                    "[hooks]\n",
                    "triggers = [\"pre-commit\", \"pre-push\"]\n",
                ),
            );

            let manifest = load(&path).expect("expected manifest to parse");

            assert_eq!(manifest.plugin.name, "my-linter");
            assert_eq!(manifest.plugin.version, "0.2.0");
            assert_eq!(manifest.plugin.description, "Run custom lint checks");
            assert_eq!(manifest.plugin.command_path.as_deref(), Some("bin/my-linter"));
            assert_eq!(manifest.plugin.provides, ["command", "validation"]);

            let command = manifest.command.expect("expected command section");
            assert_eq!(command.name, "lint");
            assert_eq!(command.aliases, ["check", "verify"]);
            assert_eq!(command.help, "Run my-linter against the repo");

            let validation = manifest.validation.expect("expected validation section");
            assert_eq!(validation.name, "lint");
            assert_eq!(validation.severity, "warning");
            assert_eq!(validation.description, "Validate repository style");

            let hooks = manifest.hooks.expect("expected hooks section");
            assert_eq!(hooks.triggers, ["pre-commit", "pre-push"]);

            fs::remove_file(path).ok();
        }

        #[test]
        fn returns_none_for_missing_file() {
            let path = std::env::temp_dir().join("therepo-manifest-missing.toml");

            assert!(load(&path).is_none());
        }

        #[test]
        fn returns_none_for_invalid_toml() {
            let path = temp_file("invalid", "this is ::: not toml");

            assert!(load(&path).is_none());

            fs::remove_file(path).ok();
        }
    }
}
