use super::{Command, Config};
use crate::decorized::Decorized;
use crate::symlink;
use crate::version::Version;
use clap;
use std::fs;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Alias {
    version: Version,
    alias: crate::alias::Alias,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find installed version '{0}'")]
    NotInstalled(Version),
}

impl Command for Alias {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        let version = config
            .latest_local_version_included_in(&self.version)
            .ok_or(Error::NotInstalled(self.version))?;

        let alias_symlink = self.alias.symlink_path(&config.aliases_dir());
        if alias_symlink.exists() {
            fs::remove_file(&alias_symlink).expect("Can't remove alias symbolic link");
        }
        let version_dir = config.versions_dir().join(version.to_string());
        symlink::link(version_dir.join("bin"), alias_symlink).expect("Can't create symlink!");

        println!(
            "Set alias {} -> {}",
            self.alias.decorized(),
            version.decorized_with_prefix()
        );
        Ok(())
    }
}
