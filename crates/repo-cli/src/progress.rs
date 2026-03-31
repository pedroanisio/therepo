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
    !crate::output::is_plain_output()
        && std::io::stderr().is_terminal()
        && std::env::var_os("TERM") != Some("dumb".into())
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
    fn is_enabled_returns_false_outside_terminal() {
        assert!(!is_enabled());
    }

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
}
