use super::{Command, Config};
use crate::{alias::Alias, version::Version};
use clap;
use colored::Colorize;
use itertools::Itertools;
use std::collections::HashMap;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct ListLocal {
    version: Option<Version>,
}

#[derive(Error, Debug)]
pub enum Error {}

impl Command for ListLocal {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        let local_versions = config.local_versions().collect_vec();
        let aliases = config.aliases();
        let printer = Printer::new(&local_versions, config.current_version(), &aliases);

        match &self.version {
            Some(version) => printer.print_versions(
                local_versions
                    .iter()
                    .filter(|local_version| version.includes(local_version)),
            ),
            None => printer.print_versions(local_versions.iter()),
        };

        Ok(())
    }
}

// TODO: refactor
pub struct Printer<'a> {
    local_versions: &'a [Version],
    current_version: Option<Version>,
    aliases: &'a HashMap<Version, Vec<Alias>>,
}

impl<'a> Printer<'a> {
    pub fn new(
        local_versions: &'a [Version],
        current_version: Option<Version>,
        aliases: &'a HashMap<Version, Vec<Alias>>,
    ) -> Self {
        Self {
            local_versions,
            current_version,
            aliases,
        }
    }
    pub fn print_version(&self, version: Version) {
        let installed = self.local_versions.contains(&version);
        let used = self.current_version == Some(version);
        let aliases_str = self
            .aliases
            .get(&version)
            .iter()
            .flat_map(|aliases| aliases.iter())
            .map(|alias| alias.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let output = format!(
            "{:<2}{:<6} {}",
            if installed { "*" } else { "" },
            version,
            aliases_str.dimmed(),
        );

        if used {
            println!("{}", output.cyan())
        } else {
            println!("{}", output)
        }
    }
    pub fn print_versions(&self, releaes: impl Iterator<Item = &'a Version>) {
        for version in releaes {
            self.print_version(*version);
        }
    }
}
