use super::{Command, Config};
use crate::clap_enum_variants;
use crate::cli::Cli;
use crate::shell::{self, Shell};
use clap::{self, CommandFactory};
use clap_complete::generate;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Completions {
    #[clap(long, value_parser = clap_enum_variants!(Shell))]
    shell: Option<Shell>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't detect using shell: {0}; You may be using unsupported shell")]
    UndetectedShell(#[from] shell::ShellDetectError),
}

impl Command for Completions {
    type Error = Error;

    fn run(&self, _: &Config) -> Result<(), Error> {
        let shell = self
            .shell
            .map_or_else(Shell::detect_shell, Ok)?
            .to_clap_shell();
        let mut app = Cli::command();
        let bin_name = app.get_name().to_string();
        let mut stdout = std::io::stdout();
        generate(shell, &mut app, bin_name, &mut stdout);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_bash() {
        let config = Config::default();
        let completions = Completions {
            shell: Some(shell::Shell::Bash),
        };
        assert!(completions.run(&config).is_ok());
    }
    #[test]
    fn for_zsh() {
        let config = Config::default();
        let completions = Completions {
            shell: Some(shell::Shell::Zsh),
        };
        assert!(completions.run(&config).is_ok());
    }
}
