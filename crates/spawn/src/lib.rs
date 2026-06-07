mod capture;
mod config;
mod control;
mod error;
mod event;
mod platform;
mod pty;
mod stdio;
mod validation;

include!("lib_core.rs");

#[cfg(test)]
mod tests {
    include!("lib_tests.rs");
}
