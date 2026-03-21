use crate::cli::UlidArgs;
use crate::config::find_repo_root;
use crate::plugin;

pub fn run(cmd: &UlidArgs) -> i32 {
    let repo_root = find_repo_root();
    super::overview::ensure_repo_dirs(&repo_root);

    let args = if cmd.count == 1 {
        Vec::new()
    } else {
        vec!["-n".to_string(), cmd.count.to_string()]
    };

    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    plugin::builtin::ulid::run(&repo_root, &refs);
    0
}
