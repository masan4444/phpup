use colored::Colorize;
use phpup::commands::{self, Command, Config};
use structopt::StructOpt;

fn main() {
    let Cli { subcmd } = Cli::from_args();
    let config = Config::default();
    let result = match subcmd {
        SubCommand::Init(cmd) => cmd.run(&config),
        SubCommand::ListRemote(cmd) => cmd.run(&config),
        SubCommand::Install(cmd) => cmd.run(&config),
        SubCommand::List(cmd) => cmd.run(&config),
        SubCommand::Use(cmd) => cmd.run(&config),
        SubCommand::Current(cmd) => cmd.run(&config),
        SubCommand::Uninstall(cmd) => cmd.run(&config),
    };
    if let Err(e) = result {
        eprintln!("{}: {}", "error".red().bold(), e);
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

    #[structopt(name = "current")]
    Current(commands::Current),

    #[structopt(name = "uninstall")]
    Uninstall(commands::Uninstall),
}
