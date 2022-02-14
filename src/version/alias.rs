use super::Local;
use crate::config::Config;
use derive_more::{Display, FromStr};
use std::cmp::Ordering;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Display, FromStr, Clone, PartialEq, Eq)]
pub struct Alias(String);

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't find an alias '{0}'")]
    NotFoundAlias(String),

    #[error("Can't parse version: '{0}'")]
    FailedParse(#[from] super::semantic::ParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Default for Alias {
    fn default() -> Self {
        Self("default".to_owned())
    }
}

impl Alias {
    fn filepath(&self, aliases_dir: impl AsRef<Path>) -> PathBuf {
        aliases_dir.as_ref().join(&self.0)
    }
    fn is_default(&self) -> bool {
        self == &Self::default()
    }

    pub fn link(&self, version: &Local, aliases_dir: impl AsRef<Path>) -> Result<(), Error> {
        let mut file = fs::File::create(self.filepath(aliases_dir))?;
        file.write_all(version.to_string().as_bytes())
            .map_err(Into::into)
    }
    pub fn resolve(&self, aliases_dir: impl AsRef<Path>) -> Result<Local, Error> {
        let filepath = self.filepath(aliases_dir);
        if filepath.exists() {
            fs::read_to_string(&filepath)?.parse().map_err(Into::into)
        } else {
            Err(Error::NotFoundAlias(self.0.clone()))
        }
    }
    pub fn remove(&self, aliases_dir: impl AsRef<Path>) -> Result<(), Error> {
        let filepath = self.filepath(aliases_dir);
        if filepath.exists() {
            fs::remove_file(&filepath).map_err(Into::into)
        } else {
            Err(Error::NotFoundAlias(self.0.clone()))
        }
    }
}

fn read_alias(filepath: impl AsRef<Path>) -> Result<(Alias, Local), Error> {
    let alias = Alias(
        filepath
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned(),
    );
    let version = fs::read_to_string(&filepath)?.parse()?;
    Ok((alias, version))
}

pub fn read_aliases_dir(config: &Config) -> impl Iterator<Item = (Alias, Local)> {
    let aliases_dir = config.aliases_dir();
    std::fs::read_dir(&aliases_dir)
        .unwrap()
        .flatten()
        .map(|entry| read_alias(entry.path()))
        .flatten()
}

impl Ord for Alias {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_default() {
            if other.is_default() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        } else if other.is_default() {
            Ordering::Greater
        } else {
            self.0.cmp(&other.0)
        }
    }
}
impl PartialOrd for Alias {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
