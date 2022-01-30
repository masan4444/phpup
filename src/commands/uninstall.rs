use super::{Command, Config, ConfigError};
use crate::{symlink, version::Version};
use clap;
use colored::Colorize;
use std::fs;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Uninstall {
    version: Version,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find installed version `{0}`")]
    NotInstalledError(Version),
    #[error(transparent)]
    NoMultiShellPathError(#[from] ConfigError),
}

impl Command for Uninstall {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        let local_versions = config.local_versions();
        let versions_dir = config.versions_dir();
        if local_versions.contains(&self.version) {
            let version = self.version;

            if config.current_version() == Some(version) {
                symlink::remove(&config.multishell_path()?).expect("Can't remove symlink!");
            }

            let version_dir = versions_dir.join(version.to_string());
            fs::remove_dir_all(&version_dir).expect("Can't remove installed directory");
            println!(
                "Version {} was removed successfully from {:?}",
                version.to_string().cyan(),
                version_dir
            );

            if let Some(aliases) = config.aliases().get(&version) {
                let aliases_dir = &config.aliases_dir();
                for alias in aliases {
                    let alias_symlink = alias.symlink_path(&aliases_dir);
                    fs::remove_file(&alias_symlink).expect("Can't remove alias symbolic link");
                    println!(
                        "Alias {} was removed successfully",
                        alias.to_string().cyan()
                    );
                }
            }
            Ok(())
        } else {
            Err(Error::NotInstalledError(self.version))?
        }
    }
}

#[cfg(test)]
mod tests {}
