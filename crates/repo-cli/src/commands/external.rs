use crate::config::find_repo_root;
use crate::plugin::{Capability, discover_plugins};

#[must_use]
pub fn run(args: &[String]) -> i32 {
    let repo_root = find_repo_root();
    super::overview::ensure_repo_dirs(&repo_root);

    let Some(cmd) = args.first() else {
        eprintln!("Unknown command.");
        eprintln!("Run `repo --help` for usage.");
        return 2;
    };

    let plugins = discover_plugins(&repo_root);
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
