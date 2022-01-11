use super::{Command, Config};
use crate::symlink;
use crate::version::Version;
use colored::Colorize;
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub struct Use {
    version: Option<Version>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Need to run `phpup install {0}`")]
    NotInstalledError(Version),
}

impl Command for Use {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let versions_dir = config.versions_dir();
        let local_versions = config.local_versions();

        match &self.version {
            Some(version) => {
                let version = local_versions
                    .iter()
                    .filter(|local_version| version.includes(local_version))
                    .max()
                    .ok_or(Error::NotInstalledError(*version))?;

                let multishell_path = config.multishell_path.as_ref().unwrap();
                let is_first_using = if multishell_path.exists() {
                    symlink::remove(multishell_path).expect("Can't remove symlink!");
                    false
                } else {
                    true
                };
                let new_original = versions_dir.join(version.to_string());
                symlink::link(new_original, multishell_path).expect("Can't create symlink!");
                println!("Using {}", version.to_string());
                if is_first_using {
                    println!(
                        "{}: Need to run `rehash` in this shell",
                        "warning".yellow().bold()
                    );
                }
            }
            None => todo!(),
        }
        Ok(())
    }
}
