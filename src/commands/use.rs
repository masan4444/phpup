use super::{Command, Config, ConfigError};
use crate::decorized::Decorized;
use crate::symlink;
use crate::version;
use crate::version::Alias;
use crate::version::Local;
use crate::version::Version;
use clap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Use {
    #[clap(
        name = "version | alias | system",
        help = "installed version or alias name or system"
    )]
    request_version: Option<RequestVersion>,

    #[clap(flatten)]
    version_file: version::File,

    /// Don't output a message to stdout
    #[clap(long)]
    quiet: bool,
}

#[derive(Debug)]
enum RequestVersion {
    Installed(Version),
    Alias(Alias),
    System,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find installed version '{version}' {source}")]
    NotInstalled { version: Version, source: Source },

    #[error(transparent)]
    NoMultiShellPath(#[from] ConfigError),

    #[error(transparent)]
    NotFoundAlias(#[from] version::alias::Error),

    #[error("Can't detect a version: {0}")]
    NoVersionFromFile(#[from] version::file::Error),

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
        let (use_version, source) = match &self.request_version {
            Some(request_version) => match request_version {
                RequestVersion::Installed(version) => {
                    (Local::Installed(*version), Source::Installed(*version))
                }
                RequestVersion::Alias(alias) => {
                    let version = alias.resolve(config.aliases_dir())?;
                    outln!(
                        !self.quiet,
                        "Resolve alias {}@ -> {}",
                        alias.decorized(),
                        version.decorized()
                    );
                    (version, Source::Alias(alias.clone()))
                }
                RequestVersion::System => (Local::System, Source::System),
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
                (info.version, Source::File(info.filepath))
            }
        };

        match use_version {
            Local::Installed(version) => {
                let use_version = version::latest_installed_by(&version, config)
                    .ok_or(Error::NotInstalled { version, source })?;
                let version_dir = config.versions_dir().join(use_version.to_string());
                replace_multishell_path(version_dir.join("bin"), config)?;

                outln!(!self.quiet, "Using {}", use_version.decorized_with_prefix());
            }
            Local::System => {
                let system_path = version::system::path().ok_or(Error::NoSystemVersion)?;
                replace_multishell_path(&system_path, config)?;

                outln!(
                    !self.quiet,
                    "Using {} -> {}",
                    Local::System.decorized_with_prefix(),
                    system_path.display().decorized()
                );
            }
        }

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
                .map(Self::Installed)
                .or_else(|_| Ok(Self::Alias(s.parse().unwrap())))
        }
    }
}

#[derive(Error, Debug)]
pub enum Source {
    Installed(Version),
    Alias(Alias),
    File(PathBuf),
    System,
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Installed(_) => String::new().fmt(f),
            Source::Alias(alias) => format!("specified by alias '{}'", alias).fmt(f),
            Source::File(path) => format!("specified by version-file '{}'", path.display()).fmt(f),
            Source::System => String::new().fmt(f),
        }
    }
}
