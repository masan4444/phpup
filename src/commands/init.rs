use super::{Command, Config};
// use crate::symlink;
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub struct Init {}

#[derive(Error, Debug)]
pub enum Error {}

impl Command for Init {
    type Error = Error;
    fn run(&self, _config: &Config) -> Result<(), Error> {
        let symlink = create_symlink();
        println!("export PHPUP_MULTISHELL_PATH={:?}", symlink);
        println!("export PATH={:?}:$PATH", symlink.join("bin"));
        println!("rehash");
        Ok(())
    }
}

fn create_symlink() -> std::path::PathBuf {
    let temp_dir = std::env::temp_dir().join("phpup");
    std::fs::create_dir_all(&temp_dir).expect("Can't create tempdir!");
    let symlink_path = loop {
        let symlink_path = temp_dir.join(generate_symlink_path());
        if !symlink_path.exists() {
            break symlink_path;
        }
    };

    // TODO: default version
    // symlink::link(&default_version_dir, &symlink_path).expect("Can't create symlink!");
    symlink_path
}

fn generate_symlink_path() -> PathBuf {
    PathBuf::from(format!(
        "{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis(),
    ))
}
