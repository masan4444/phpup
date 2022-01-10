use super::{Command, Config};
use crate::release::{Release, Support};
use crate::version::Version;
use std::collections::BTreeMap;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ListLocal {
    version: Option<String>,
}

impl Command for ListLocal {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let local_versions = &config.local_versions;
        let empty = vec![];
        let printer = Printer::new(&empty);

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

// TODO: refactor
pub struct Printer<'a> {
    local_versions: &'a Vec<Version>,
    supports: BTreeMap<Version, Support>,
}

impl<'a> Printer<'a> {
    pub fn new(local_versions: &'a Vec<Version>) -> Self {
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
