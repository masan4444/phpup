use super::{Command, Config};
use crate::decorized::Decorized;
use crate::version;
use crate::version::Local;
use crate::version::Version;
use clap;
use itertools::Itertools;
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
        let current_version = Local::current(config);

        let local_versions = match &self.version {
            Some(request_version) => version::installed_by(request_version, config)
                .map(Local::Installed)
                .collect_vec(),
            None => Local::local(config).collect_vec(),
        };

        for local_version in local_versions {
            let installed = true;
            let used = Some(&local_version) == current_version.as_ref();
            println!("{}", local_version.to_string_by(installed, used))
        }

        for (alias, linked_version) in version::alias::read_aliases_dir(config).sorted() {
            println!("{}@ -> {}", alias.decorized(), linked_version.decorized())
        }
        Ok(())
    }
}
