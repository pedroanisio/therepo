//! Plugin discovery and capability model.
//!
//! This module defines the [`PluginInfo`] and [`Capability`] types used to
//! represent both built-in and external plugins, and provides
//! [`discover_plugins`] which scans the repo for all available plugins.
//!
//! # Plugin sources
//! - **Built-in**: compiled into the binary (docs, health, skills, prompt, ulid).
//! - **External**: discovered from `.repo/plugins/<name>/plugin.toml` or a
//!   same-named executable in that directory.
//!
//! Phase 2 will add scanning of `$REPO_PLUGIN_PATH` and `repo-*` on `$PATH`.

pub mod builtin;
pub mod manifest;

use std::path::Path;

/// What a plugin can provide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    Command,
    Validation,
    Hook,
}

/// A discovered plugin (built-in or external).
#[derive(Debug)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<Capability>,
    pub builtin: bool,
    // Read in tests; reserved for Phase 2 external plugin dispatch.
    // #[expect] doesn't suppress field-level dead_code on Debug-derived types (rustc quirk).
    #[allow(dead_code)]
    pub path: Option<String>,
}

/// Discover all available plugins (built-in + external).
#[must_use]
pub fn discover_plugins(repo_root: &Path) -> Vec<PluginInfo> {
    let mut plugins = vec![
        PluginInfo {
            name: "docs".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Browse plans, ADRs, and references in _docs/".into(),
            capabilities: vec![Capability::Command, Capability::Validation],
            builtin: true,
            path: None,
        },
        PluginInfo {
            name: "health".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Check development environment and repository health".into(),
            capabilities: vec![Capability::Command, Capability::Validation],
            builtin: true,
            path: None,
        },
        PluginInfo {
            name: "skills".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Manage required agent skills".into(),
            capabilities: vec![Capability::Command],
            builtin: true,
            path: None,
        },
        PluginInfo {
            name: "prompt".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Reusable prompt snippets for AI agents and workflows".into(),
            capabilities: vec![Capability::Command],
            builtin: true,
            path: None,
        },
        PluginInfo {
            name: "ulid".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Generate valid ULIDs".into(),
            capabilities: vec![Capability::Command],
            builtin: true,
            path: None,
        },
    ];

    // External plugins from .repo/plugins/.
    let plugins_dir = repo_root.join(".repo").join("plugins");
    if plugins_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&plugins_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let manifest_path = path.join("plugin.toml");
            if let Some(manifest) = manifest::load(&manifest_path) {
                plugins.push(PluginInfo {
                    name: manifest.plugin.name,
                    version: manifest.plugin.version,
                    description: manifest.plugin.description,
                    capabilities: manifest
                        .plugin
                        .provides
                        .iter()
                        .filter_map(|s| match s.as_str() {
                            "command" => Some(Capability::Command),
                            "validation" => Some(Capability::Validation),
                            "hook" => Some(Capability::Hook),
                            _ => None,
                        })
                        .collect(),
                    builtin: false,
                    path: Some(path.to_string_lossy().into_owned()),
                });
            } else {
                // Convention: directory with an executable of the same name.
                let exe_path = path.join(&name);
                if exe_path.is_file() {
                    plugins.push(PluginInfo {
                        name: name.clone(),
                        version: String::new(),
                        description: format!("Plugin at .repo/plugins/{name}"),
                        capabilities: vec![Capability::Command],
                        builtin: false,
                        path: Some(exe_path.to_string_lossy().into_owned()),
                    });
                }
            }
        }
    }

    // TODO: scan $REPO_PLUGIN_PATH and repo-* on $PATH — see issue #3 (Phase 2: PATH plugin discovery)

    plugins.sort_by(|a, b| a.name.cmp(&b.name));
    plugins
}

#[cfg(test)]
mod tests {
    use super::*;

    mod capability {
        use super::*;

        #[test]
        fn equality_holds_for_same_variant() {
            assert_eq!(Capability::Command, Capability::Command);
            assert_eq!(Capability::Validation, Capability::Validation);
            assert_eq!(Capability::Hook, Capability::Hook);
        }

        #[test]
        fn inequality_holds_for_different_variants() {
            assert_ne!(Capability::Command, Capability::Validation);
            assert_ne!(Capability::Command, Capability::Hook);
            assert_ne!(Capability::Validation, Capability::Hook);
        }
    }

    mod discover_plugins {
        use super::*;

        #[test]
        fn returns_sorted_builtin_plugins_when_no_external_plugins_dir() {
            let tmp = std::env::temp_dir().join("repo_test_no_plugins");
            std::fs::create_dir_all(&tmp).unwrap();
            let plugins = super::discover_plugins(&tmp);
            let names: Vec<&str> = plugins.iter().map(|p| p.name.as_str()).collect();
            // Registered built-ins are always present and sorted.
            assert!(names.contains(&"docs"));
            assert!(names.contains(&"prompt"));
            assert!(names.contains(&"ulid"));
            assert_eq!(names, {
                let mut sorted = names.clone();
                sorted.sort_unstable();
                sorted
            });
            std::fs::remove_dir_all(&tmp).ok();
        }

        #[test]
        fn all_builtins_are_marked_builtin() {
            let tmp = std::env::temp_dir().join("repo_test_builtin_flag");
            std::fs::create_dir_all(&tmp).unwrap();
            let plugins = super::discover_plugins(&tmp);
            for p in plugins.iter().filter(|p| p.builtin) {
                assert!(p.path.is_none(), "built-in '{}' should have no path", p.name);
            }
            std::fs::remove_dir_all(&tmp).ok();
        }

        #[test]
        fn external_plugin_loaded_from_plugin_toml() {
            let tmp = std::env::temp_dir().join("repo_test_ext_plugin");
            let plugin_dir = tmp.join(".repo").join("plugins").join("my-tool");
            std::fs::create_dir_all(&plugin_dir).unwrap();
            std::fs::write(
                plugin_dir.join("plugin.toml"),
                "[plugin]\nname = \"my-tool\"\nversion = \"1.0.0\"\ndescription = \"A test plugin\"\nprovides = [\"command\"]\n",
            )
            .unwrap();

            let plugins = super::discover_plugins(&tmp);
            let ext: Vec<_> = plugins.iter().filter(|p| !p.builtin).collect();
            assert_eq!(ext.len(), 1);
            assert_eq!(ext[0].name, "my-tool");
            assert_eq!(ext[0].version, "1.0.0");
            assert!(ext[0].capabilities.contains(&Capability::Command));
            std::fs::remove_dir_all(&tmp).ok();
        }
    }
}
