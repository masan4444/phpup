#![cfg(unix)]

use thiserror::Error;

pub fn detect() -> Result<super::super::Shell, super::ShellDetectError> {
    let mut pid = std::process::id();
    let mut visited = 0;

    loop {
        if visited > super::MAX_SEARCH_ITERATIONS {
            return Err(super::ShellDetectError::TooManyTracing);
        }
        if pid == 0 {
            return Err(super::ShellDetectError::ReachedFirstProcess);
        }
        let process_info = get_process_info(pid)?;
        let process_name = process_info
            .command
            .trim_start_matches('-')
            .split('/')
            .last()
            .unwrap();
        if let Ok(shell) = process_name.parse() {
            return Ok(shell);
        }
        pid = process_info.parent_pid;
        visited += 1;
    }
}

struct ProcessInfo {
    parent_pid: u32,
    command: String,
}
#[derive(Debug, Error)]
pub enum ProcessInfoError {
    #[error("Can't execute `{command}` because {source}")]
    FailedExecute {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to exec '{0}' command")]
    ExitFailed(String),

    #[error("can't parse 'ps' command output: {0}")]
    Parse(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn get_process_info(pid: u32) -> Result<ProcessInfo, ProcessInfoError> {
    use std::io::{BufRead, BufReader};
    use std::process::Command;

    let ps = [
        "ps".to_owned(),
        "-o".to_owned(),
        "ppid=,comm=".to_owned(),
        pid.to_string(),
    ];

    let mut child = Command::new(&ps[0])
        .args(&ps[1..])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|source| ProcessInfoError::FailedExecute {
            command: ps.join(" "),
            source,
        })?;

    match child.wait() {
        Ok(status) if status.success() => {}
        _ => return Err(ProcessInfoError::ExitFailed(ps.join(" "))),
    }

    let mut line = String::new();
    BufReader::new(child.stdout.unwrap()).read_line(&mut line)?;

    let mut parts = line.trim().split_whitespace();
    let ppid = parts
        .next()
        .ok_or_else(|| ProcessInfoError::Parse(line.to_string()))?;
    let command = parts
        .next()
        .ok_or_else(|| ProcessInfoError::Parse(line.to_string()))?;

    Ok(ProcessInfo {
        parent_pid: ppid
            .parse()
            .map_err(|_| ProcessInfoError::Parse(line.to_string()))?,
        command: command.into(),
    })
}
