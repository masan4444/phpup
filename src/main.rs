use clap::Parser;
use phpup::commands::{self, Command};
use phpup::config::Config;

fn main() {
    let Cli { config, subcmd } = Cli::parse();
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

#[derive(clap::Parser, Debug)]
#[clap(name = "phpup")]
pub struct Cli {
    #[clap(flatten)]
    pub config: Config,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(clap::Parser, Debug)]
pub enum SubCommand {
    #[clap(name = "init")]
    Init(commands::Init),

    #[clap(name = "list-remote", visible_aliases = &["ls-remote"])]
    ListRemote(commands::ListRemote),

    #[clap(name = "install")]
    Install(commands::Install),

    #[clap(name = "list", visible_aliases = &["ls"])]
    List(commands::ListLocal),

    #[clap(name = "use")]
    Use(commands::Use),

    #[clap(name = "current")]
    Current(commands::Current),

    #[clap(name = "uninstall")]
    Uninstall(commands::Uninstall),

    #[clap(name = "alias")]
    Alias(commands::Alias),

    #[clap(name = "unalias")]
    Unalias(commands::Unalias),
}
