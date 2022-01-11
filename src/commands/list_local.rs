use super::{Command, Config};
use crate::version::Version;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ListLocal {
    version: Option<Version>,
}

impl Command for ListLocal {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let local_versions = config.local_versions();
        let printer = Printer::new(&[], config.current_version());

        match &self.version {
            Some(version) => local_versions
                .iter()
                .filter(|local_version| version.contains(local_version))
                .for_each(|&local_version| printer.print_version(local_version)),
            None => local_versions
                .iter()
                .for_each(|&local_version| printer.print_version(local_version)),
        };

        Ok(())
    }
}

// TODO: refactor
pub struct Printer<'a> {
    local_versions: &'a [Version],
    current_version: Option<Version>,
}

impl<'a> Printer<'a> {
    pub fn new(local_versions: &'a [Version], current_version: Option<Version>) -> Self {
        Self {
            local_versions,
            current_version,
        }
    }
    pub fn print_version(&self, version: Version) {
        let installed = self.local_versions.contains(&version);
        let used = self.current_version == Some(version);
        println!(
            "{:<3}{:<7}",
            if used {
                "->"
            } else {
                if installed {
                    "*"
                } else {
                    ""
                }
            },
            version.to_string(),
        );
    }
    pub fn print_versions(&self, releaes: impl Iterator<Item = &'a Version>) {
        for version in releaes {
            self.print_version(*version);
        }
    }
}
