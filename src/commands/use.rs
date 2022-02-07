use super::{Command, Config, ConfigError};
use crate::alias::{self, Alias};
use crate::symlink;
use crate::version::Version;
use crate::version_file::{self, VersionFile};
use clap;
use colored::Colorize;
use derive_more::Display;
use std::str::FromStr;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Use {
    #[clap(name = "version | alias", help = "semantic version or alias name")]
    version_name: Option<VersionName>,

    #[clap(flatten)]
    version_file: VersionFile,

    /// Don't output a message to stdout
    #[clap(long)]
    quiet: bool,
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
    NotFoundAliasError(#[from] alias::Error),

    #[error("Can't detect a version: {0}")]
    NoVersionFromFileError(#[from] version_file::Error),
}

macro_rules! outln {
    ($disp:expr, $($arg:tt)*) => {
        if $disp {
            println!($($arg)*);
        };
    };
}

impl Command for Use {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        let request_version = match &self.version_name {
            Some(version_name) => match version_name {
                VersionName::Version(version) => config
                    .latest_local_version_included_in(version)
                    .ok_or(Error::NotInstalledError(*version))?,
                VersionName::Alias(alias) => {
                    let (_, version) = alias.resolve(config.aliases_dir())?;
                    outln!(
                        !self.quiet,
                        "Resolve alias: {} -> {}",
                        alias,
                        version.to_string().cyan()
                    );
                    version
                }
            },
            None => {
                let (version, version_file_path) = match self.version_file.get_version() {
                    Err(version_file::Error::NoVersionFileError(_)) if self.quiet => return Ok(()),
                    other => other,
                }?;
                outln!(
                    !self.quiet,
                    "Detected {} from {:?}",
                    version.to_string().cyan(),
                    version_file_path
                );
                config
                    .latest_local_version_included_in(&version)
                    .ok_or(Error::NotInstalledError(version))?
            }
        };

        let multishell_path = config.multishell_path()?;
        let is_used_yet = multishell_path.exists();
        let version_dir = config.versions_dir().join(request_version.to_string());

        symlink::remove(multishell_path).expect("Can't remove symlink!");
        symlink::link(version_dir, multishell_path).expect("Can't create symlink!");

        outln!(
            !self.quiet,
            "Using PHP {}",
            request_version.to_string().cyan()
        );
        if !is_used_yet {
            outln!(
                !self.quiet,
                "{}: Need to type `rehash` in this shell if you are using zsh (only first time)",
                "warning".yellow().bold()
            );
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
