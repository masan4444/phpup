use super::semantic;
use super::system;
use crate::config::Config;
use crate::decorized::{color::Color, Decorized};
use colored::Colorize;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum Local {
    Installed(semantic::Version),
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

impl FromStr for Local {
    type Err = semantic::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "system" {
            Ok(Self::System)
        } else {
            s.parse::<semantic::Version>().map(Self::Installed)
        }
    }
}

impl Local {
    pub fn as_version(&self) -> Option<semantic::Version> {
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
        let installed = super::installed(config).map(Local::Installed);
        let system = system::path().map(|_| Local::System);
        installed.chain(system.into_iter())
    }
    pub fn to_string_by(&self, installed: bool, used: bool) -> String {
        let output = format!(
            "{:<2}{:<6} {}",
            installed.then(|| "*").unwrap_or_default(),
            self,
            used.then(|| "<-").unwrap_or_default(),
        );

        if used {
            format!("{}", output.color(colored::Color::Green).bold())
        } else {
            format!(
                "{}",
                output.color(<semantic::Version as Decorized>::Color::color())
            )
        }
    }
}
