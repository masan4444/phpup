use super::semantic::Version;
use derive_more::{Display, FromStr};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Display, FromStr, PartialEq, Eq, Hash)]
pub struct Alias(String);

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't resolve alias: {0}")]
    NotFoundAlias(String),
    #[error("Can't find version: '{0}'")]
    NotFoundVersion(String),
}

impl Alias {
    pub fn symlink_path(&self, aliases_dir: impl AsRef<Path>) -> PathBuf {
        aliases_dir.as_ref().join(&self.0)
    }
    // read symbolic link
    pub fn resolve(&self, aliases_dir: impl AsRef<Path>) -> Result<(PathBuf, Version), Error> {
        let alias_symlink = self.symlink_path(aliases_dir);
        let version_dir = alias_symlink
            .read_link()
            .map_err(|_| Error::NotFoundAlias(self.0.clone()))?;
        let version = version_dir
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .and_then(|s| s.parse::<Version>().ok())
            .ok_or_else(|| Error::NotFoundVersion(self.0.clone()))?;
        Ok((version_dir, version))
    }
}
