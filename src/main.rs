use clap::Parser;
use phpup::cli::Cli;

fn main() {
    let Cli { config, subcmd } = Cli::parse();
    subcmd.apply(config);
}
