use phpup::commands::{self, Command};
use phpup::config::Config;
use structopt::StructOpt;

fn main() {
    let Cli { config, subcmd } = Cli::from_args();
    match subcmd {
        SubCommand::Init(cmd) => cmd.apply(&config),
        SubCommand::ListRemote(cmd) => cmd.apply(&config),
        SubCommand::Install(cmd) => cmd.apply(&config),
        SubCommand::List(cmd) => cmd.apply(&config),
        SubCommand::Use(cmd) => cmd.apply(&config),
        SubCommand::Current(cmd) => cmd.apply(&config),
        SubCommand::Uninstall(cmd) => cmd.apply(&config),
        SubCommand::Alias(cmd) => cmd.apply(&config),
        SubCommand::Unalias(cmd) => cmd.apply(&config),
    };
}

#[derive(StructOpt, Debug)]
#[structopt(name = "phpup")]
pub struct Cli {
    #[structopt(flatten)]
    pub config: Config,
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

    #[structopt(name = "alias")]
    Alias(commands::Alias),

    #[structopt(name = "unalias")]
    Unalias(commands::Unalias),
}
