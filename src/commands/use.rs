use structopt::StructOpt;

use super::Command;

#[derive(StructOpt, Debug)]
pub struct Use {}

impl Command for Use {
    fn run(&self) {
        println!("use")
    }
}
