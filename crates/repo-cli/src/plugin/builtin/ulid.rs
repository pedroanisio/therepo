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

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_count_tests {
        use super::*;

        #[test]
        fn defaults_to_one_without_flag() {
            assert_eq!(parse_count(&[]), 1);
        }

        #[test]
        fn parses_valid_count() {
            assert_eq!(parse_count(&["-n", "5"]), 5);
        }

        #[test]
        fn returns_default_for_non_numeric_argument() {
            assert_eq!(parse_count(&["-n", "abc"]), 1);
        }

        #[test]
        fn ignores_n_at_end_without_value() {
            assert_eq!(parse_count(&["--json", "-n"]), 1);
        }
    }

    mod run_tests {
        use super::*;
        use std::path::PathBuf;

        #[test]
        fn help_flag_returns_zero() {
            let root = PathBuf::from("/tmp");
            assert_eq!(run(&root, &["--help"]), 0);
            assert_eq!(run(&root, &["-h"]), 0);
        }

        #[test]
        fn default_invocation_returns_zero() {
            let root = PathBuf::from("/tmp");
            assert_eq!(run(&root, &[]), 0);
        }

        #[test]
        fn json_flag_returns_zero() {
            let root = PathBuf::from("/tmp");
            assert_eq!(run(&root, &["--json"]), 0);
        }

        #[test]
        fn json_with_count_returns_zero() {
            let root = PathBuf::from("/tmp");
            assert_eq!(run(&root, &["--json", "-n", "3"]), 0);
        }
    }
}
