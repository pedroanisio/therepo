use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Repo-level configuration loaded from `.repo/config.toml`.
#[derive(Debug, Default, Deserialize)]
// Fields are consumed in Phase 2 (plugin dispatch, validation, hooks).
#[expect(dead_code)]
pub struct RepoConfig {
    #[serde(default)]
    pub repo: RepoSection,
    #[serde(default)]
    pub plugins: PluginsSection,
    #[serde(default)]
    pub hooks: HooksSection,
    #[serde(default)]
    pub check: CheckSection,
}

#[derive(Debug, Default, Deserialize)]
pub struct RepoSection {
    pub name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
// Fields used in Phase 2 when external plugin paths and disabling are implemented.
#[expect(dead_code)]
pub struct PluginsSection {
    #[serde(default)]
    pub extra_paths: Vec<String>,
    #[serde(default)]
    pub disabled: Vec<String>,
}

#[derive(Debug, Deserialize)]
// Fields used in Phase 2 when hook execution is implemented.
#[expect(dead_code)]
pub struct HooksSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub skip_in_ci: bool,
}

impl Default for HooksSection {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            timeout: default_timeout(),
            skip_in_ci: false,
        }
    }
}

#[derive(Debug, Deserialize)]
// Field used in Phase 2 when the validation framework is implemented.
pub struct CheckSection {
    // Read in tests; consumed in Phase 2 when validation is wired up.
    // #[expect] doesn't suppress field-level dead_code on Debug-derived types (rustc quirk).
    #[allow(dead_code)]
    #[serde(default = "default_fail_on")]
    pub fail_on: String,
}

impl Default for CheckSection {
    fn default() -> Self {
        Self {
            fail_on: default_fail_on(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    30
}

fn default_fail_on() -> String {
    "error".into()
}

impl RepoConfig {
    /// Load config from `.repo/config.toml` under the given root.
    ///
    /// Returns defaults if the file does not exist or cannot be parsed.
    pub fn load(repo_root: &Path) -> Self {
        let path = repo_root.join(".repo").join("config.toml");
        Self::load_from(&path)
    }

    pub(crate) fn load_from(path: &Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        match toml::from_str(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {e}", path.display());
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(name: &str, content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        std::fs::File::create(&path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        path
    }

    mod load_from {
        use super::*;

        #[test]
        fn returns_defaults_when_file_missing() {
            let cfg = RepoConfig::load_from(std::path::Path::new(
                "/nonexistent/repo_test_missing.toml",
            ));
            assert!(cfg.repo.name.is_none());
            assert!(cfg.hooks.enabled);
            assert_eq!(cfg.hooks.timeout, 30);
            assert_eq!(cfg.check.fail_on, "error");
        }

        #[test]
        fn parses_repo_name() {
            let path = write_temp(
                "repo_test_name.toml",
                "[repo]\nname = \"my-project\"\n",
            );
            let cfg = RepoConfig::load_from(&path);
            assert_eq!(cfg.repo.name.as_deref(), Some("my-project"));
            std::fs::remove_file(path).ok();
        }

        #[test]
        fn returns_defaults_on_invalid_toml() {
            let path = write_temp("repo_test_bad.toml", "this is ::: not valid toml");
            let cfg = RepoConfig::load_from(&path);
            assert!(cfg.repo.name.is_none());
            std::fs::remove_file(path).ok();
        }

        #[test]
        fn parses_hooks_section() {
            let path = write_temp(
                "repo_test_hooks.toml",
                "[hooks]\nenabled = false\ntimeout = 60\n",
            );
            let cfg = RepoConfig::load_from(&path);
            assert!(!cfg.hooks.enabled);
            assert_eq!(cfg.hooks.timeout, 60);
            std::fs::remove_file(path).ok();
        }

        #[test]
        fn parses_check_fail_on() {
            let path = write_temp(
                "repo_test_check.toml",
                "[check]\nfail_on = \"warning\"\n",
            );
            let cfg = RepoConfig::load_from(&path);
            assert_eq!(cfg.check.fail_on, "warning");
            std::fs::remove_file(path).ok();
        }
    }
}

/// Find the repository root by walking up from CWD.
///
/// Strong markers (`.repo/`, `.git/`) are collected across all ancestors;
/// the highest one wins — this ensures we find the true repo root even
/// when invoked from a nested subdirectory.  Weak markers (`Cargo.toml`)
/// are only used as a fallback when no strong marker exists anywhere.
pub fn find_repo_root() -> PathBuf {
    let start = std::env::current_dir().unwrap_or_default();
    let mut dir = start.clone();

    let mut last_strong: Option<PathBuf> = None;
    let mut first_weak: Option<PathBuf> = None;

    loop {
        // Strong markers — keep the highest (outermost) match.
        if dir.join(".repo").is_dir() || dir.join(".git").exists() {
            last_strong = Some(dir.clone());
        }

        // Weak marker — keep the first (deepest) match as fallback.
        if first_weak.is_none() && dir.join("Cargo.toml").exists() {
            first_weak = Some(dir.clone());
        }

        if !dir.pop() {
            break;
        }
    }

    last_strong.or(first_weak).unwrap_or(start)
}
