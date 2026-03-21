pub mod config;
pub mod output;
pub mod plugin;

use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use config::{RepoConfig, find_repo_root};
use output::{bold, dim};
use plugin::{Capability, PluginInfo, discover_plugins};

const TOP_LEVEL_EXAMPLES: &str = "\
Examples:
  repo
  repo docs designs --status accepted --json
  repo health --verbose
  repo prompt format-plan
  repo skills init
  repo completions zsh";

const DOCS_EXAMPLES: &str = "\
Examples:
  repo docs
  repo docs plans --json
  repo docs designs --status accepted
  repo docs refs";

const HEALTH_EXAMPLES: &str = "\
Examples:
  repo health
  repo health --verbose
  repo health --check-updates
  repo health init
  repo health export";

const SKILLS_EXAMPLES: &str = "\
Examples:
  repo skills
  repo skills init
  repo skills export
  repo skills install
  repo skills deploy --force";

const PROMPT_EXAMPLES: &str = "\
Examples:
  repo prompt
  repo prompt list --tag review
  repo prompt format-plan
  repo prompt init";

const ULID_EXAMPLES: &str = "\
Examples:
  repo ulid
  repo ulid -n 3";

#[derive(Parser, Debug)]
#[command(
    name = "repo",
    version,
    about = "Repository maintenance CLI",
    long_about = None,
    propagate_version = true,
    after_help = TOP_LEVEL_EXAMPLES
)]
struct Cli {
    /// Disable ANSI styling in human-readable output.
    #[arg(long, global = true)]
    plain: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Browse plans, ADRs, and references.
    Docs(DocsArgs),
    /// Check development environment and repository health.
    Health(HealthArgs),
    /// Manage required agent skills.
    Skills(SkillsArgs),
    /// Reusable prompt snippets for AI workflows.
    Prompt(PromptArgs),
    /// Generate valid ULIDs.
    Ulid(UlidArgs),
    /// List discovered plugins.
    Plugins(PluginsArgs),
    /// Generate shell completion scripts.
    Completions(CompletionArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
#[command(after_help = DOCS_EXAMPLES)]
struct DocsArgs {
    #[command(subcommand)]
    command: Option<DocsCommand>,
}

#[derive(Subcommand, Debug)]
enum DocsCommand {
    /// List plans in .repo/storage/.
    Plans(DocsListArgs),
    /// List design documents in _docs/designs/.
    Designs(DocsListArgs),
    /// List ADRs in _docs/adrs/.
    Adrs(DocsListArgs),
    /// List reference documents in _docs/references/.
    #[command(alias = "refs")]
    References(DocsListArgs),
}

#[derive(Args, Debug, Default)]
struct DocsListArgs {
    /// Filter by status.
    #[arg(long)]
    status: Option<String>,
    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
#[command(after_help = HEALTH_EXAMPLES)]
struct HealthArgs {
    #[command(subcommand)]
    command: Option<HealthCommand>,
    /// Show optional checks that are not present.
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Check for newer available tool versions.
    #[arg(short = 'u', long = "check-updates")]
    check_updates: bool,
}

#[derive(Subcommand, Debug)]
enum HealthCommand {
    /// Create a blank .repo/health.toml template.
    Init,
    /// Snapshot the current environment into .repo/health.toml.
    Export,
}

#[derive(Args, Debug)]
#[command(after_help = SKILLS_EXAMPLES)]
struct SkillsArgs {
    #[command(subcommand)]
    command: Option<SkillsCommand>,
}

#[derive(Subcommand, Debug)]
enum SkillsCommand {
    /// Create .repo/skills.toml and copy built-in assets.
    Init,
    /// Snapshot installed skills into .repo/skills.toml.
    Export,
    /// Sync .repo/skills.toml with installed skills.
    Sync,
    /// Install missing skills declared in .repo/skills.toml.
    Install,
    /// Remove unfixable entries from .repo/skills.toml.
    Fix,
    /// Deploy built-in skills into the local agent skills ecosystem.
    Deploy(DeployArgs),
}

#[derive(Args, Debug)]
struct DeployArgs {
    /// Overwrite already-installed skills and symlinks.
    #[arg(short = 'f', long)]
    force: bool,
}

#[derive(Args, Debug)]
#[command(after_help = PROMPT_EXAMPLES)]
struct PromptArgs {
    /// Filter prompt listings by tag.
    #[arg(long)]
    tag: Option<String>,
    #[command(subcommand)]
    command: Option<PromptCommand>,
}

#[derive(Subcommand, Debug)]
enum PromptCommand {
    /// Write built-in defaults to .repo/prompts/.
    Init,
    /// List all available prompts.
    List(PromptListArgs),
    #[command(external_subcommand)]
    Show(Vec<String>),
}

#[derive(Args, Debug, Default)]
struct PromptListArgs {
    /// Filter prompts by tag.
    #[arg(long)]
    tag: Option<String>,
}

#[derive(Args, Debug)]
#[command(after_help = ULID_EXAMPLES)]
struct UlidArgs {
    /// Number of ULIDs to generate.
    #[arg(short = 'n', long, default_value_t = 1)]
    count: usize,
}

#[derive(Args, Debug)]
struct PluginsArgs {
    #[command(subcommand)]
    command: Option<PluginsCommand>,
}

#[derive(Subcommand, Debug)]
enum PluginsCommand {
    /// List discovered plugins.
    List,
    /// Show details about a specific discovered plugin.
    Info {
        /// Plugin name.
        name: String,
    },
}

#[derive(Args, Debug)]
struct CompletionArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    shell: Shell,
}

#[must_use]
pub fn run_cli(args: &[String]) -> i32 {
    let cli = match Cli::try_parse_from(args.iter().cloned()) {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            return err.exit_code();
        }
    };

    if cli.plain {
        output::disable_color();
    }

    match cli.command {
        Some(Commands::Docs(cmd)) => {
            let repo_root = find_repo_root();
            ensure_repo_dirs(&repo_root);
            dispatch_docs(&repo_root, cmd);
        }
        Some(Commands::Health(cmd)) => {
            let repo_root = find_repo_root();
            ensure_repo_dirs(&repo_root);
            dispatch_health(&repo_root, &cmd);
        }
        Some(Commands::Skills(cmd)) => {
            let repo_root = find_repo_root();
            ensure_repo_dirs(&repo_root);
            dispatch_skills(&repo_root, cmd);
        }
        Some(Commands::Prompt(cmd)) => {
            let repo_root = find_repo_root();
            ensure_repo_dirs(&repo_root);
            dispatch_prompt(&repo_root, cmd);
        }
        Some(Commands::Ulid(cmd)) => {
            let repo_root = find_repo_root();
            ensure_repo_dirs(&repo_root);
            dispatch_ulid(&repo_root, &cmd);
        }
        Some(Commands::Plugins(cmd)) => {
            let repo_root = find_repo_root();
            ensure_repo_dirs(&repo_root);
            let config = RepoConfig::load(&repo_root);
            cmd_plugins(&repo_root, &config, cmd);
        }
        Some(Commands::Completions(cmd)) => {
            let mut command = Cli::command();
            generate(cmd.shell, &mut command, "repo", &mut std::io::stdout());
        }
        Some(Commands::External(args)) => {
            let repo_root = find_repo_root();
            ensure_repo_dirs(&repo_root);
            return dispatch_external(&repo_root, &args);
        }
        None => {
            let repo_root = find_repo_root();
            ensure_repo_dirs(&repo_root);
            let config = RepoConfig::load(&repo_root);
            cmd_overview(&repo_root, &config);
        }
    }

    0
}

fn dispatch_docs(repo_root: &std::path::Path, cmd: DocsArgs) {
    let args = match cmd.command {
        Some(DocsCommand::Plans(flags)) => list_args("plans", flags),
        Some(DocsCommand::Designs(flags)) => list_args("designs", flags),
        Some(DocsCommand::Adrs(flags)) => list_args("adrs", flags),
        Some(DocsCommand::References(flags)) => list_args("references", flags),
        None => Vec::new(),
    };

    run_docs_args(repo_root, &args);
}

fn dispatch_health(repo_root: &std::path::Path, cmd: &HealthArgs) {
    let args = match cmd.command {
        Some(HealthCommand::Init) => vec!["init".to_string()],
        Some(HealthCommand::Export) => vec!["export".to_string()],
        None => {
            let mut args = Vec::new();
            if cmd.verbose {
                args.push("--verbose".to_string());
            }
            if cmd.check_updates {
                args.push("--check-updates".to_string());
            }
            args
        }
    };

    run_health_args(repo_root, &args);
}

fn dispatch_skills(repo_root: &std::path::Path, cmd: SkillsArgs) {
    let args = match cmd.command {
        Some(SkillsCommand::Init) => vec!["init".to_string()],
        Some(SkillsCommand::Export) => vec!["export".to_string()],
        Some(SkillsCommand::Sync) => vec!["sync".to_string()],
        Some(SkillsCommand::Install) => vec!["install".to_string()],
        Some(SkillsCommand::Fix) => vec!["fix".to_string()],
        Some(SkillsCommand::Deploy(flags)) => {
            let mut args = vec!["deploy".to_string()];
            if flags.force {
                args.push("--force".to_string());
            }
            args
        }
        None => Vec::new(),
    };

    run_skills_args(repo_root, &args);
}

fn dispatch_prompt(repo_root: &std::path::Path, cmd: PromptArgs) {
    let args = match cmd.command {
        Some(PromptCommand::Init) => vec!["init".to_string()],
        Some(PromptCommand::List(flags)) => prompt_list_args(flags),
        Some(PromptCommand::Show(values)) => values,
        None => {
            if let Some(tag) = cmd.tag {
                prompt_list_args(PromptListArgs { tag: Some(tag) })
            } else {
                Vec::new()
            }
        }
    };

    run_prompt_args(repo_root, &args);
}

fn dispatch_ulid(repo_root: &std::path::Path, cmd: &UlidArgs) {
    let args = if cmd.count == 1 {
        Vec::new()
    } else {
        vec!["-n".to_string(), cmd.count.to_string()]
    };

    run_ulid_args(repo_root, &args);
}

fn list_args(name: &str, flags: DocsListArgs) -> Vec<String> {
    let mut args = vec![name.to_string()];
    if let Some(status) = flags.status {
        args.push("--status".to_string());
        args.push(status);
    }
    if flags.json {
        args.push("--json".to_string());
    }
    args
}

fn prompt_list_args(flags: PromptListArgs) -> Vec<String> {
    let mut args = vec!["list".to_string()];
    if let Some(tag) = flags.tag {
        args.push("--tag".to_string());
        args.push(tag);
    }
    args
}

fn run_docs_args(repo_root: &std::path::Path, args: &[String]) {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::docs::run(repo_root, &refs);
}

fn run_health_args(repo_root: &std::path::Path, args: &[String]) {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::health::run(repo_root, &refs);
}

fn run_skills_args(repo_root: &std::path::Path, args: &[String]) {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::skills::run(repo_root, &refs);
}

fn run_prompt_args(repo_root: &std::path::Path, args: &[String]) {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::prompt::run(repo_root, &refs);
}

fn run_ulid_args(repo_root: &std::path::Path, args: &[String]) {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::ulid::run(repo_root, &refs);
}

fn dispatch_external(repo_root: &std::path::Path, args: &[String]) -> i32 {
    let Some(cmd) = args.first() else {
        eprintln!("Unknown command.");
        eprintln!("Run `repo --help` for usage.");
        return 2;
    };

    let plugins = discover_plugins(repo_root);
    let matched = plugins.iter().find(|plugin| {
        !plugin.builtin
            && plugin.capabilities.contains(&Capability::Command)
            && plugin.name == *cmd
    });

    if matched.is_some() {
        eprintln!("External plugin dispatch not yet implemented.");
        eprintln!("Plugin '{cmd}' was found but cannot be run yet.");
    } else {
        eprintln!("Unknown command: {cmd}");
        eprintln!("Run `repo --help` for usage.");
    }

    1
}

fn ensure_repo_dirs(repo_root: &std::path::Path) {
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

fn cmd_overview(repo_root: &std::path::Path, config: &RepoConfig) {
    let name = config.repo.name.as_deref().unwrap_or_else(|| {
        repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repo")
    });

    println!("{}", bold(name));
    println!();

    plugin::builtin::docs::list_all(repo_root);
    println!();

    let plugins = discover_plugins(repo_root);
    let external_count = plugins.iter().filter(|p| !p.builtin).count();
    let builtin_count = plugins.iter().filter(|p| p.builtin).count();

    println!(
        "  {} {builtin_count} built-in, {external_count} external",
        bold("plugins"),
    );

    let config_path = repo_root.join(".repo").join("config.toml");
    if config_path.is_file() {
        println!("  {} .repo/config.toml", dim("config "));
    } else {
        println!("  {} {}", dim("config "), dim("no .repo/config.toml found"));
    }

    println!();
}

fn cmd_plugins(repo_root: &std::path::Path, _config: &RepoConfig, args: PluginsArgs) {
    match args.command.unwrap_or(PluginsCommand::List) {
        PluginsCommand::List => {
            let plugins = discover_plugins(repo_root);
            if plugins.is_empty() {
                println!("No plugins found.");
                return;
            }

            print_plugin_table(&plugins);
        }
        PluginsCommand::Info { name } => {
            let plugins = discover_plugins(repo_root);
            let Some(plugin) = plugins.iter().find(|plugin| plugin.name == name) else {
                eprintln!("Unknown plugin: {name}");
                eprintln!("Run `repo plugins list` to see available plugins.");
                std::process::exit(1);
            };

            println!("{}", bold(&plugin.name));
            println!("  {} {}", dim("version:"), plugin.version);
            println!(
                "  {} {}",
                dim("type:"),
                if plugin.builtin { "built-in" } else { "external" }
            );
            println!("  {} {}", dim("description:"), plugin.description);
            let capabilities = plugin
                .capabilities
                .iter()
                .map(|capability| match capability {
                    Capability::Command => "command",
                    Capability::Validation => "validation",
                    Capability::Hook => "hook",
                })
                .collect::<Vec<_>>()
                .join(", ");
            println!("  {} {}", dim("capabilities:"), capabilities);
            if let Some(path) = &plugin.path {
                println!("  {} {}", dim("path:"), path);
            }
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

    for plugin in plugins {
        let kind = if plugin.builtin {
            "built-in"
        } else {
            "external"
        };
        let capabilities = plugin
            .capabilities
            .iter()
            .map(|capability| match capability {
                Capability::Command => "command",
                Capability::Validation => "validation",
                Capability::Hook => "hook",
            })
            .collect::<Vec<_>>();

        println!(
            "  {:<w_name$}  {:<w_ver$}  {:<w_type$}  {:<w_caps$}  {}",
            plugin.name,
            plugin.version,
            kind,
            capabilities.join(", "),
            plugin.description,
        );
    }

    println!();
    println!("  {} plugin(s)", plugins.len());
}
