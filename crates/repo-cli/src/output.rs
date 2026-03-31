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
    if use_color() {
        format!("\x1b[1m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

#[must_use]
pub fn dim(s: &str) -> String {
    if use_color() {
        format!("\x1b[2m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

#[must_use]
pub fn green(s: &str) -> String {
    if use_color() {
        format!("\x1b[32m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

#[must_use]
pub fn yellow(s: &str) -> String {
    if use_color() {
        format!("\x1b[33m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

#[must_use]
pub fn red(s: &str) -> String {
    if use_color() {
        format!("\x1b[31m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

#[must_use]
pub fn cyan(s: &str) -> String {
    if use_color() {
        format!("\x1b[36m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

#[must_use]
pub fn status_color(s: &str) -> String {
    if !use_color() {
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

    // Color may or may not be enabled depending on the test runner's TTY.
    // All assertions check that the plain text is present in the output.

    #[test]
    fn cyan_output_contains_input_text() {
        assert!(cyan("hello").contains("hello"));
    }

    #[test]
    fn status_color_contains_text_for_active_statuses() {
        assert!(status_color("active").contains("active"));
        assert!(status_color("accepted").contains("accepted"));
        assert!(status_color("complete").contains("complete"));
    }

    #[test]
    fn status_color_contains_text_for_proposal() {
        assert!(status_color("proposal").contains("proposal"));
    }

    #[test]
    fn status_color_contains_text_for_draft() {
        assert!(status_color("draft").contains("draft"));
    }

    #[test]
    fn status_color_contains_text_for_deprecated_statuses() {
        assert!(status_color("superseded").contains("superseded"));
        assert!(status_color("deprecated").contains("deprecated"));
        assert!(status_color("archived").contains("archived"));
        assert!(status_color("rejected").contains("rejected"));
    }

    #[test]
    fn status_color_returns_unknown_statuses_unchanged() {
        // Unknown statuses are never decorated, regardless of color state.
        assert_eq!(status_color("foobar"), "foobar");
    }
}
