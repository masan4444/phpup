use super::{Command, Config};
use crate::version::Local;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Current {}

#[derive(Error, Debug)]
pub enum Error {}

impl Command for Current {
    type Error = Error;

    fn run(&self, config: &Config) -> Result<(), Error> {
        match Local::current(config) {
            Some(Local::Installed(version)) => println!("{}", version),
            Some(Local::System) => println!("system"),
            None => println!("none"),
        }
        Ok(())
    }
}
