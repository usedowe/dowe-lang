mod capture;
mod config;
mod control;
mod error;
mod event;
mod platform;
mod process_identity;
mod pty;
mod stdio;
mod supervision;
mod validation;
#[cfg(windows)]
mod windows_conpty;

fn next_spawn_id() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub use supervision::{SupervisorCommand, run_spawn_supervisor, spawn_supervised};
#[cfg(windows)]
mod windows_job;

include!("lib_core.rs");

#[cfg(test)]
mod tests {
    include!("lib_tests.rs");
}
