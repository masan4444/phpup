use super::{Command, Config};
use clap;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Current {}

#[derive(Error, Debug)]
pub enum Error {}

impl Command for Current {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        if let Some(version) = config.current_version() {
            println!("{}", version);
        } else {
            println!("none");
        }
        Ok(())
    }
}
