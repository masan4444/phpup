use super::Config;
use super::{list_local::Printer, Command};
use crate::release;
use crate::version::Version;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ListRemote {
    version: Option<String>,
}

impl Command for ListRemote {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let local_versions = &config.local_versions;
        let mut printer = Printer::new(local_versions);

        match &self.version {
            Some(version) => {
                let version = Version::from_str(version)?;
                if version.patch_version().is_some() {
                    let oldest_patch_release = release::fetch_oldest_patch(version)?;
                    let support = oldest_patch_release.calculate_support();
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
