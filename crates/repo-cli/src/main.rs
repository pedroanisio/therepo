mod config;
mod output;
mod plugin;

use config::{RepoConfig, find_repo_root};
use output::{bold, dim};
use plugin::{Capability, PluginInfo, discover_plugins};
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("repo {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Find the first non-flag argument as the subcommand.
    let subcommand = args[1..]
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str());

    if args.iter().any(|a| a == "--help" || a == "-h") && subcommand.is_none() {
        print_help();
        return;
    }

    let repo_root = find_repo_root();
    ensure_repo_dirs(&repo_root);
    let config = RepoConfig::load(&repo_root);

    let sub_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    match subcommand {
        Some("docs")   => plugin::builtin::docs::run(&repo_root, &sub_args),
        Some("health") => plugin::builtin::health::run(&repo_root, &sub_args),
        Some("skills") => plugin::builtin::skills::run(&repo_root, &sub_args),
        Some("prompt") => plugin::builtin::prompt::run(&repo_root, &sub_args),
        Some("ulid")   => plugin::builtin::ulid::run(&repo_root, &sub_args),
        Some("plugins") => cmd_plugins(&repo_root, &config, sub_args.first().copied()),
        Some(cmd) => {
            // Check if it matches an external plugin command.
            let plugins = discover_plugins(&repo_root);
            let matched = plugins.iter().find(|p| {
                !p.builtin && p.capabilities.contains(&Capability::Command) && p.name == cmd
            });

            if matched.is_some() {
                // TODO: execute external plugin — see issue #2 (Phase 2: external plugin dispatch)
                eprintln!("External plugin dispatch not yet implemented.");
                eprintln!("Plugin '{cmd}' was found but cannot be run yet.");
                process::exit(1);
            } else {
                eprintln!("Unknown command: {cmd}");
                eprintln!("Run `repo --help` for usage.");
                process::exit(1);
            }
        }
        None => cmd_overview(&repo_root, &config),
    }
}

// ── Help ────────────────────────────────────────────────────────────

fn print_help() {
    println!(
        "\
repo — Repository maintenance CLI

USAGE:
    repo [COMMAND] [OPTIONS]

COMMANDS:
    docs        Browse plans (.repo/storage), ADRs, and references (_docs/)
    health      Check development environment (tools, versions, config)
    skills      Check and install required agent skills
    prompt      Reusable prompt snippets for AI agents and workflows
    ulid        Generate valid ULIDs
    plugins     List discovered plugins

OPTIONS:
    -h, --help         Print this help message
    -V, --version      Print version

When no command is given, a repository overview is shown.

Configuration is loaded from .repo/config.toml. Plugins are discovered
from .repo/plugins/ and repo-* executables on $PATH."
    );
}

// ── Ensure required .repo directories exist ────────────────────────

fn ensure_repo_dirs(repo_root: &std::path::Path) {
    let storage_dir = repo_root.join(".repo").join("storage");
    if let Err(e) = std::fs::create_dir_all(&storage_dir) {
        eprintln!("Warning: could not create {}: {e}", storage_dir.display());
        return;
    }
    let keep = storage_dir.join(".keep");
    if !keep.exists() {
        if let Err(e) = std::fs::write(&keep, "") {
            eprintln!("Warning: could not write {}: {e}", keep.display());
        }
    }
}

// ── Overview command ────────────────────────────────────────────────

fn cmd_overview(repo_root: &std::path::Path, config: &RepoConfig) {
    let name = config.repo.name.as_deref().unwrap_or_else(|| {
        repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repo")
    });

    println!("{}", bold(name));
    println!();

    // Docs summary.
    plugin::builtin::docs::list_all(repo_root);
    println!();

    // Plugins summary.
    let plugins = discover_plugins(repo_root);
    let external_count = plugins.iter().filter(|p| !p.builtin).count();
    let builtin_count = plugins.iter().filter(|p| p.builtin).count();

    println!(
        "  {} {builtin_count} built-in, {external_count} external",
        bold("plugins"),
    );

    // Config status.
    let config_path = repo_root.join(".repo").join("config.toml");
    if config_path.is_file() {
        println!("  {} .repo/config.toml", dim("config "));
    } else {
        println!("  {} {}", dim("config "), dim("no .repo/config.toml found"),);
    }

    println!();
}

// ── Plugins command ─────────────────────────────────────────────────

fn cmd_plugins(repo_root: &std::path::Path, _config: &RepoConfig, sub: Option<&str>) {
    match sub {
        Some("list") | None => {
            let plugins = discover_plugins(repo_root);
            if plugins.is_empty() {
                println!("No plugins found.");
                return;
            }

            print_plugin_table(&plugins);
        }
        Some("info") => {
            eprintln!("Usage: repo plugins info <name> (not yet implemented)");
        }
        Some(other) => {
            eprintln!("Unknown plugins subcommand: {other}");
            eprintln!("Available: list, info");
        }
    }
}

fn print_plugin_table(plugins: &[PluginInfo]) {
    let w_name = plugins
        .iter()
        .map(|p| p.name.len())
        .max()
        .unwrap_or(0)
        .max(4);
    let w_ver = plugins
        .iter()
        .map(|p| p.version.len())
        .max()
        .unwrap_or(0)
        .max(7);
    let w_type = 10;
    let w_caps = 20;

    println!(
        "  {:<w_name$}  {:<w_ver$}  {:<w_type$}  {:<w_caps$}  {}",
        bold("NAME"),
        bold("VERSION"),
        bold("TYPE"),
        bold("CAPABILITIES"),
        bold("DESCRIPTION"),
    );

    println!(
        "  {}  {}  {}  {}  {}",
        dim(&"\u{2500}".repeat(w_name)),
        dim(&"\u{2500}".repeat(w_ver)),
        dim(&"\u{2500}".repeat(w_type)),
        dim(&"\u{2500}".repeat(w_caps)),
        dim(&"\u{2500}".repeat(30)),
    );

    for p in plugins {
        let kind = if p.builtin { "built-in" } else { "external" };
        let caps: Vec<&str> = p
            .capabilities
            .iter()
            .map(|c| match c {
                Capability::Command => "command",
                Capability::Validation => "validation",
                Capability::Hook => "hook",
            })
            .collect();

        println!(
            "  {:<w_name$}  {:<w_ver$}  {:<w_type$}  {:<w_caps$}  {}",
            p.name,
            p.version,
            kind,
            caps.join(", "),
            p.description,
        );
    }

    println!();
    println!("  {} plugin(s)", plugins.len());
}
