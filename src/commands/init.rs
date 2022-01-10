use crate::symlink;

use super::{Command, Config};
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Init {}

impl Command for Init {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let symlink = create_symlink(&config.versions_dir);
        println!("export PHPUP_MULTISHELL_PATH={:?}", symlink);
        println!("export PATH={:?}:$PATH", symlink.join("bin"));
        Ok(())
    }
}

fn create_symlink(versions_dir: impl AsRef<Path>) -> std::path::PathBuf {
    let system_temp_dir = std::env::temp_dir();
    let symlink_path = loop {
        let symlink_path = generate_symlink_path(&system_temp_dir);
        if !symlink_path.exists() {
            break symlink_path;
        }
    };

    symlink::link(versions_dir.as_ref().join("8.1.1"), &symlink_path)
        .expect("Can't create symlink!");
    symlink_path
}

fn generate_symlink_path(root: &std::path::Path) -> std::path::PathBuf {
    let temp_dir_name = format!(
        "phpup_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis(),
    );
    root.join(temp_dir_name)
}
