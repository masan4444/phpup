use super::{Command, Config};
// use crate::symlink;
use crate::shell::{self, Shell};
use crate::version;
use clap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Init {
    /// Spacify a shell type
    #[clap(long, possible_values(shell::available_shells()))]
    shell: Option<Shell>,

    /// Enable automatically version switching when changing directory
    #[clap(long, visible_alias = "auto")]
    auto_switch: bool,

    #[clap(flatten)]
    version_file: version::File,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't detect using shell: {0}; You may be using unsupported shell")]
    UndetectedShell(#[from] shell::ShellDetectError),
}

impl Command for Init {
    type Error = Error;
    fn run(&self, _config: &Config) -> Result<(), Error> {
        let shell = self.shell.map_or_else(Shell::detect_shell, Ok)?;
        let symlink = create_symlink();
        let mut eval_stmts = vec![
            shell.set_env("PHPUP_MULTISHELL_PATH", symlink.display()),
            shell.set_path(symlink),
        ];
        if self.auto_switch {
            eval_stmts.push(shell.auto_switch_hook(&self.version_file))
        }
        if let Some(rehash) = shell.rehash() {
            eval_stmts.push(rehash)
        }
        println!("{}", eval_stmts.join("\n"));
        Ok(())
    }
}

fn create_symlink() -> std::path::PathBuf {
    let temp_dir = std::env::temp_dir().join("phpup");
    std::fs::create_dir_all(&temp_dir).expect("Can't create tempdir!");

    // TODO: default version
    // symlink::link(&default_version_dir, &symlink_path).expect("Can't create symlink!");
    loop {
        let symlink_path = temp_dir.join(generate_symlink_path());
        if !symlink_path.exists() {
            break symlink_path;
        }
    }
}

fn generate_symlink_path() -> PathBuf {
    PathBuf::from(format!(
        "{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis(),
    ))
}

#[cfg(test)]
mod tests {}
