use super::{Command, Config};
use crate::alias::Alias;
use crate::decorized::Decorized;
use clap;
use std::fs;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Unalias {
    alias: Alias,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find alias '{0}'")]
    NotFoundAliasError(String),
}

impl Command for Unalias {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        let alias_symlink = self.alias.symlink_path(&config.aliases_dir());
        if alias_symlink.exists() {
            fs::remove_file(&alias_symlink).expect("Can't remove alias symbolic link");
        } else {
            return Err(Error::NotFoundAliasError(self.alias.to_string()))?;
        }
        println!("Remove the alias {}", self.alias.decorized());
        Ok(())
    }
}
