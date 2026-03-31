use crate::output::bold;
use std::path::Path;

#[must_use]
pub fn run(_repo_root: &Path, args: &[&str]) -> i32 {
    if args.iter().any(|a| *a == "--help" || *a == "-h") {
        print_help();
        return 0;
    }

    let json = args.contains(&"--json");
    let count = parse_count(args);
    if count == 0 {
        eprintln!("Error: count must be greater than zero.");
        return 1;
    }

    let values: Vec<String> = (0..count).map(|_| ulid::Ulid::new().to_string()).collect();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&values).unwrap_or_else(|_| "[]".to_string())
        );
    } else {
        for value in values {
            println!("{value}");
        }
    }

    0
}

fn parse_count(args: &[&str]) -> usize {
    args.windows(2)
        .find(|w| w[0] == "-n")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(1)
}

fn print_help() {
    println!(
        "\
{} — Generate valid ULIDs (Universally Unique Lexicographically Sortable Identifiers)

USAGE:
    repo ulid            Generate one ULID
    repo ulid -n <N>     Generate N ULIDs

OPTIONS:
    -n <N>         Number of ULIDs to generate (default: 1)
    -h, --help     Print this help message",
        bold("repo ulid"),
    );
}
