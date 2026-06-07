use std::io::{self, IsTerminal, Write};
use std::sync::{Mutex, OnceLock};

const LOADING_FRAMES: [&str; 4] = ["|", "/", "-", "\\"];

#[derive(Default)]
struct StatusLine {
    active: bool,
    interactive: bool,
    width: usize,
    message: String,
    frame: usize,
}

pub(crate) struct LoadingStatus {
    finished: bool,
    interactive: bool,
}

impl LoadingStatus {
    pub(crate) fn start(message: impl Into<String>) -> Self {
        let interactive = start_status(message.into());
        Self {
            finished: false,
            interactive,
        }
    }

    pub(crate) fn update(&self, message: impl Into<String>) {
        update_status(message.into());
    }

    pub(crate) fn is_interactive(&self) -> bool {
        self.interactive
    }

    pub(crate) fn tick(&self) {
        tick_status();
    }

    pub(crate) fn finish(mut self) {
        finish_status();
        self.finished = true;
    }
}

impl Drop for LoadingStatus {
    fn drop(&mut self) {
        if !self.finished {
            finish_status();
        }
    }
}

pub(crate) fn log_info(message: impl AsRef<str>) {
    log_stdout_line("INFO", message.as_ref());
}

pub(crate) fn log_error(message: impl AsRef<str>) {
    let mut state = status_state().lock().expect("status line lock");
    let redraw = state.clear();
    eprintln!("ERROR {}", message.as_ref());
    if redraw {
        state.draw();
    }
}

fn status_state() -> &'static Mutex<StatusLine> {
    static STATUS: OnceLock<Mutex<StatusLine>> = OnceLock::new();
    STATUS.get_or_init(|| Mutex::new(StatusLine::default()))
}

fn log_stdout_line(level: &str, message: &str) {
    let mut state = status_state().lock().expect("status line lock");
    let redraw = state.clear();
    let mut stdout = io::stdout();
    let _ = writeln!(stdout, "{level} {message}");
    let _ = stdout.flush();
    if redraw {
        state.draw();
    }
}

fn start_status(message: String) -> bool {
    let mut state = status_state().lock().expect("status line lock");
    let _ = state.clear();
    state.interactive = io::stdout().is_terminal();
    state.message = message;
    state.frame = 0;
    if state.interactive {
        state.active = true;
        state.draw();
    } else {
        state.active = false;
        let mut stdout = io::stdout();
        let _ = writeln!(stdout, "INFO {}", state.message);
        let _ = stdout.flush();
    }
    state.interactive
}

fn update_status(message: String) {
    let mut state = status_state().lock().expect("status line lock");
    if !state.active || !state.interactive {
        return;
    }
    state.message = message;
    state.draw();
}

fn tick_status() {
    let mut state = status_state().lock().expect("status line lock");
    if !state.active || !state.interactive {
        return;
    }
    state.frame = (state.frame + 1) % LOADING_FRAMES.len();
    state.draw();
}

fn finish_status() {
    let mut state = status_state().lock().expect("status line lock");
    let _ = state.clear();
    state.active = false;
    state.message.clear();
    state.frame = 0;
}

impl StatusLine {
    fn draw(&mut self) {
        if !self.active || !self.interactive {
            return;
        }
        let line = interactive_status_line(&self.message, self.frame);
        let width = line.chars().count();
        let padding = " ".repeat(self.width.saturating_sub(width));
        let mut stdout = io::stdout();
        let _ = write!(stdout, "\r{line}{padding}");
        let _ = stdout.flush();
        self.width = width;
    }

    fn clear(&mut self) -> bool {
        if !self.active || !self.interactive {
            return false;
        }
        let mut stdout = io::stdout();
        let _ = write!(stdout, "\r{}\r", " ".repeat(self.width));
        let _ = stdout.flush();
        self.width = 0;
        true
    }
}

fn interactive_status_line(message: &str, frame: usize) -> String {
    format!(
        "INFO {message} [{}]",
        LOADING_FRAMES[frame % LOADING_FRAMES.len()]
    )
}

#[cfg(test)]
mod tests {
    use super::interactive_status_line;

    #[test]
    fn formats_interactive_loading_frames() {
        let message = "Loading dev targets: Android app, iOS app";

        assert_eq!(
            interactive_status_line(message, 0),
            "INFO Loading dev targets: Android app, iOS app [|]"
        );
        assert_eq!(
            interactive_status_line(message, 1),
            "INFO Loading dev targets: Android app, iOS app [/]"
        );
        assert_eq!(
            interactive_status_line(message, 4),
            "INFO Loading dev targets: Android app, iOS app [|]"
        );
    }
}
