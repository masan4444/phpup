pub trait Command {
    fn run(&self) {}
}

mod install;
mod list;
mod list_remote;
mod r#use;

pub use install::Install;
pub use list::List;
pub use list_remote::ListRemote;
pub use r#use::Use;
