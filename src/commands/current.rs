use super::{Command, Config};
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub struct Current {}

#[derive(Error, Debug)]
pub enum Error {}

impl Command for Current {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        if let Some(version) = config.current_version() {
            println!("{}", version.to_string());
        } else {
            println!("none");
        }
        Ok(())
    }
}
