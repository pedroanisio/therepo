use crate::cli::{SkillsArgs, SkillsCommand};
use crate::config::find_repo_root;
use crate::plugin;

pub fn run(cmd: SkillsArgs, json: bool) -> i32 {
    let repo_root = find_repo_root();
    super::overview::ensure_repo_dirs(&repo_root);

    let args = match cmd.command {
        Some(SkillsCommand::Init) => vec!["init".to_string()],
        Some(SkillsCommand::Export) => with_json("export", json),
        Some(SkillsCommand::Sync) => with_json("sync", json),
        Some(SkillsCommand::Install) => with_json("install", json),
        Some(SkillsCommand::Fix) => with_json("fix", json),
        Some(SkillsCommand::Deploy(flags)) => {
            let mut args = vec!["deploy".to_string()];
            if flags.force {
                args.push("--force".to_string());
            }
            if json {
                args.push("--json".to_string());
            }
            args
        }
        None => {
            if json {
                vec!["--json".to_string()]
            } else {
                Vec::new()
            }
        }
    };

    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::skills::run(&repo_root, &refs)
}

fn with_json(name: &str, json: bool) -> Vec<String> {
    let mut args = vec![name.to_string()];
    if json {
        args.push("--json".to_string());
    }
    args
}
