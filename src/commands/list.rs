use structopt::StructOpt;

use super::Command;

#[derive(StructOpt, Debug)]
pub struct List {}

impl Command for List {
    fn run(&self) {
        println!("list")
    }
}
