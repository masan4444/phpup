use super::Command;
use crate::release::{Release, Support};
use crate::version::Version;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ListLocal {
    version: Option<String>,
}

impl ListBase for ListLocal {}

impl Command for ListLocal {
    fn run(&self) -> anyhow::Result<()> {
        let home_dir = dirs::home_dir()
            .expect("Can't get home directory")
            .join(".phpup");
        let versions_dir = home_dir.join("versions").join("php");

        let local_versions = Self::get_local_versions(versions_dir);
        let printer = Printer::new(vec![]);

        match &self.version {
            Some(version) => {
                let version = Version::from_str(version)?;
                local_versions
                    .iter()
                    .filter(|local_version| version.contains(local_version))
                    .for_each(|&local_version| printer.print_version(local_version, None))
            }
            None => local_versions
                .iter()
                .for_each(|&local_version| printer.print_version(local_version, None)),
        };

        Ok(())
    }
}

pub trait ListBase {
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
}

pub struct Printer {
    local_versions: Vec<Version>,
    supports: BTreeMap<Version, Support>,
}

impl Printer {
    pub fn new(local_versions: Vec<Version>) -> Self {
        Self {
            local_versions,
            supports: BTreeMap::new(),
        }
    }
    pub fn print_version(&self, version: Version, support: Option<Support>) {
        let installed = self.local_versions.contains(&version);
        println!(
            "{:<3}{:<7}   {}",
            if installed { "*" } else { "" },
            version.to_string(),
            support.map_or("".to_owned(), |s| format!("({})", s.to_string())),
        );
    }
    fn print_release(&mut self, version: Version, release: &Release) {
        let minor_version =
            Version::from_numbers(version.major_version(), version.minor_version(), None);
        let support = if version.patch_version() == Some(0) {
            let support = release.calculate_support();
            Some(*self.supports.entry(minor_version).or_insert(support))
        } else {
            self.supports.get(&minor_version).cloned()
        };
        self.print_version(version, support)
    }
    pub fn print_releases(&mut self, releaes: &BTreeMap<Version, Release>) {
        for (&version, release) in releaes {
            self.print_release(version, release);
        }
    }
}
