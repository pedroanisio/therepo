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
    std::io::stderr().is_terminal() && std::env::var_os("TERM") != Some("dumb".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_start_is_safe_when_output_is_not_a_tty() {
        let mut spinner = Spinner::start("checking");
        spinner.finish("done");
    }
}
