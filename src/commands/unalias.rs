use super::{Command, Config};
use crate::decorized::Decorized;
use crate::version::alias;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Unalias {
    alias: alias::Alias,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't remove alias: {0}")]
    FailedRemoveAlias(#[from] alias::Error),
}

impl Command for Unalias {
    type Error = Error;

    fn run(&self, config: &Config) -> Result<(), Error> {
        self.alias.remove(config.aliases_dir())?;
        println!("Remove the alias {}", self.alias.decorized());
        Ok(())
    }
}
