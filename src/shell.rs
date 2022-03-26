use crate::version;
use indoc::formatdoc;
use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

pub const fn available_shells() -> &'static [&'static str] {
    &["bash", "zsh", "fish"]
}

#[derive(Debug, Error)]
pub enum ShellDetectError {
    #[error("parent process tracing count reached the limit: {MAX_SEARCH_ITERATIONS}")]
    TooManyTracing,

    #[error("reached first process PID=0 when tracing processes")]
    ReachedFirstProcess,

    #[error(transparent)]
    FailedGetProcessInfo(#[from] ProcessInfoError),
}

const MAX_SEARCH_ITERATIONS: u8 = 10;

use Shell::*;

impl Shell {
    pub fn detect_shell() -> Result<Self, ShellDetectError> {
        let mut pid = std::process::id();
        let mut visited = 0;

        loop {
            if visited > MAX_SEARCH_ITERATIONS {
                return Err(ShellDetectError::TooManyTracing);
            }
            if pid == 0 {
                return Err(ShellDetectError::ReachedFirstProcess);
            }
            let process_info = get_process_info(pid)?;
            let binary = process_info
                .command
                .trim_start_matches('-')
                .split('/')
                .last()
                .unwrap();
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
                format!("export PATH={}:$PATH", path.as_ref().display())
            }
            Fish => format!("set -gx PATH {} $PATH;", path.as_ref().display()),
        }
    }
    pub fn set_env(&self, name: impl Display, value: impl Display) -> String {
        match &self {
            Bash | Zsh => {
                format!("export {}={}", name, value)
            }
            Fish => format!("set -gx {} {};", name, value),
        }
    }
    pub fn auto_switch_hook(&self, version_file: &version::File) -> String {
        let version_file_name = version_file.filename().display();
        let is_recursive_version_file = if version_file.is_recursive() {
            "--recursive-version-file"
        } else {
            ""
        };
        let phpup_use = format!(
            "phpup use --quiet --version-file-name {} {}",
            version_file_name, is_recursive_version_file
        );

        match &self {
            Bash => {
                formatdoc! {
                    r#"
                    __phpup_use() {{
                        {phpup_use}
                    }}
                    __phpupcd() {{
                        \cd "$@" || return $?
                        __phpup_use
                    }}
                    alias cd=__phpupcd
                    __phpup_use"#,
                    phpup_use = phpup_use
                }
            }
            Zsh => {
                formatdoc! {
                    r#"
                    autoload -U add-zsh-hook
                    _phpup_autoload_hook () {{
                        {phpup_use}
                    }}
                    add-zsh-hook chpwd _phpup_autoload_hook \
                        && _phpup_autoload_hook"#,
                    phpup_use = phpup_use
                }
            }
            Fish => {
                formatdoc!(
                    r#"
                    function _phpup_autoload_hook --on-variable PWD --description 'Change PHP version on directory change'
                        status --is-command-substitution; and return
                        {phpup_use}
                    end
                    _phpup_autoload_hook"#,
                    phpup_use = phpup_use
                )
            }
        }
    }
    pub fn rehash(&self) -> Option<String> {
        match &self {
            Bash | Fish => None,
            Zsh => Some("rehash".to_string()),
        }
    }
    pub fn to_clap_shell(&self) -> clap_complete::Shell {
        match &self {
            Bash => clap_complete::Shell::Bash,
            Zsh => clap_complete::Shell::Zsh,
            Fish => clap_complete::Shell::Fish,
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseShellError {
    #[error("Unknown shell: {0}")]
    UnknownShell(String),
}
impl FromStr for Shell {
    type Err = ParseShellError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bash" | "dash" => Ok(Bash),
            "zsh" => Ok(Zsh),
            "fish" => Ok(Fish),
            _ => Err(ParseShellError::UnknownShell(s.to_owned())),
        }
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
#[cfg(unix)]
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
