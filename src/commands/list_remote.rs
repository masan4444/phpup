use super::Config;
use super::{list_local::Printer, Command};
use crate::release;
use crate::version::Version;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ListRemote {
    version: Option<Version>,
    #[structopt(long)]
    old: bool,
}

impl Command for ListRemote {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let local_versions = config.local_versions();
        let printer = Printer::new(&local_versions, config.current_version());

        match &self.version {
            Some(version) => {
                printer.print_versions(release::fetch_all(*version)?.keys());
            }
            None => {
                if self.old {
                    printer.print_versions(release::fetch_all(Version::from_major(3))?.keys());
                    printer.print_versions(release::fetch_all(Version::from_major(4))?.keys());
                    printer.print_versions(release::fetch_all(Version::from_major(5))?.keys());
                }
                printer.print_versions(release::fetch_all(Version::from_major(7))?.keys());
                printer.print_versions(release::fetch_all(Version::from_major(8))?.keys());
            }
        };
        Ok(())
    }
}
