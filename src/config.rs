use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct Config {
    /// Specify a custom PHP-UP directory
    #[arg(long = "phpup-dir", env = "PHPUP_DIR")]
    base_dir: Option<PathBuf>,

    /// Specify a custom symbolic link used for version switching
    #[arg(long, env = "PHPUP_MULTISHELL_PATH", hide = true)]
    multishell_path: Option<PathBuf>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not yet initialized; Need to run `eval \"$(phpup init)\"`")]
    NoMultiShellPath,
}

impl Config {
    pub fn multishell_path(&self) -> Result<&Path, Error> {
        self.multishell_path
            .as_deref()
            .ok_or(Error::NoMultiShellPath)
    }
    pub fn base_dir(&self) -> PathBuf {
        if let Some(base_dir) = self.base_dir.as_ref() {
            base_dir.clone()
        } else {
            dirs::home_dir()
                .expect("Can't get home directory")
                .join(".phpup")
        }
    }
    pub fn versions_dir(&self) -> PathBuf {
        let versions_dir = self.base_dir().join("versions").join("php");
        fs::create_dir_all(&versions_dir)
            .unwrap_or_else(|_| panic!("Can't create version dirctory: {:?}", versions_dir));
        versions_dir
    }
    pub fn aliases_dir(&self) -> PathBuf {
        let aliases_dir = self.base_dir().join("aliases");
        fs::create_dir_all(&aliases_dir)
            .unwrap_or_else(|_| panic!("Can't create alias dirctory: {:?}", aliases_dir));
        aliases_dir
    }

    #[cfg(test)]
    pub fn with_base_dir(mut self, base_dir: impl AsRef<std::path::Path>) -> Self {
        self.base_dir = Some(PathBuf::from(base_dir.as_ref()));
        self
    }
}
