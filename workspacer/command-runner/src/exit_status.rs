// ---------------- [ File: command-runner/src/exit_status.rs ]
crate::ix!();

#[cfg(unix)]
pub use std::os::unix::process::ExitStatusExt;

#[cfg(windows)]
pub use std::os::windows::process::ExitStatusExt;

#[cfg(unix)]
pub fn make_exit_status(code: i32) -> std::process::ExitStatus {
    std::process::ExitStatus::from_raw(code)
}

#[cfg(windows)]
pub fn make_exit_status(code: u32) -> std::process::ExitStatus {
    std::process::ExitStatus::from_raw(code)
}
