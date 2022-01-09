use super::Command;
use crate::version::Version;
use itertools::Itertools;
use std::str::FromStr;
use std::{fs, path::Path};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Use {
    version: Option<String>,
}

impl Command for Use {
    fn run(&self) -> anyhow::Result<()> {
        let home_dir = dirs::home_dir()
            .expect("Can't get home directory")
            .join(".phpup");
        let versions_dir = home_dir.join("versions").join("php");
        let local_versions = get_local_versions(&versions_dir);

        match &self.version {
            Some(version) => {
                let version = Version::from_str(version)?;
                if local_versions.contains(&version) {
                    let multishell_path = std::env::var("PHPUP_MULTISHELL_PATH").unwrap();
                    let multishell_path = Path::new(&multishell_path);
                    remove_symlink_dir(multishell_path).unwrap();

                    let new_original = versions_dir.join(version.to_string());
                    create_symlink_dir(new_original, multishell_path)
                        .expect("Can't create symlink!");
                } else {
                    todo!()
                }
            }
            None => todo!(),
        }
        Ok(())
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

#[cfg(unix)]
fn create_symlink_dir<P: AsRef<Path>, U: AsRef<Path>>(original: P, link: U) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)?;
    Ok(())
}

#[cfg(unix)]
fn remove_symlink_dir<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    std::fs::remove_file(path)?;
    Ok(())
}
