use crate::version::Version;
use itertools::Itertools;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub trait Command {
    fn run(&self, config: &Config) -> anyhow::Result<()>;
}

pub struct Config {
    home_dir: PathBuf,
    versions_dir: PathBuf,
    local_versions: Vec<Version>,
    multishell_path: Option<PathBuf>,
    current_version: Option<Version>,
}

impl Config {
    pub fn new() -> Self {
        let home_dir = dirs::home_dir()
            .expect("Can't get home directory")
            .join(".phpup");
        let versions_dir = home_dir.join("versions").join("php");
        let local_versions = get_local_versions(&versions_dir);
        let multishell_path = std::env::var("PHPUP_MULTISHELL_PATH")
            .map(move |path| PathBuf::from(path))
            .ok();
        let current_version = multishell_path
            .as_ref()
            .map(|symlink| {
                symlink.read_link().ok().map(|version_dir_path| {
                    version_dir_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .parse()
                        .ok()
                })
            })
            .flatten()
            .flatten();

        Self {
            home_dir,
            versions_dir,
            local_versions,
            multishell_path,
            current_version,
        }
    }
}

fn get_local_versions(versions_dir: impl AsRef<Path>) -> Vec<Version> {
    fs::read_dir(&versions_dir)
        .unwrap()
        .flat_map(|entry| entry.ok())
        .flat_map(|path| path.path().file_name().map(ToOwned::to_owned))
        .flat_map(|dir_os_str| dir_os_str.into_string())
        .flat_map(|dir_str| Version::from_str(&dir_str).ok())
        .filter(|version| {
            versions_dir
                .as_ref()
                .join(version.to_string())
                .join("bin")
                .join("php")
                .is_file()
        })
        .sorted()
        .collect()
}

mod current;
mod init;
mod install;
mod list_local;
mod list_remote;
mod r#use;

pub use current::Current;
pub use init::Init;
pub use install::Install;
pub use list_local::ListLocal;
pub use list_remote::ListRemote;
pub use r#use::Use;
