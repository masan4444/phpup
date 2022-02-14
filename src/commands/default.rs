use super::{Command, Config};
use crate::version;
use clap;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Default {
    link_version: version::Local,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FailedCreateAlias(#[from] super::alias::Error),
}

impl Command for Default {
    type Error = Error;

    fn run(&self, config: &Config) -> Result<(), Error> {
        let cmd = super::Alias {
            alias: crate::version::Alias::default(),
            link_version: self.link_version,
        };
        cmd.run(config).map_err(Into::into)
    }
}
