use crate::cli::{PromptArgs, PromptCommand, PromptListArgs};
use crate::config::find_repo_root;
use crate::plugin;

pub fn run(cmd: PromptArgs, json: bool) -> i32 {
    let repo_root = find_repo_root();
    super::overview::ensure_repo_dirs(&repo_root);

    let args = match cmd.command {
        Some(PromptCommand::Init) => {
            let mut args = vec!["init".to_string()];
            if json {
                args.push("--json".to_string());
            }
            args
        }
        Some(PromptCommand::List(flags)) => prompt_list_args(flags, json),
        Some(PromptCommand::Show(values)) => values,
        None => {
            if cmd.tag.is_some() || json {
                prompt_list_args(PromptListArgs {
                    tag: cmd.tag,
                }, json)
            } else {
                Vec::new()
            }
        }
    };

    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::prompt::run(&repo_root, &refs)
}

fn prompt_list_args(flags: PromptListArgs, json: bool) -> Vec<String> {
    let mut args = vec!["list".to_string()];
    if let Some(tag) = flags.tag {
        args.push("--tag".to_string());
        args.push(tag);
    }
    if json {
        args.push("--json".to_string());
    }
    args
}
