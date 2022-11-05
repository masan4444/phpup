use super::{Command, Config};
use crate::decorized::Decorized;
use crate::version;
use crate::version::Local;
use colored::Colorize;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Alias {
    pub alias: crate::version::Alias,
    pub link_version: version::Local,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't make an alias: ")]
    FailedCreateAlias(#[from] version::alias::Error),
}

impl Command for Alias {
    type Error = Error;

    fn run(&self, config: &Config) -> Result<(), Error> {
        if !Local::local(config).any(|local| match local {
            Local::Installed(installed_version) => match self.link_version {
                Local::Installed(link_version) => link_version.includes(&installed_version),
                Local::System => false,
            },
            Local::System => self.link_version == Local::System,
        }) {
            println!(
                "{}: Version '{}' does not exist",
                "warning".yellow().bold(),
                self.link_version
            );
        }

        self.alias.link(&self.link_version, config.aliases_dir())?;

        println!(
            "Set alias {}@ -> {}",
            self.alias.decorized(),
            self.link_version.decorized()
        );
        Ok(())
    }
}
