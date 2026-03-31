use crate::cli::{HealthArgs, HealthCommand};
use crate::config::find_repo_root;
use crate::plugin;

pub fn run(cmd: &HealthArgs, json: bool) -> i32 {
    let repo_root = find_repo_root();
    super::overview::ensure_repo_dirs(&repo_root);

    let args = match cmd.command {
        Some(HealthCommand::Init) => {
            let mut args = vec!["init".to_string()];
            if json {
                args.push("--json".to_string());
            }
            args
        }
        Some(HealthCommand::Export) => {
            let mut args = vec!["export".to_string()];
            if json {
                args.push("--json".to_string());
            }
            args
        }
        None => {
            let mut args = Vec::new();
            if cmd.verbose {
                args.push("--verbose".to_string());
            }
            if cmd.check_updates {
                args.push("--check-updates".to_string());
            }
            if json {
                args.push("--json".to_string());
            }
            args
        }
    };

    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::health::run(&repo_root, &refs)
}
