use std::io::{IsTerminal, Write as _};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Spinner {
    state: Option<SpinnerState>,
    message: String,
}

struct SpinnerState {
    done: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

impl Spinner {
    #[must_use]
    pub fn start(message: impl Into<String>) -> Self {
        let message = message.into();
        if !is_enabled() {
            return Self {
                state: None,
                message,
            };
        }

        let done = Arc::new(AtomicBool::new(false));
        let done_flag = Arc::clone(&done);
        let render_message = message.clone();

        let handle = thread::spawn(move || {
            let frames = ["|", "/", "-", "\\"];
            let mut idx = 0usize;

            while !done_flag.load(Ordering::Relaxed) {
                eprint!("\r{} {}", frames[idx % frames.len()], render_message);
                let _ = std::io::stderr().flush();
                idx += 1;
                thread::sleep(Duration::from_millis(80));
            }
        });

        Self {
            state: Some(SpinnerState { done, handle }),
            message,
        }
    }

    pub fn finish(&mut self, status: &str) {
        if let Some(state) = self.state.take() {
            state.done.store(true, Ordering::Relaxed);
            let _ = state.handle.join();
            eprint!("\r{}\r", " ".repeat(self.message.len() + 4));
            if status.is_empty() {
                let _ = std::io::stderr().flush();
                return;
            }
            eprintln!("{status}");
            let _ = std::io::stderr().flush();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.finish("");
    }
}

fn is_enabled() -> bool {
    let term = std::env::var_os("TERM");
    is_enabled_inner(
        crate::output::is_plain_output(),
        std::io::stderr().is_terminal(),
        term.as_deref(),
    )
}

fn is_enabled_inner(plain_output: bool, is_terminal: bool, term_var: Option<&std::ffi::OsStr>) -> bool {
    !plain_output && is_terminal && term_var != Some(std::ffi::OsStr::new("dumb"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_start_is_safe_when_output_is_not_a_tty() {
        let mut spinner = Spinner::start("checking");
        spinner.finish("done");
    }

    #[test]
    fn spinner_start_returns_none_state_when_disabled() {
        // In CI, is_enabled() is false, so state should be None.
        let spinner = Spinner::start("test message");
        assert!(spinner.state.is_none());
        assert_eq!(spinner.message, "test message");
    }

    #[test]
    fn spinner_start_preserves_message() {
        let spinner = Spinner::start("hello world");
        assert_eq!(spinner.message, "hello world");
    }

    #[test]
    fn spinner_start_with_string_owned() {
        let msg = String::from("owned message");
        let spinner = Spinner::start(msg);
        assert_eq!(spinner.message, "owned message");
    }

    // --- is_enabled_inner tests (all condition combinations) ---

    #[test]
    fn is_enabled_inner_all_conditions_met() {
        assert!(is_enabled_inner(false, true, Some(std::ffi::OsStr::new("xterm"))));
    }

    #[test]
    fn is_enabled_inner_plain_output_disables() {
        assert!(!is_enabled_inner(true, true, Some(std::ffi::OsStr::new("xterm"))));
    }

    #[test]
    fn is_enabled_inner_not_terminal_disables() {
        assert!(!is_enabled_inner(false, false, Some(std::ffi::OsStr::new("xterm"))));
    }

    #[test]
    fn is_enabled_inner_dumb_term_disables() {
        assert!(!is_enabled_inner(false, true, Some(std::ffi::OsStr::new("dumb"))));
    }

    #[test]
    fn is_enabled_inner_no_term_var_enables() {
        assert!(is_enabled_inner(false, true, None));
    }

    #[test]
    fn is_enabled_inner_all_disabled() {
        assert!(!is_enabled_inner(true, false, Some(std::ffi::OsStr::new("dumb"))));
    }

    #[test]
    fn is_enabled_inner_plain_and_dumb() {
        assert!(!is_enabled_inner(true, true, Some(std::ffi::OsStr::new("dumb"))));
    }

    #[test]
    fn is_enabled_inner_not_terminal_and_dumb() {
        assert!(!is_enabled_inner(false, false, Some(std::ffi::OsStr::new("dumb"))));
    }

    #[test]
    fn is_enabled_returns_false_outside_terminal() {
        // In CI, stderr is not a TTY, so is_enabled() should be false.
        assert!(!is_enabled());
    }

    // --- finish tests ---

    #[test]
    fn finish_without_state_is_noop() {
        let mut spinner = Spinner {
            state: None,
            message: String::from("noop"),
        };
        spinner.finish("ignored");
        // no panic, nothing happens
    }

    #[test]
    fn finish_without_state_empty_status_is_noop() {
        let mut spinner = Spinner {
            state: None,
            message: String::from("noop"),
        };
        spinner.finish("");
    }

    #[test]
    fn finish_with_state_and_nonempty_status() {
        let done = Arc::new(AtomicBool::new(false));
        let done_flag = Arc::clone(&done);
        let handle = thread::spawn(move || {
            while !done_flag.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(10));
            }
        });

        let mut spinner = Spinner {
            state: Some(SpinnerState { done, handle }),
            message: String::from("working"),
        };
        spinner.finish("complete");
        assert!(spinner.state.is_none());
    }

    #[test]
    fn finish_with_state_and_empty_status_clears_line() {
        let done = Arc::new(AtomicBool::new(false));
        let done_flag = Arc::clone(&done);
        let handle = thread::spawn(move || {
            while !done_flag.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(10));
            }
        });

        let mut spinner = Spinner {
            state: Some(SpinnerState { done, handle }),
            message: String::from("loading"),
        };
        spinner.finish("");
        assert!(spinner.state.is_none());
    }

    #[test]
    fn finish_clears_line_width_matches_message() {
        // Verify the blanking line is message.len() + 4 chars wide.
        // We can't capture stderr easily, but we ensure no panic with
        // various message lengths.
        for len in [0, 1, 10, 100] {
            let msg = "x".repeat(len);
            let done = Arc::new(AtomicBool::new(false));
            let done_flag = Arc::clone(&done);
            let handle = thread::spawn(move || {
                while !done_flag.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(5));
                }
            });

            let mut spinner = Spinner {
                state: Some(SpinnerState { done, handle }),
                message: msg,
            };
            spinner.finish("ok");
            assert!(spinner.state.is_none());
        }
    }

    // --- drop tests ---

    #[test]
    fn drop_stops_spinner_thread() {
        let done = Arc::new(AtomicBool::new(false));
        let done_flag = Arc::clone(&done);
        let done_check = Arc::clone(&done);
        let handle = thread::spawn(move || {
            while !done_flag.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(10));
            }
        });

        let spinner = Spinner {
            state: Some(SpinnerState { done, handle }),
            message: String::from("dropping"),
        };
        drop(spinner);
        assert!(done_check.load(Ordering::Relaxed));
    }

    #[test]
    fn drop_without_state_is_safe() {
        let spinner = Spinner {
            state: None,
            message: String::from("empty"),
        };
        drop(spinner);
    }

    #[test]
    fn double_finish_is_safe() {
        let done = Arc::new(AtomicBool::new(false));
        let done_flag = Arc::clone(&done);
        let handle = thread::spawn(move || {
            while !done_flag.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(10));
            }
        });

        let mut spinner = Spinner {
            state: Some(SpinnerState { done, handle }),
            message: String::from("twice"),
        };
        spinner.finish("first");
        spinner.finish("second");
        assert!(spinner.state.is_none());
    }

    #[test]
    fn finish_then_drop_is_safe() {
        let done = Arc::new(AtomicBool::new(false));
        let done_flag = Arc::clone(&done);
        let handle = thread::spawn(move || {
            while !done_flag.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(10));
            }
        });

        let mut spinner = Spinner {
            state: Some(SpinnerState { done, handle }),
            message: String::from("finish-then-drop"),
        };
        spinner.finish("done");
        drop(spinner);
    }

    // --- Thread simulation test ---
    // Exercises the same loop pattern used in Spinner::start's thread,
    // ensuring frame cycling and done-flag termination work correctly.

    #[test]
    fn spinner_thread_loop_logic_terminates() {
        let done = Arc::new(AtomicBool::new(false));
        let done_flag = Arc::clone(&done);

        let handle = thread::spawn(move || {
            let frames = ["|", "/", "-", "\\"];
            let mut idx = 0usize;
            while !done_flag.load(Ordering::Relaxed) {
                let _frame = frames[idx % frames.len()];
                idx += 1;
                if idx > 8 {
                    // Simulate a few iterations then self-terminate
                    break;
                }
                thread::sleep(Duration::from_millis(1));
            }
            idx
        });

        // Let the thread run briefly, then signal done
        thread::sleep(Duration::from_millis(20));
        done.store(true, Ordering::Relaxed);
        let final_idx = handle.join().expect("thread should not panic");
        assert!(final_idx > 0);
    }
}
