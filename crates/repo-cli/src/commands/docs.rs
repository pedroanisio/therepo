use crate::cli::{DocsArgs, DocsCommand, DocsListArgs};
use crate::config::find_repo_root;
use crate::plugin;

pub fn run(cmd: DocsArgs, _json: bool) -> i32 {
    let repo_root = find_repo_root();
    super::overview::ensure_repo_dirs(&repo_root);

    let args = match cmd.command {
        Some(DocsCommand::Plans(flags)) => list_args("plans", flags),
        Some(DocsCommand::Designs(flags)) => list_args("designs", flags),
        Some(DocsCommand::Adrs(flags)) => list_args("adrs", flags),
        Some(DocsCommand::References(flags)) => list_args("references", flags),
        None => Vec::new(),
    };

    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::docs::run(&repo_root, &refs);
    0
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
