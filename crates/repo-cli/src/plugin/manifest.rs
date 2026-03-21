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
#[expect(dead_code)]
pub struct Manifest {
    pub plugin: PluginMeta,
    pub command: Option<CommandConfig>,
    pub validation: Option<ValidationConfig>,
    pub hooks: Option<HooksConfig>,
}

#[derive(Debug, Deserialize)]
// `command_path` unused until Phase 2 external plugin dispatch is implemented.
#[expect(dead_code)]
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
#[expect(dead_code)]
pub struct CommandConfig {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub help: String,
}

#[derive(Debug, Deserialize)]
// Fields used in Phase 2 when the validation framework is implemented.
#[expect(dead_code)]
pub struct ValidationConfig {
    pub name: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
// Trigger list used in Phase 2 when hook execution is implemented.
#[expect(dead_code)]
pub struct HooksConfig {
    #[serde(default)]
    pub triggers: Vec<String>,
}

fn default_severity() -> String {
    "error".into()
}

/// Load and parse a `plugin.toml` file. Returns `None` if it doesn't exist
/// or fails to parse.
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
