use crate::version::Version;
use derive_more::{Display, FromStr};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Display, FromStr, PartialEq, Eq, Hash)]
pub struct Alias(String);

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't resolve alias: {0}")]
    NotFoundAliasError(String),
    #[error("Can't find version: {0}`")]
    NotFoundVersionError(String),
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
            .or(Err(Error::NotFoundAliasError(self.0.clone())))?;
        let version = version_dir
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .and_then(|s| s.parse::<Version>().ok())
            .ok_or(Error::NotFoundVersionError(self.0.clone()))?;
        Ok((version_dir, version))
    }
}
