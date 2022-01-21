use super::{Command, Config};
use crate::{alias::Alias as AliasName, symlink, version::Version};
use clap;
use std::fs;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Alias {
    version: Version,
    alias: AliasName,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find installed version `{0}`")]
    NotInstalledError(Version),
}

impl Command for Alias {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        let local_versions = config.local_versions();
        // TODO: funcionarize
        let version = local_versions
            .iter()
            .filter(|local_version| self.version.includes(local_version))
            .max()
            .ok_or(Error::NotInstalledError(self.version))?;

        let alias_symlink = self.alias.symlink_path(&config.aliases_dir());
        if alias_symlink.exists() {
            fs::remove_file(&alias_symlink).expect("Can't remove alias symbolic link");
        }
        let version_dir = config.versions_dir().join(version.to_string());
        symlink::link(version_dir, alias_symlink).expect("Can't create symlink!");

        println!("Set `{}` as the alias to {}", self.alias, version);
        Ok(())
    }
}
