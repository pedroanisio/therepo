use crate::cli::{DocsArgs, DocsCommand, DocsListArgs};
use crate::config::find_repo_root;
use crate::plugin;

pub fn run(cmd: DocsArgs, json: bool) -> i32 {
    let repo_root = find_repo_root();
    super::overview::ensure_repo_dirs(&repo_root);

    let args = match cmd.command {
        Some(DocsCommand::Plans(flags)) => list_args("plans", flags, json),
        Some(DocsCommand::Designs(flags)) => list_args("designs", flags, json),
        Some(DocsCommand::Adrs(flags)) => list_args("adrs", flags, json),
        Some(DocsCommand::References(flags)) => list_args("references", flags, json),
        None => {
            if json {
                vec!["--json".to_string()]
            } else {
                Vec::new()
            }
        }
    };

    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::docs::run(&repo_root, &refs)
}

fn list_args(name: &str, flags: DocsListArgs, global_json: bool) -> Vec<String> {
    let mut args = vec![name.to_string()];
    if let Some(query) = flags.query {
        args.push(query);
    }
    if let Some(status) = flags.status {
        args.push("--status".to_string());
        args.push(status);
    }
    if let Some(sort) = flags.sort {
        args.push("--sort".to_string());
        args.push(
            match sort {
                crate::cli::DocsSort::Date => "date",
                crate::cli::DocsSort::Status => "status",
                crate::cli::DocsSort::Title => "title",
                crate::cli::DocsSort::Progress => "progress",
            }
            .to_string(),
        );
    }
    if let Some(limit) = flags.limit {
        args.push("--limit".to_string());
        args.push(limit.to_string());
    }
    if let Some(details) = flags.details {
        args.push("--details".to_string());
        args.push(
            match details {
                crate::cli::DocsDetails::None => "none",
                crate::cli::DocsDetails::Incomplete => "incomplete",
                crate::cli::DocsDetails::All => "all",
            }
            .to_string(),
        );
    }
    if flags.interactive {
        args.push("--interactive".to_string());
    }
    if flags.json || global_json {
        args.push("--json".to_string());
    }
    args
}
