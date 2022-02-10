use super::{Command, Config, ConfigError};
use crate::decorized::Decorized;
use crate::symlink;
use crate::version::Version;
use clap;
use std::fs;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Uninstall {
    version: Version,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find installed version '{0}'")]
    NotInstalledError(Version),
    #[error(transparent)]
    NoMultiShellPathError(#[from] ConfigError),
}

impl Command for Uninstall {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        let uninstall_version = config
            .local_versions()
            .find(|local| local == &self.version)
            .ok_or(Error::NotInstalledError(self.version))?;

        if config.current_version() == Some(uninstall_version) {
            symlink::remove(&config.multishell_path()?).expect("Can't remove symlink!");
        }

        let version_dir = config.versions_dir().join(uninstall_version.to_string());
        fs::remove_dir_all(&version_dir).expect("Can't remove installed directory");
        println!(
            "{} was removed successfully from {}",
            uninstall_version.decorized_with_prefix(),
            version_dir.display().decorized()
        );

        if let Some(aliases) = config.aliases().get(&uninstall_version) {
            let aliases_dir = &config.aliases_dir();
            for alias in aliases {
                let alias_symlink = alias.symlink_path(&aliases_dir);
                fs::remove_file(&alias_symlink).expect("Can't remove alias symbolic link");
                println!("Alias {} was removed successfully", alias.decorized());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
