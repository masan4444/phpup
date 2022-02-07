use crate::commands::{self, Command};
use crate::config::Config;

#[derive(clap::Parser, Debug)]
#[clap(name = "PHP-UP", bin_name = "phpup")]
pub struct Cli {
    #[clap(flatten)]
    pub config: Config,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(clap::Parser, Debug)]
pub enum SubCommand {
    /// Initialize and Print shell script required for PHP-UP
    #[clap(bin_name = "init")]
    Init(commands::Init),

    /// List remote PHP versions
    #[clap(bin_name = "list-remote", visible_aliases = &["ls-remote"])]
    ListRemote(commands::ListRemote),

    /// Install a new PHP version
    #[clap(bin_name = "install")]
    Install(commands::Install),

    /// List local PHP versions
    #[clap(bin_name = "list", visible_aliases = &["ls"])]
    List(commands::ListLocal),

    /// Switch PHP version
    #[clap(bin_name = "use")]
    Use(commands::Use),

    /// Print the current PHP version
    #[clap(bin_name = "current")]
    Current(commands::Current),

    /// Uninstall a PHP version
    #[clap(bin_name = "uninstall")]
    Uninstall(commands::Uninstall),

    /// Alias a version to a common name
    #[clap(bin_name = "alias")]
    Alias(commands::Alias),

    /// Remove an alias definition
    #[clap(bin_name = "unalias")]
    Unalias(commands::Unalias),

    /// Print shell completions
    #[clap(bin_name = "completions")]
    Completions(commands::Completions),
}

impl SubCommand {
    pub fn apply(&self, config: Config) {
        use SubCommand::*;
        match self {
            Init(cmd) => cmd.apply(&config),
            ListRemote(cmd) => cmd.apply(&config),
            Install(cmd) => cmd.apply(&config),
            List(cmd) => cmd.apply(&config),
            Use(cmd) => cmd.apply(&config),
            Current(cmd) => cmd.apply(&config),
            Uninstall(cmd) => cmd.apply(&config),
            Alias(cmd) => cmd.apply(&config),
            Unalias(cmd) => cmd.apply(&config),
            Completions(cmd) => cmd.apply(&config),
        };
    }
}
