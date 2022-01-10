use super::{Command, Config};
use crate::symlink;
use crate::version::Version;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Use {
    version: Option<String>,
}

impl Command for Use {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let versions_dir = &config.versions_dir;
        let local_versions = &config.local_versions;

        match &self.version {
            Some(version) => {
                let version = Version::from_str(version)?;
                if local_versions.contains(&version) {
                    let multishell_path = config.multishell_path.as_ref().unwrap();
                    if multishell_path.exists() {
                        symlink::remove(multishell_path).expect("Can't remove symlink!");
                    }
                    let new_original = versions_dir.join(version.to_string());
                    symlink::link(new_original, multishell_path).expect("Can't create symlink!");
                } else {
                    println!("please install");
                    todo!()
                }
            }
            None => todo!(),
        }
        Ok(())
    }
}
