use crate::config::Config;
use crate::decorized::{color::Color, Decorized};
use colored::Colorize;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum Local {
    Installed(Version),
    System,
}

impl Display for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Local::Installed(version) => format!("{}", version).fmt(f),
            Local::System => "system".fmt(f),
        }
    }
}

impl Local {
    pub fn as_version(&self) -> Option<Version> {
        match self {
            Local::Installed(version) => Some(*version),
            Local::System => None,
        }
    }
    pub fn current(config: &Config) -> Option<Self> {
        config
            .multishell_path()
            .ok()
            .and_then(|symlink| symlink.read_link().ok())
            .and_then(|path| {
                (system::path().as_ref() == Some(&path.join("php")))
                    .then(|| Local::System)
                    .or_else(|| {
                        path.parent()
                            .unwrap()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .parse()
                            .ok()
                            .map(Local::Installed)
                    })
            })
    }
    pub fn local(config: &Config) -> impl Iterator<Item = Self> {
        let installed = installed(config).map(Local::Installed);
        let system = system::path().map(|_| Local::System);
        installed.chain(system.into_iter())
    }
    pub fn to_string_by(&self, installed: bool, used: bool, config: &Config) -> String {
        let aliases_str = self
            .as_version()
            .map(|version| {
                config
                    .aliases()
                    .get(&version)
                    .into_iter()
                    .flatten()
                    .map(|alias| alias.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        let output = format!(
            "{:<2}{:<6} {} {}",
            installed.then(|| "*").unwrap_or_default(),
            self,
            aliases_str.dimmed(),
            used.then(|| "<-").unwrap_or_default(),
        );

        if used {
            format!("{}", output.color(<Version as Decorized>::Color::color()))
        } else {
            output
        }
    }
}

pub fn installed(config: &Config) -> impl Iterator<Item = Version> {
    let versions_dir = config.versions_dir();
    std::fs::read_dir(&versions_dir)
        .unwrap()
        .flatten()
        .flat_map(|path| path.path().file_name().map(ToOwned::to_owned))
        .flat_map(|dir_os_str| dir_os_str.into_string())
        .flat_map(|dir_str| dir_str.parse::<Version>())
        .filter(|version| {
            versions_dir
                .join(version.to_string())
                // TODO: windows
                .join("bin")
                .join("php")
                .is_file()
        })
        .sorted()
}

pub fn installed_by<'a>(
    version: &'a Version,
    config: &Config,
) -> impl Iterator<Item = Version> + 'a {
    installed(config).filter(|v| version.includes(v))
}

pub fn latest_installed_by(version: &Version, config: &Config) -> Option<Version> {
    installed_by(version, config).max()
}

pub fn aliases(config: &Config) -> HashMap<Version, Vec<Alias>> {
    let aliases_dir = config.aliases_dir();
    let mut map: HashMap<Version, Vec<Alias>> = HashMap::new();
    std::fs::read_dir(&aliases_dir)
        .unwrap()
        .flatten()
        .flat_map(|path| path.path().file_name().map(ToOwned::to_owned))
        .flat_map(|dir_os_str| dir_os_str.into_string())
        .flat_map(|dir_str| dir_str.parse::<Alias>())
        .flat_map(|alias: Alias| {
            alias
                .resolve(&aliases_dir)
                .map(|(_, version)| (version, alias))
        })
        .for_each(|(version, alias)| {
            map.entry(version).or_default().push(alias);
        });
    map
}

pub mod alias;
pub mod file;
pub mod semantic;
pub mod system;

pub use alias::Alias;
pub use file::File;
pub use semantic::Version;
