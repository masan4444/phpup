use crate::config::Config;
use crate::config::Error as ConfigError;
use colored::Colorize;

pub trait Command {
    type Error: std::error::Error;
    fn run(&self, config: &Config) -> Result<(), Self::Error>;
    fn apply(&self, config: &Config) {
        if let Err(e) = self.run(config) {
            eprintln!("{}: {}", "error".red().bold(), e);
            std::process::exit(1);
        }
    }
}

mod alias;
mod completions;
mod current;
mod init;
mod install;
mod list_local;
mod list_remote;
mod unalias;
mod uninstall;
mod r#use;

pub use alias::Alias;
pub use completions::Completions;
pub use current::Current;
pub use init::Init;
pub use install::Install;
pub use list_local::ListLocal;
pub use list_remote::ListRemote;
pub use r#use::Use;
pub use unalias::Unalias;
pub use uninstall::Uninstall;
