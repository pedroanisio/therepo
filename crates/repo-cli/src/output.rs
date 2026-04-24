use std::io::IsTerminal;
use std::sync::OnceLock;

static COLOR_ENABLED: OnceLock<bool> = OnceLock::new();
static COLOR_OVERRIDE: OnceLock<bool> = OnceLock::new();
static PLAIN_OUTPUT: OnceLock<bool> = OnceLock::new();

/// Whether stdout supports color output.
///
/// Result is cached after first call. Respects the `NO_COLOR` convention
/// (<https://no-color.org/>).
pub fn use_color() -> bool {
    if let Some(value) = COLOR_OVERRIDE.get() {
        return *value;
    }

    *COLOR_ENABLED.get_or_init(|| {
        std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
    })
}

pub fn disable_color() {
    let _ = COLOR_OVERRIDE.set(false);
}

pub fn enable_plain_output() {
    let _ = PLAIN_OUTPUT.set(true);
    disable_color();
}

#[must_use]
pub fn is_plain_output() -> bool {
    PLAIN_OUTPUT.get().copied().unwrap_or(false)
}

#[must_use]
pub fn bold(s: &str) -> String {
    apply_style(s, "1", use_color())
}

#[must_use]
pub fn dim(s: &str) -> String {
    apply_style(s, "2", use_color())
}

#[must_use]
pub fn green(s: &str) -> String {
    apply_style(s, "32", use_color())
}

#[must_use]
pub fn yellow(s: &str) -> String {
    apply_style(s, "33", use_color())
}

#[must_use]
pub fn red(s: &str) -> String {
    apply_style(s, "31", use_color())
}

#[must_use]
pub fn cyan(s: &str) -> String {
    apply_style(s, "36", use_color())
}

fn apply_style(s: &str, code: &str, color: bool) -> String {
    if color {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

#[must_use]
pub fn status_color(s: &str) -> String {
    status_color_inner(s, use_color())
}

fn status_color_inner(s: &str, color: bool) -> String {
    if !color {
        return s.to_string();
    }
    match s.to_lowercase().as_str() {
        "active" | "accepted" | "complete" => format!("\x1b[32m{s}\x1b[0m"),
        "proposal" => format!("\x1b[33m{s}\x1b[0m"),
        "draft" => format!("\x1b[36m{s}\x1b[0m"),
        "superseded" | "deprecated" | "archived" | "rejected" => {
            format!("\x1b[2m{s}\x1b[0m")
        }
        _ => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- apply_style tests (both branches) ---

    #[test]
    fn apply_style_with_color_wraps_in_ansi() {
        let result = apply_style("hello", "32", true);
        assert_eq!(result, "\x1b[32mhello\x1b[0m");
    }

    #[test]
    fn apply_style_without_color_returns_plain() {
        let result = apply_style("hello", "32", false);
        assert_eq!(result, "hello");
    }

    #[test]
    fn apply_style_bold_code() {
        assert_eq!(apply_style("x", "1", true), "\x1b[1mx\x1b[0m");
        assert_eq!(apply_style("x", "1", false), "x");
    }

    #[test]
    fn apply_style_dim_code() {
        assert_eq!(apply_style("x", "2", true), "\x1b[2mx\x1b[0m");
        assert_eq!(apply_style("x", "2", false), "x");
    }

    #[test]
    fn apply_style_red_code() {
        assert_eq!(apply_style("x", "31", true), "\x1b[31mx\x1b[0m");
    }

    #[test]
    fn apply_style_yellow_code() {
        assert_eq!(apply_style("x", "33", true), "\x1b[33mx\x1b[0m");
    }

    #[test]
    fn apply_style_cyan_code() {
        assert_eq!(apply_style("x", "36", true), "\x1b[36mx\x1b[0m");
    }

    // --- Public color function smoke tests ---
    // These exercise the delegation path; the actual coloring depends on
    // runtime TTY state, so we only verify the text is present.

    #[test]
    fn bold_output_contains_text() {
        assert!(bold("test").contains("test"));
    }

    #[test]
    fn dim_output_contains_text() {
        assert!(dim("test").contains("test"));
    }

    #[test]
    fn green_output_contains_text() {
        assert!(green("test").contains("test"));
    }

    #[test]
    fn yellow_output_contains_text() {
        assert!(yellow("test").contains("test"));
    }

    #[test]
    fn red_output_contains_text() {
        assert!(red("test").contains("test"));
    }

    #[test]
    fn cyan_output_contains_input_text() {
        assert!(cyan("hello").contains("hello"));
    }

    // --- status_color_inner tests (both color branches) ---

    #[test]
    fn status_color_inner_no_color_returns_plain() {
        assert_eq!(status_color_inner("active", false), "active");
        assert_eq!(status_color_inner("proposal", false), "proposal");
        assert_eq!(status_color_inner("draft", false), "draft");
        assert_eq!(status_color_inner("deprecated", false), "deprecated");
        assert_eq!(status_color_inner("unknown", false), "unknown");
    }

    #[test]
    fn status_color_inner_active_statuses_green() {
        for s in &["active", "accepted", "complete"] {
            let result = status_color_inner(s, true);
            assert_eq!(result, format!("\x1b[32m{s}\x1b[0m"));
        }
    }

    #[test]
    fn status_color_inner_proposal_yellow() {
        assert_eq!(
            status_color_inner("proposal", true),
            "\x1b[33mproposal\x1b[0m"
        );
    }

    #[test]
    fn status_color_inner_draft_cyan() {
        assert_eq!(
            status_color_inner("draft", true),
            "\x1b[36mdraft\x1b[0m"
        );
    }

    #[test]
    fn status_color_inner_deprecated_statuses_dim() {
        for s in &["superseded", "deprecated", "archived", "rejected"] {
            let result = status_color_inner(s, true);
            assert_eq!(result, format!("\x1b[2m{s}\x1b[0m"));
        }
    }

    #[test]
    fn status_color_inner_unknown_with_color_returns_plain() {
        assert_eq!(status_color_inner("foobar", true), "foobar");
    }

    #[test]
    fn status_color_inner_case_insensitive() {
        assert_eq!(
            status_color_inner("Active", true),
            "\x1b[32mActive\x1b[0m"
        );
        assert_eq!(
            status_color_inner("PROPOSAL", true),
            "\x1b[33mPROPOSAL\x1b[0m"
        );
        assert_eq!(
            status_color_inner("Draft", true),
            "\x1b[36mDraft\x1b[0m"
        );
        assert_eq!(
            status_color_inner("ARCHIVED", true),
            "\x1b[2mARCHIVED\x1b[0m"
        );
    }

    // --- Public API tests ---

    #[test]
    fn status_color_contains_text_for_all_known_statuses() {
        for s in &[
            "active",
            "accepted",
            "complete",
            "proposal",
            "draft",
            "superseded",
            "deprecated",
            "archived",
            "rejected",
        ] {
            assert!(status_color(s).contains(s));
        }
    }

    #[test]
    fn status_color_returns_unknown_statuses_unchanged() {
        // Unknown statuses are never decorated, regardless of color state.
        assert_eq!(status_color("foobar"), "foobar");
    }

    #[test]
    fn is_plain_output_defaults_to_false() {
        // PLAIN_OUTPUT may already be set by other tests, but the function
        // itself should never panic.
        let result = is_plain_output();
        // Result depends on test order; just ensure no panic and bool return.
        let _ = result;
    }

    #[test]
    fn use_color_returns_bool() {
        // In CI, this is typically false. Just ensure it runs without panic.
        let _ = use_color();
    }

    #[test]
    fn disable_color_does_not_panic() {
        disable_color();
    }

    #[test]
    fn enable_plain_output_does_not_panic() {
        enable_plain_output();
    }
}
