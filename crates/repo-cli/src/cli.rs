use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

const TOP_LEVEL_EXAMPLES: &str = "\
Examples:
  repo
  repo --json
  repo docs designs --status accepted --json
  repo health --verbose --json
  repo prompt list --tag review --json
  repo skills sync --json
  repo completions zsh";

const DOCS_EXAMPLES: &str = "\
Examples:
  repo docs
  repo docs plans --json
  repo docs plans plan-phase-0
  repo docs designs --status accepted
  repo docs plans --details incomplete --sort progress
  repo docs refs";

const HEALTH_EXAMPLES: &str = "\
Examples:
  repo health
  repo health --verbose
  repo health --check-updates --json
  repo health init
  repo health export";

const SKILLS_EXAMPLES: &str = "\
Examples:
  repo skills
  repo skills --json
  repo skills sync --json
  repo skills install
  repo skills deploy --force";

const PROMPT_EXAMPLES: &str = "\
Examples:
  repo prompt
  repo prompt list --tag review --json
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
pub struct Cli {
    /// Disable ANSI styling in human-readable output.
    #[arg(long, global = true)]
    pub plain: bool,

    /// Emit machine-readable JSON where supported.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: Option<DocsCommand>,
}

#[derive(Subcommand, Debug)]
pub enum DocsCommand {
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
pub struct DocsListArgs {
    /// Show details for a specific document by filename, stem, or title prefix.
    pub query: Option<String>,
    /// Filter by status.
    #[arg(long)]
    pub status: Option<String>,
    /// Sort the result set.
    #[arg(long, value_enum)]
    pub sort: Option<DocsSort>,
    /// Limit the number of listed documents.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Expand phase details in the human-readable output.
    #[arg(long, value_enum)]
    pub details: Option<DocsDetails>,
    /// Interactively choose one document to inspect.
    #[arg(long)]
    pub interactive: bool,
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum DocsSort {
    Date,
    Status,
    Title,
    Progress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum DocsDetails {
    None,
    Incomplete,
    All,
}

#[derive(Args, Debug)]
#[command(after_help = HEALTH_EXAMPLES)]
pub struct HealthArgs {
    #[command(subcommand)]
    pub command: Option<HealthCommand>,
    /// Show optional checks that are not present.
    #[arg(short = 'v', long)]
    pub verbose: bool,
    /// Check for newer available tool versions.
    #[arg(short = 'u', long = "check-updates")]
    pub check_updates: bool,
}

#[derive(Subcommand, Debug)]
pub enum HealthCommand {
    /// Create a blank .repo/health.toml template.
    Init,
    /// Snapshot the current environment into .repo/health.toml.
    Export,
}

#[derive(Args, Debug)]
#[command(after_help = SKILLS_EXAMPLES)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: Option<SkillsCommand>,
}

#[derive(Subcommand, Debug)]
pub enum SkillsCommand {
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
pub struct DeployArgs {
    /// Overwrite already-installed skills and symlinks.
    #[arg(short = 'f', long)]
    pub force: bool,
}

#[derive(Args, Debug)]
#[command(after_help = PROMPT_EXAMPLES)]
pub struct PromptArgs {
    /// Filter prompt listings by tag.
    #[arg(long)]
    pub tag: Option<String>,
    #[command(subcommand)]
    pub command: Option<PromptCommand>,
}

#[derive(Subcommand, Debug)]
pub enum PromptCommand {
    /// Write built-in defaults to .repo/prompts/.
    Init,
    /// List all available prompts.
    List(PromptListArgs),
    #[command(external_subcommand)]
    Show(Vec<String>),
}

#[derive(Args, Debug, Default)]
pub struct PromptListArgs {
    /// Filter prompts by tag.
    #[arg(long)]
    pub tag: Option<String>,
}

#[derive(Args, Debug)]
#[command(after_help = ULID_EXAMPLES)]
pub struct UlidArgs {
    /// Number of ULIDs to generate.
    #[arg(short = 'n', long, default_value_t = 1)]
    pub count: usize,
}

#[derive(Args, Debug)]
pub struct PluginsArgs {
    #[command(subcommand)]
    pub command: Option<PluginsCommand>,
}

#[derive(Subcommand, Debug)]
pub enum PluginsCommand {
    /// List discovered plugins.
    List,
    /// Show details about a specific discovered plugin.
    Info {
        /// Plugin name.
        name: String,
    },
}

#[derive(Args, Debug)]
pub struct CompletionArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: Shell,
}
