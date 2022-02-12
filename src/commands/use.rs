use super::{Command, Config, ConfigError};
use crate::decorized::{color::Color, Decorized};
use crate::symlink;
use crate::version;
use crate::version::Alias;
use crate::version::Version;
use clap;
use colored::Colorize;
use derive_more::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Use {
    #[clap(name = "version | alias", help = "semantic version or alias name")]
    version_name: Option<RequestVersion>,

    #[clap(flatten)]
    version_file: version::File,

    /// Don't output a message to stdout
    #[clap(long)]
    quiet: bool,
}

#[derive(Debug, Display)]
enum RequestVersion {
    Version(Version),
    Alias(Alias),
    System,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find installed version '{0}'")]
    NotInstalled(Version),

    #[error(transparent)]
    NoMultiShellPath(#[from] ConfigError),

    #[error(transparent)]
    NotFoundAlias(#[from] version::alias::Error),

    #[error("Can't detect a version: {0}")]
    NoVersionFromFile(#[from] version::file::Error),

    #[error("Can't find installed version '{0}', specified by '{1}'")]
    NotInstalledFromFile(Version, PathBuf),

    #[error("Can't find a system version")]
    NoSystemVersion,
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
                RequestVersion::Version(version) => config
                    .latest_local_version_included_in(version)
                    .ok_or(Error::NotInstalled(*version))?,
                RequestVersion::Alias(alias) => {
                    let (_, version) = alias.resolve(config.aliases_dir())?;
                    outln!(
                        !self.quiet,
                        "Resolve alias {} -> {}",
                        alias.decorized(),
                        version.decorized_with_prefix()
                    );
                    version
                }
                RequestVersion::System => {
                    let system_path = version::system::path().ok_or(Error::NoSystemVersion)?;
                    replace_multishell_path(system_path, config)?;

                    outln!(
                        !self.quiet,
                        "Using {} PHP",
                        "system".color(<Version as Decorized>::Color::color())
                    );
                    return Ok(());
                }
            },
            None => {
                let info = match self.version_file.get_version_info() {
                    Err(version::file::Error::NoVersionFile(_)) if self.quiet => return Ok(()),
                    other => other,
                }?;
                outln!(
                    !self.quiet,
                    "{} has been specified from {}",
                    info.version.decorized(),
                    info.filepath.display().decorized()
                );
                config
                    .latest_local_version_included_in(&info.version)
                    .ok_or(Error::NotInstalledFromFile(info.version, info.filepath))?
            }
        };

        let version_dir = config.versions_dir().join(request_version.to_string());
        replace_multishell_path(version_dir.join("bin"), config)?;

        outln!(
            !self.quiet,
            "Using {}",
            request_version.decorized_with_prefix()
        );
        Ok(())
    }
}

fn replace_multishell_path(new_path: impl AsRef<Path>, config: &Config) -> Result<(), Error> {
    let multishell_path = config.multishell_path()?;
    symlink::remove(multishell_path).expect("Can't remove symlink!");
    symlink::link(new_path, multishell_path).expect("Can't create symlink!");
    Ok(())
}

impl FromStr for RequestVersion {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "system" {
            Ok(Self::System)
        } else {
            s.parse::<Version>()
                .map(Self::Version)
                .or_else(|_| Ok(Self::Alias(s.parse().unwrap())))
        }
    }
}
