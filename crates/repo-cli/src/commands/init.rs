use crate::config::find_repo_root;
use crate::output::{dim, green, yellow};
use crate::plugin;

const BUILTIN_DOCS: &[(&str, &str)] = &[
    ("CLAUDE.md", include_str!("../../defaults/docs/CLAUDE.md")),
    (
        "DISCLAIMER.md",
        include_str!("../../defaults/docs/DISCLAIMER.md"),
    ),
];

pub fn run(json: bool) -> i32 {
    let repo_root = find_repo_root();
    super::overview::ensure_repo_dirs(&repo_root);

    if !json {
        println!("  {} initializing .repo/ in {}", green("=>"), repo_root.display());
        println!();
    }

    let mut overall: i32 = 0;

    if !json {
        println!("  {} health", green("::"));
    }
    let mut health_args = vec!["init".to_string()];
    if json {
        health_args.push("--json".to_string());
    }
    let refs: Vec<&str> = health_args.iter().map(String::as_str).collect();
    let code = plugin::builtin::health::run(&repo_root, &refs);
    if code != 0 {
        overall = code;
    }

    if !json {
        println!();
        println!("  {} skills, references, schemas", green("::"));
    }
    let skills_args = ["init".to_string()];
    let refs: Vec<&str> = skills_args.iter().map(String::as_str).collect();
    let code = plugin::builtin::skills::run(&repo_root, &refs);
    if code != 0 {
        overall = code;
    }

    if !json {
        println!();
        println!("  {} prompts", green("::"));
    }
    let mut prompt_args = vec!["init".to_string()];
    if json {
        prompt_args.push("--json".to_string());
    }
    let refs: Vec<&str> = prompt_args.iter().map(String::as_str).collect();
    let code = plugin::builtin::prompt::run(&repo_root, &refs);
    if code != 0 {
        overall = code;
    }

    if !json {
        println!();
        println!("  {} docs", green("::"));
    }
    for (name, content) in BUILTIN_DOCS {
        let target = repo_root.join(name);
        if target.exists() {
            if !json {
                println!("  {} {} already exists (not overwritten)", dim("--"), name);
            }
            continue;
        }
        if let Err(e) = std::fs::write(&target, *content) {
            if !json {
                eprintln!("  {} failed to write {name}: {e}", yellow("!!"));
            }
            overall = 1;
        } else if !json {
            println!("  {} wrote {name}", green("ok"));
        }
    }

    if json {
        println!(
            "{}",
            serde_json::json!({ "ok": overall == 0, "exit_code": overall })
        );
    } else {
        println!();
        if overall == 0 {
            println!("  {} repo initialized", green("done"));
        } else {
            println!("  {} repo init completed with errors", yellow("!!"));
        }
    }

    overall
}
