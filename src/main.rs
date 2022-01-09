use phpup::commands::{self, Command};
use structopt::StructOpt;

fn main() {
    let Cli { subcmd } = Cli::from_args();
    let result = match subcmd {
        SubCommand::Init(cmd) => cmd.run(),
        SubCommand::ListRemote(cmd) => cmd.run(),
        SubCommand::Install(cmd) => cmd.run(),
        SubCommand::List(cmd) => cmd.run(),
        SubCommand::Use(cmd) => cmd.run(),
    };
    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
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
    #[structopt(name = "init")]
    Init(commands::Init),

    #[structopt(name = "list-remote", visible_aliases = &["ls-remote"])]
    ListRemote(commands::ListRemote),

    #[structopt(name = "install")]
    Install(commands::Install),

    #[structopt(name = "list", visible_aliases = &["ls"])]
    List(commands::ListLocal),

    #[structopt(name = "use")]
    Use(commands::Use),
}
