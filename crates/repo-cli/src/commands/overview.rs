use crate::config::{RepoConfig, find_repo_root};
use crate::output::{bold, dim};
use crate::plugin::discover_plugins;
use serde::Serialize;

#[derive(Serialize)]
struct OverviewJson<'a> {
    name: &'a str,
    builtin_plugins: usize,
    external_plugins: usize,
    config_present: bool,
}

#[must_use]
pub fn run(json: bool) -> i32 {
    let repo_root = find_repo_root();
    ensure_repo_dirs(&repo_root);
    let config = RepoConfig::load(&repo_root);

    let name = config.repo.name.as_deref().unwrap_or_else(|| {
        repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repo")
    });

    let plugins = discover_plugins(&repo_root);
    let external_count = plugins.iter().filter(|p| !p.builtin).count();
    let builtin_count = plugins.iter().filter(|p| p.builtin).count();
    let config_path = repo_root.join(".repo").join("config.toml");

    if json {
        let payload = OverviewJson {
            name,
            builtin_plugins: builtin_count,
            external_plugins: external_count,
            config_present: config_path.is_file(),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        );
        return 0;
    }

    println!("{}", bold(name));
    println!();

    crate::plugin::builtin::docs::list_all(&repo_root);
    println!();

    println!(
        "  {} {builtin_count} built-in, {external_count} external",
        bold("plugins"),
    );

    if config_path.is_file() {
        println!("  {} .repo/config.toml", dim("config "));
    } else {
        println!("  {} {}", dim("config "), dim("no .repo/config.toml found"));
    }

    println!();
    0
}

pub fn ensure_repo_dirs(repo_root: &std::path::Path) {
    let storage_dir = repo_root.join(".repo").join("storage");
    if let Err(e) = std::fs::create_dir_all(&storage_dir) {
        eprintln!("Warning: could not create {}: {e}", storage_dir.display());
        return;
    }

    let keep = storage_dir.join(".keep");
    if !keep.exists() && let Err(e) = std::fs::write(&keep, "") {
        eprintln!("Warning: could not write {}: {e}", keep.display());
    }
}
