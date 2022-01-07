use phpup::commands::{self, Command};
use structopt::StructOpt;

fn main() {
    let Cli { subcmd } = Cli::from_args();
    match subcmd {
        SubCommand::ListRemote(cmd) => cmd.run(),
        SubCommand::Install(cmd) => cmd.run(),
        SubCommand::List(cmd) => cmd.run(),
        SubCommand::Use(cmd) => cmd.run(),
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "phpup")]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    #[structopt(name = "list-remote", visible_aliases = &["ls-remote"])]
    ListRemote(commands::ListRemote),

    #[structopt(name = "install")]
    Install(commands::Install),

    #[structopt(name = "ls")]
    List(commands::List),

    #[structopt(name = "ls")]
    Use(commands::Use),
}
