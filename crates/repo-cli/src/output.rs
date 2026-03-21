use std::io::IsTerminal;
use std::sync::OnceLock;

static COLOR_ENABLED: OnceLock<bool> = OnceLock::new();

/// Whether stdout supports color output.
///
/// Result is cached after first call. Respects the `NO_COLOR` convention
/// (<https://no-color.org/>).
pub fn use_color() -> bool {
    *COLOR_ENABLED.get_or_init(|| {
        std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
    })
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
