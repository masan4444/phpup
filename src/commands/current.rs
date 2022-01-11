use super::{Command, Config};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Current {}

impl Command for Current {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        if let Some(version) = config.current_version() {
            println!("{}", version.to_string());
        } else {
            eprintln!("{{N/A}}");
        }
        Ok(())
    }
}
