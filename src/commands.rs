use anyhow::Result;

pub trait Command {
    fn run(&self) -> Result<()>;
}

mod init;
mod install;
mod list_local;
mod list_remote;
mod r#use;

pub use init::Init;
pub use install::Install;
pub use list_local::ListLocal;
pub use list_remote::ListRemote;
pub use r#use::Use;
