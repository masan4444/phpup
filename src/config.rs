use crate::version::Version;
use clap;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct Config {
    /// Specify a custom PHP-UP directory
    #[clap(long = "phpup-dir", env = "PHPUP_DIR")]
    base_dir: Option<PathBuf>,

    /// Specify a custom symbolic link used for version switching
    #[clap(long, env = "PHPUP_MULTISHELL_PATH", hide = true)]
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
    pub fn current_version(&self) -> Option<Version> {
        self.multishell_path
            .as_ref()
            .and_then(|symlink| symlink.read_link().ok())
            .and_then(|bin_path| {
                bin_path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .parse()
                    .ok()
            })
    }
    pub fn local_versions(&self) -> impl Iterator<Item = Version> {
        let versions_dir = self.versions_dir();
        fs::read_dir(&versions_dir)
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
    pub fn local_versions_included_in<'a>(
        &self,
        version: &'a Version,
    ) -> impl Iterator<Item = Version> + 'a {
        self.local_versions()
            .filter(|local_version| version.includes(local_version))
    }
    pub fn latest_local_version_included_in(&self, version: &Version) -> Option<Version> {
        self.local_versions_included_in(version).max()
    }
    pub fn aliases(&self) -> HashMap<Version, Vec<crate::alias::Alias>> {
        use crate::alias::Alias;
        let aliases_dir = self.aliases_dir();
        let mut map: HashMap<Version, Vec<Alias>> = HashMap::new();
        fs::read_dir(&aliases_dir)
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

    #[cfg(test)]
    pub fn with_base_dir(mut self, base_dir: impl AsRef<std::path::Path>) -> Self {
        self.base_dir = Some(PathBuf::from(base_dir.as_ref()));
        self
    }
}
