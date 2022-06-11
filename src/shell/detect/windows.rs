#![cfg(not(unix))]

use std::ffi::OsStr;
use sysinfo::{ProcessExt, System, SystemExt};

pub fn detect() -> Result<super::super::Shell, super::ShellDetectError> {
    let mut system = System::new();
    let mut current_pid = sysinfo::get_current_pid().ok();
    let mut visited = 0;

    while let Some(pid) = current_pid {
        if visited > super::MAX_SEARCH_ITERATIONS {
            return Err(super::ShellDetectError::TooManyTracing);
        }
        system.refresh_process(pid);
        if let Some(process) = system.process(pid) {
            current_pid = process.parent();
            let process_name = process
                .exe()
                .file_stem()
                .and_then(OsStr::to_str)
                .map(str::to_lowercase);
            if let Some(shell) = process_name.as_deref().and_then(|x| x.parse().ok())
            {
                return Ok(shell);
            }
        } else {
            current_pid = None;
        }
        visited += 1;
    }

    Err(super::ShellDetectError::ReachedFirstProcess)
}
