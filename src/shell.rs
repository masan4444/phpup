use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum Shell {
    Bash,
    Zsh,
}

pub const fn available_shells() -> &'static [&'static str] {
    &["bash", "zsh"]
}

#[derive(Debug, Error)]
pub enum ShellDetectError {
    #[error("parent process tracing count reached the limit: {MAX_ITERATIONS}")]
    TracingParentLimitError,
    #[error("reached first process PID=0 when tracing processes")]
    ReachedFirstProcessError,
    #[error(transparent)]
    ProcessInfoError(#[from] ProcessInfoError),
}

const MAX_ITERATIONS: u8 = 10;

use Shell::*;
impl Shell {
    pub fn detect_shell() -> Result<Self, ShellDetectError> {
        use ShellDetectError::*;

        let mut pid = std::process::id();
        let mut visited = 0;

        loop {
            if visited > MAX_ITERATIONS {
                return Err(TracingParentLimitError);
            }
            if pid == 0 {
                return Err(ReachedFirstProcessError);
            }
            let process_info = get_process_info(pid)?;
            let binary = process_info
                .command
                .trim_start_matches('-')
                .split('/')
                .last()
                .unwrap();
            println!("{}", binary);
            if let Ok(shell) = Self::from_str(binary) {
                return Ok(shell);
            }
            pid = process_info.parent_pid;
            visited += 1;
        }
    }
    pub fn set_path(&self, path: impl AsRef<Path>) -> String {
        match &self {
            Bash | Zsh => {
                format!("export PATH={:?}:$PATH", path.as_ref().to_str().unwrap())
            }
        }
    }
    pub fn set_env(&self, name: &str, value: &str) -> String {
        match &self {
            Bash | Zsh => {
                format!("export {}={:?}", name, value)
            }
        }
    }
    pub fn rehash(&self) -> Option<String> {
        match &self {
            Bash => None,
            Zsh => Some("rehash".to_string()),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseShellError {
    #[error("Unknown shell: {0}")]
    UnsupportedShell(String),
}
impl FromStr for Shell {
    type Err = ParseShellError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bash" | "dash" => Ok(Bash),
            "zsh" => Ok(Zsh),
            _ => Err(ParseShellError::UnsupportedShell(s.to_owned())),
        }
    }
}

struct ProcessInfo {
    parent_pid: u32,
    command: String,
}
#[derive(Debug, Error)]
pub enum ProcessInfoError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("failed to exec 'ps' command")]
    FaildExecError,
    #[error("can't parse 'ps' command output: {0}")]
    ParseError(String),
}
#[cfg(unix)]
fn get_process_info(pid: u32) -> Result<ProcessInfo, ProcessInfoError> {
    use std::io::{BufRead, BufReader};
    use std::process::Command;
    use ProcessInfoError::*;

    let mut child = Command::new("ps")
        .arg("-o")
        .arg("ppid=,comm=")
        .arg(pid.to_string())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    match child.wait() {
        Ok(status) if status.success() => {}
        _ => return Err(FaildExecError),
    }

    let mut line = String::new();
    BufReader::new(child.stdout.unwrap()).read_line(&mut line)?;

    let mut parts = line.trim().split_whitespace();
    let ppid = parts.next().ok_or_else(|| ParseError(line.to_string()))?;
    let command = parts.next().ok_or_else(|| ParseError(line.to_string()))?;

    Ok(ProcessInfo {
        parent_pid: ppid
            .parse()
            .or_else(|_| Err(ParseError(line.to_string())))?,
        command: command.into(),
    })
}
