use crate::commands::{self, Command};
use crate::config::Config;

#[derive(clap::Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    bin_name = "phpup",
    about = env!("CARGO_PKG_DESCRIPTION")
)]
pub struct Cli {
    #[clap(flatten)]
    pub config: Config,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(clap::Parser, Debug)]
pub enum SubCommand {
    /// Initialize and Print shell script required for PHP-UP
    #[command(bin_name = "init")]
    Init(commands::Init),

    /// List remote PHP versions
    #[command(bin_name = "list-remote", visible_aliases = &["ls-remote"])]
    ListRemote(commands::ListRemote),

    /// Install a new PHP version
    #[command(bin_name = "install")]
    Install(commands::Install),

    /// List local PHP versions
    #[command(bin_name = "list", visible_aliases = &["ls"])]
    List(commands::ListLocal),

    /// Switch PHP version
    #[command(bin_name = "use")]
    Use(commands::Use),

    /// Print the current PHP version
    #[command(bin_name = "current")]
    Current(commands::Current),

    /// Uninstall a PHP version
    #[command(bin_name = "uninstall")]
    Uninstall(commands::Uninstall),

    /// Alias a version to a common name
    #[command(bin_name = "alias")]
    Alias(commands::Alias),

    /// Remove an alias definition
    #[command(bin_name = "unalias")]
    Unalias(commands::Unalias),

    /// Set a version as the default version
    #[command(bin_name = "default")]
    Default(commands::Default),

    /// Print shell completions
    #[command(bin_name = "completions")]
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
            Default(cmd) => cmd.apply(&config),
            Completions(cmd) => cmd.apply(&config),
        };
    }
}
