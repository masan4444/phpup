use super::{
    list_local::{ListBase, Printer},
    Command,
};
use crate::release;
use crate::version::Version;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ListRemote {
    version: Option<String>,
}

impl ListBase for ListRemote {}

impl Command for ListRemote {
    fn run(&self) -> anyhow::Result<()> {
        let home_dir = dirs::home_dir()
            .expect("Can't get home directory")
            .join(".phpup");
        let versions_dir = home_dir.join("versions").join("php");
        let local_versions = Self::get_local_versions(versions_dir);
        let mut printer = Printer::new(local_versions);

        match &self.version {
            Some(version) => {
                let version = Version::from_str(version)?;
                if version.patch_version().is_some() {
                    let oldest_minor_release = release::fetch_oldest_patch(version)?;
                    let support = oldest_minor_release.calculate_support();
                    printer.print_version(version, Some(support));
                } else {
                    printer.print_releases(&release::fetch_all(version)?);
                }
            }
            None => {
                // Self::print_releases(&release::fetch_all_releases(Version::from_major(5))?);
                printer.print_releases(&release::fetch_all(Version::from_major(7))?);
                printer.print_releases(&release::fetch_all(Version::from_major(8))?);
            }
        };
        Ok(())
    }
}

use octocrab;
#[allow(unused)]
async fn fetch_versions_from_github() {
    let instance = octocrab::instance();

    let page = instance
        .repos("php", "php-src")
        .list_tags()
        .per_page(100u8)
        .send()
        .await
        .unwrap();
    println!("number of pages: {}", page.number_of_pages().unwrap());

    let tags = instance
        .all_pages::<octocrab::models::repos::Tag>(page)
        .await
        .unwrap();
    println!("number of tags: {}", tags.len());

    let tag_names = tags
        .iter()
        .map(|tag| tag.name.clone())
        .collect::<Vec<String>>();

    for tag_name in &tag_names {
        println!("{}", tag_name)
    }

    //     use tokio::runtime::Runtime;
    //     Runtime::new()
    //         .unwrap()
    //         .block_on(fetch_versions_from_github());
}
