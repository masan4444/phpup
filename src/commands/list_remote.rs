use super::Config;
use super::{list_local::Printer, Command};
use crate::release;
use crate::version::Version;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ListRemote {
    version: Option<Version>,
}

impl Command for ListRemote {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let local_versions = &config.local_versions;
        let mut printer = Printer::new(local_versions, config.current_version);

        match &self.version {
            Some(version) => {
                if version.patch_version().is_some() {
                    let oldest_patch_release = release::fetch_oldest_patch(*version)?;
                    let support = oldest_patch_release.calculate_support();
                    printer.print_version(*version, Some(support));
                } else {
                    printer.print_releases(&release::fetch_all(*version)?);
                }
            }
            None => {
                // Self::print_releases(&release::fetch_all_releases(Version::from_major(5))?);
                printer.print_releases(&release::fetch_all(Version::from_major(7))?);
                printer.print_releases(&release::fetch_all(Version::from_major(8))?);
            }
        };
        Ok(())
    }
}
