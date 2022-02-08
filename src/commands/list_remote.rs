use super::Config;
use super::{list_local::Printer, Command};
use crate::release;
use crate::version::Version;
use clap;
use colored::Colorize;
use itertools::Itertools;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct ListRemote {
    version: Option<Version>,
    #[clap(
        short,
        long,
        conflicts_with = "version",
        help = "List all old versions"
    )]
    all: bool,
    #[clap(
        long = "latest-patch",
        visible_alias = "lp",
        help = "List latest patch release (avairable only if patch number is NOt specified)"
    )]
    only_latest_patch: bool,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    CantFetchReleaseError(#[from] release::Error),
}

impl Command for ListRemote {
    type Error = Error;
    fn run(&self, config: &Config) -> Result<(), Error> {
        let query_versions = match &self.version {
            Some(version) => {
                if self.only_latest_patch && version.patch_version().is_some() {
                    println!(
                        "{}: `--latest-patch` is available only if patch number is NOT specified: {}",
                        "warning".yellow().bold(),
                        version
                    );
                }
                vec![*version]
            }
            None => {
                if self.all {
                    vec![
                        Version::from_major(3),
                        Version::from_major(4),
                        Version::from_major(5),
                        Version::from_major(7),
                        Version::from_major(8),
                    ]
                } else {
                    vec![Version::from_major(7), Version::from_major(8)]
                }
            }
        };

        let local_versions = config.local_versions().collect_vec();
        let aliases = config.aliases();
        let printer = Printer::new(&local_versions, config.current_version(), &aliases);
        featch_and_print_versions(&query_versions, self.only_latest_patch, &printer)?;
        Ok(())
    }
}

fn featch_and_print_versions(
    query_versions: &[Version],
    latest_patch: bool,
    printer: &Printer,
) -> Result<(), Error> {
    for &query_version in query_versions {
        let releases = release::fetch_all(query_version)?;
        let versions = releases.keys();
        if latest_patch {
            printer.print_versions(filter_latest_patch(versions));
        } else {
            printer.print_versions(versions);
        };
    }
    Ok(())
}

fn filter_latest_patch<'a, I>(versions: I) -> impl Iterator<Item = &'a Version>
where
    I: Iterator<Item = &'a Version> + DoubleEndedIterator,
{
    let mut latest_patch: Option<&'a Version> = None;
    let mut latest_patches = versions
        .rev()
        .filter_map(|version| {
            if latest_patch.is_none()
                || latest_patch.unwrap().minor_version() != version.minor_version()
            {
                latest_patch.replace(version)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    latest_patches.push(latest_patch.unwrap());
    latest_patches.into_iter().rev()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_not_specified() {
        let base_dir = tempfile::tempdir().unwrap();
        let config = Config::default().with_base_dir(base_dir);
        let cmd = ListRemote {
            version: None,
            all: true,
            only_latest_patch: false,
        };
        assert!(cmd.run(&config).is_ok());
    }
    #[test]
    fn version_specified() {
        let base_dir = tempfile::tempdir().unwrap();
        let config = Config::default().with_base_dir(base_dir);
        let cmd = ListRemote {
            version: Some("7.2".parse().unwrap()),
            all: false,
            only_latest_patch: false,
        };
        assert!(cmd.run(&config).is_ok());
    }
}

// TODO: can't get last itm
// fn filter_latest_patch<'a, T>(versions: T) -> impl Iterator<Item = &'a Version>
// where
//     T: Iterator<Item = &'a Version> + DoubleEndedIterator,
// {
//     let mut latest_patch: Option<&'a Version> = None;
//     versions
//         .map(move |version| match latest_patch {
//             Some(latest) if latest.minor_version().unwrap() == version.minor_version().unwrap() => {
//                 latest_patch.replace(version);
//                 None
//             }
//             _ => latest_patch.replace(version),
//         })
//         .filter_map(|e| e)
// }
