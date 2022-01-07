use structopt::StructOpt;

use super::Command;

#[derive(StructOpt, Debug)]
pub struct Install {
    version: String,
}

impl Command for Install {
    fn run(&self) {
        println!("install {}", self.version)
    }
}
