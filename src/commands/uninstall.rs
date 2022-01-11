use super::{Command, Config};
use crate::version::Version;
use anyhow::anyhow;
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Uninstall {
    version: Version,
}

impl Command for Uninstall {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let local_versions = config.local_versions();
        let versions_dir = config.versions_dir();
        if local_versions.contains(&self.version) {
            let installed_dir = versions_dir.join(self.version.to_string());
            fs::remove_dir_all(&installed_dir)?;
        } else {
            return Err(anyhow!("Version `{}` isn't installed", self.version));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
