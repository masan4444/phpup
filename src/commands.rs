use crate::version::Version;
use itertools::Itertools;
use std::fs;
use std::path::PathBuf;

pub trait Command {
    fn run(&self, config: &Config) -> anyhow::Result<()>;
}

pub struct Config {
    base_dir: PathBuf,
    multishell_path: Option<PathBuf>,
}

impl Default for Config {
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
    pub fn versions_dir(&self) -> PathBuf {
        let base_dir = self.base_dir.join("versions").join("php");
        fs::create_dir_all(&base_dir)
            .expect(&format!("Can't create base dirctory: {:?}", base_dir));
        base_dir
    }
    pub fn current_version(&self) -> Option<Version> {
        self.multishell_path
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
            .flatten()
    }
    pub fn local_versions(&self) -> Vec<Version> {
        let versions_dir = self.versions_dir();
        fs::read_dir(&versions_dir)
            .unwrap()
            .flat_map(|entry| entry.ok())
            .flat_map(|path| path.path().file_name().map(ToOwned::to_owned))
            .flat_map(|dir_os_str| dir_os_str.into_string())
            .flat_map(|dir_str| dir_str.parse::<Version>().ok())
            .filter(|version| {
                versions_dir
                    .join(version.to_string())
                    .join("bin")
                    .join("php")
                    .is_file()
            })
            .sorted()
            .collect()
    }

    #[cfg(test)]
    pub fn with_base_dir(mut self, base_dir: impl AsRef<std::path::Path>) -> Self {
        self.base_dir.clear();
        self.base_dir.push(base_dir);
        self
    }
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
