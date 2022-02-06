use super::{Command, Config, ConfigError};
use crate::alias::Alias;
use crate::symlink;
use crate::version::Version;
use clap;
use colored::Colorize;
use derive_more::Display;
use std::str::FromStr;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Use {
    #[clap(name = "version | alias", help = "semantic version or alias name")]
    version_name: Option<VersionName>,
}

#[derive(Debug, Display)]
enum VersionName {
    Version(Version),
    Alias(Alias),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find installed version `{0}`")]
    NotInstalledError(Version),
    #[error(transparent)]
    NoMultiShellPathError(#[from] ConfigError),
    #[error(transparent)]
    NotFoundAliasError(#[from] crate::alias::Error),
}

impl Command for Use {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        let local_versions = config.local_versions();

        match &self.version_name {
            Some(version_name) => {
                let (version_dir, version) = match version_name {
                    VersionName::Version(version) => {
                        // find latest version installed
                        let version = local_versions
                            .iter()
                            .filter(|local_version| version.includes(local_version))
                            .max()
                            .ok_or(Error::NotInstalledError(*version))?;
                        (config.versions_dir().join(version.to_string()), *version)
                    }
                    VersionName::Alias(alias) => {
                        let (version_dir, version) = alias.resolve(config.aliases_dir())?;
                        println!("Resolve alias: `{}` -> {:?}", alias, version_dir);
                        (version_dir, version)
                    }
                };

                let multishell_path = config.multishell_path()?;
                let is_used_yet = multishell_path.exists();
                if is_used_yet {
                    symlink::remove(multishell_path).expect("Can't remove symlink!");
                }
                symlink::link(version_dir, multishell_path).expect("Can't create symlink!");
                println!("Using {}", version);
                if !is_used_yet {
                    println!(
                        "{}: Need to type `rehash` in this shell if you are using zsh (only first time)",
                        "warning".yellow().bold()
                    );
                }
            }
            None => todo!("use .php-version"),
        }
        Ok(())
    }
}

impl FromStr for VersionName {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<Version>()
            .map_or(Self::Alias(s.parse().unwrap()), |v| Self::Version(v)))
    }
}
