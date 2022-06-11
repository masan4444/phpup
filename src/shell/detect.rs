mod unix;
mod windows;

use std::str::FromStr;
use thiserror::Error;

#[cfg(unix)]
pub use self::unix::detect;
#[cfg(not(unix))]
pub use self::windows::detect;

const MAX_SEARCH_ITERATIONS: u8 = 10;

#[derive(Debug, Error)]
pub enum ParseShellError {
    #[error("Unknown shell: {0}")]
    UnknownShell(String),
}

impl FromStr for super::Shell {
    type Err = ParseShellError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bash" | "dash" => Ok(super::Bash),
            "zsh" => Ok(super::Zsh),
            "fish" => Ok(super::Fish),
            "powershell" => Ok(super::PowerShell),
            _ => Err(ParseShellError::UnknownShell(s.to_owned())),
        }
    }
}

#[derive(Debug, Error)]
pub enum ShellDetectError {
    #[error("parent process tracing count reached the limit: {MAX_SEARCH_ITERATIONS}")]
    TooManyTracing,

    #[error("reached first process PID=0 when tracing processes")]
    ReachedFirstProcess,

    #[cfg(unix)]
    #[error(transparent)]
    FailedGetProcessInfo(#[from] unix::ProcessInfoError),
}
