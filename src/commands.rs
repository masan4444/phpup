use crate::version::Version;
use colored::Colorize;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub trait Command {
    type Error: std::error::Error;
    fn run(&self, config: &Config) -> Result<(), Self::Error>;
    fn apply(&self, config: &Config) {
        if let Err(e) = self.run(config) {
            eprintln!("{}: {}", "error".red().bold(), e);
            std::process::exit(1);
        }
    }
}

pub struct Config {
    base_dir: PathBuf,
    multishell_path: Option<PathBuf>,
}

impl std::default::Default for Config {
    fn default() -> Self {
        let base_dir = dirs::home_dir()
            .expect("Can't get home directory")
            .join(".phpup");
        let multishell_path = std::env::var("PHPUP_MULTISHELL_PATH")
            .map(move |path| PathBuf::from(path))
            .ok();
        Self {
            base_dir,
            multishell_path,
        }
    }
}

impl Config {
    pub fn multishell_path(&self) -> Option<&Path> {
        self.multishell_path.as_ref().map(|path| path.as_path())
    }
    pub fn versions_dir(&self) -> PathBuf {
        let versions_dir = self.base_dir.join("versions").join("php");
        fs::create_dir_all(&versions_dir).expect(&format!(
            "Can't create version dirctory: {:?}",
            versions_dir
        ));
        versions_dir
    }
    pub fn aliases_dir(&self) -> PathBuf {
        let aliases_dir = self.base_dir.join("aliases");
        fs::create_dir_all(&aliases_dir)
            .expect(&format!("Can't create alias dirctory: {:?}", aliases_dir));
        aliases_dir
    }
    pub fn current_version(&self) -> Option<Version> {
        self.multishell_path
            .as_ref()
            .and_then(|symlink| symlink.read_link().ok())
            .and_then(|version_dir_path| {
                version_dir_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .parse()
                    .ok()
            })
    }
    pub fn local_versions(&self) -> Vec<Version> {
        let versions_dir = self.versions_dir();
        fs::read_dir(&versions_dir)
            .unwrap()
            .flat_map(|entry| entry)
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
            .collect()
    }
    pub fn aliases(&self) -> HashMap<Version, Vec<crate::alias::Alias>> {
        use crate::alias::Alias;
        let aliases_dir = self.aliases_dir();
        let mut map: HashMap<Version, Vec<Alias>> = HashMap::new();
        fs::read_dir(&aliases_dir)
            .unwrap()
            .flat_map(|entry| entry)
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
        self.base_dir.clear();
        self.base_dir.push(base_dir);
        self
    }
}

mod alias;
mod current;
mod init;
mod install;
mod list_local;
mod list_remote;
mod unalias;
mod uninstall;
mod r#use;

pub use alias::Alias;
pub use current::Current;
pub use init::Init;
pub use install::Install;
pub use list_local::ListLocal;
pub use list_remote::ListRemote;
pub use r#use::Use;
pub use unalias::Unalias;
pub use uninstall::Uninstall;
