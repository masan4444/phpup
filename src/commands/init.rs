use std::path::Path;

use super::Command;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Init {}

impl Command for Init {
    fn run(&self) -> anyhow::Result<()> {
        let symlink = create_symlink();
        println!("export PHPUP_MULTISHELL_PATH={:?}", symlink);
        println!("export PATH={:?}:$PATH", symlink.join("bin"));
        Ok(())
    }
}

fn create_symlink() -> std::path::PathBuf {
    let home_dir = dirs::home_dir()
        .expect("Can't get home directory")
        .join(".phpup");
    let versions_dir = home_dir.join("versions").join("php");

    let system_temp_dir = std::env::temp_dir();
    let mut temp_dir = generate_symlink_path(&system_temp_dir);
    while temp_dir.exists() {
        temp_dir = generate_symlink_path(&system_temp_dir);
    }

    create_symlink_dir(versions_dir.join("8.1.1"), &temp_dir).expect("Can't create symlink!");
    temp_dir
}

fn generate_symlink_path(root: &std::path::Path) -> std::path::PathBuf {
    let temp_dir_name = format!(
        "phpup_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis(),
    );
    root.join(temp_dir_name)
}

#[cfg(unix)]
fn create_symlink_dir<P: AsRef<Path>, U: AsRef<Path>>(original: P, link: U) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)?;
    Ok(())
}
