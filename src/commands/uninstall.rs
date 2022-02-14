use super::{Command, Config, ConfigError};
use crate::decorized::Decorized;
use crate::symlink;
use crate::version;
use crate::version::Local;
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
    NotInstalled(Version),
    #[error(transparent)]
    NoMultiShellPath(#[from] ConfigError),
}

impl Command for Uninstall {
    type Error = Error;

    fn run(&self, config: &Config) -> Result<(), Error> {
        let uninstall_version = version::installed(config)
            .find(|installed_version| installed_version == &self.version)
            .ok_or(Error::NotInstalled(self.version))?;

        if Local::current(config) == Some(Local::Installed(uninstall_version)) {
            symlink::remove(&config.multishell_path()?).expect("Can't remove symlink!");
        }

        let version_dir = config.versions_dir().join(uninstall_version.to_string());
        fs::remove_dir_all(&version_dir).expect("Can't remove installed directory");
        println!(
            "{} was removed successfully from {}",
            uninstall_version.decorized_with_prefix(),
            version_dir.display().decorized()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
