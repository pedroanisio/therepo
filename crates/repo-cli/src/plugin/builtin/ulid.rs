use crate::output::bold;
use std::path::Path;

pub fn run(_repo_root: &Path, args: &[&str]) {
    if args.iter().any(|a| *a == "--help" || *a == "-h") {
        print_help();
        return;
    }

    let count = parse_count(args);
    if count == 0 {
        eprintln!("Error: count must be greater than zero.");
        std::process::exit(1);
    }

    for _ in 0..count {
        println!("{}", ulid::Ulid::new());
    }
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
