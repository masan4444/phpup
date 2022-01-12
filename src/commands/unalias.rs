use super::{Command, Config};
use crate::alias::Alias;
use std::fs;
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub struct Unalias {
    alias: Alias,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find alias `{0}`")]
    NotFoundAliasError(String),
}

impl Command for Unalias {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let alias_symlink = self.alias.symlink_path(&config.aliases_dir());
        if alias_symlink.exists() {
            fs::remove_file(&alias_symlink).expect("Can't remove alias symbolic link");
        } else {
            return Err(Error::NotFoundAliasError(self.alias.to_string()))?;
        }
        println!("Remove the alias `{}`", self.alias);
        Ok(())
    }
}
