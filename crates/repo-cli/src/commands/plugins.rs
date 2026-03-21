use crate::cli::{PluginsArgs, PluginsCommand};
use crate::config::{RepoConfig, find_repo_root};
use crate::output::{bold, dim};
use crate::plugin::{Capability, PluginInfo, discover_plugins};
use serde::Serialize;

#[derive(Serialize)]
struct PluginJson<'a> {
    name: &'a str,
    version: &'a str,
    description: &'a str,
    capabilities: Vec<&'static str>,
    builtin: bool,
    path: Option<&'a str>,
}

#[derive(Serialize)]
struct PluginsOverview<'a> {
    plugins: Vec<PluginJson<'a>>,
}

#[must_use]
pub fn run(args: PluginsArgs, json: bool) -> i32 {
    let repo_root = find_repo_root();
    super::overview::ensure_repo_dirs(&repo_root);
    let _config = RepoConfig::load(&repo_root);

    match args.command.unwrap_or(PluginsCommand::List) {
        PluginsCommand::List => {
            let plugins = discover_plugins(&repo_root);
            if json {
                print_json_list(&plugins);
                return 0;
            }

            if plugins.is_empty() {
                println!("No plugins found.");
                return 0;
            }

            print_plugin_table(&plugins);
        }
        PluginsCommand::Info { name } => {
            let plugins = discover_plugins(&repo_root);
            let Some(plugin) = plugins.iter().find(|plugin| plugin.name == name) else {
                eprintln!("Unknown plugin: {name}");
                eprintln!("Run `repo plugins list` to see available plugins.");
                return 1;
            };

            if json {
                print_json_plugin(plugin);
                return 0;
            }

            println!("{}", bold(&plugin.name));
            println!("  {} {}", dim("version:"), plugin.version);
            println!(
                "  {} {}",
                dim("type:"),
                if plugin.builtin { "built-in" } else { "external" }
            );
            println!("  {} {}", dim("description:"), plugin.description);
            println!(
                "  {} {}",
                dim("capabilities:"),
                capability_labels(&plugin.capabilities).join(", "),
            );
            if let Some(path) = &plugin.path {
                println!("  {} {}", dim("path:"), path);
            }
        }
    }

    0
}

fn print_json_list(plugins: &[PluginInfo]) {
    let payload = PluginsOverview {
        plugins: plugins.iter().map(as_json_plugin).collect(),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
    );
}

fn print_json_plugin(plugin: &PluginInfo) {
    println!(
        "{}",
        serde_json::to_string_pretty(&as_json_plugin(plugin)).unwrap_or_else(|_| "{}".to_string())
    );
}

fn as_json_plugin(plugin: &PluginInfo) -> PluginJson<'_> {
    PluginJson {
        name: &plugin.name,
        version: &plugin.version,
        description: &plugin.description,
        capabilities: capability_labels(&plugin.capabilities),
        builtin: plugin.builtin,
        path: plugin.path.as_deref(),
    }
}

fn capability_labels(capabilities: &[Capability]) -> Vec<&'static str> {
    capabilities
        .iter()
        .map(|capability| match capability {
            Capability::Command => "command",
            Capability::Validation => "validation",
            Capability::Hook => "hook",
        })
        .collect()
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

    for plugin in plugins {
        let kind = if plugin.builtin {
            "built-in"
        } else {
            "external"
        };

        println!(
            "  {:<w_name$}  {:<w_ver$}  {:<w_type$}  {:<w_caps$}  {}",
            plugin.name,
            plugin.version,
            kind,
            capability_labels(&plugin.capabilities).join(", "),
            plugin.description,
        );
    }

    println!();
    println!("  {} plugin(s)", plugins.len());
}
