use crate::config::Config;
use itertools::Itertools;

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

pub mod alias;
pub mod file;
pub mod local;
pub mod semantic;
pub mod system;

pub use alias::Alias;
pub use file::File;
pub use local::Local;
pub use semantic::Version;
