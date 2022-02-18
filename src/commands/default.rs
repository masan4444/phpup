use super::{Command, Config};
use crate::version;
use clap;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Default {
    link_version: Option<version::Local>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FailedCreateAlias(#[from] super::alias::Error),
}

impl Command for Default {
    type Error = Error;

    fn run(&self, config: &Config) -> Result<(), Error> {
        let alias = crate::version::Alias::default();
        if let Some(link_version) = self.link_version {
            let cmd = super::Alias {
                alias,
                link_version,
            };
            cmd.run(config).map_err(Into::into)
        } else {
            match alias.resolve(config.aliases_dir()) {
                Ok(version) => println!("{}", version),
                Err(_) => println!("none"),
            }
            Ok(())
        }
    }
}
