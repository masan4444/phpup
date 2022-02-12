use super::{Command, Config};
use crate::release;
use crate::version;
use crate::version::Local;
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
    FailedFetchRelease(#[from] release::FetchError),
}

impl Command for ListRemote {
    type Error = Error;

    fn run(&self, config: &Config) -> Result<(), Error> {
        let query_versions = match &self.version {
            Some(version) => {
                if self.only_latest_patch && version.patch_version().is_some() {
                    println!(
                        "{}: '--latest-patch' is available only if patch number is NOT specified: {}",
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

        let installed_versions = version::installed(config).collect_vec();
        let current_version = Local::current(config);

        for query_version in query_versions {
            let releases = release::fetch_all(query_version)?;
            let remote_versions = releases.keys();

            let remote_versions = if self.only_latest_patch {
                filter_latest_patch(remote_versions).collect_vec()
            } else {
                remote_versions.collect_vec()
            };

            for &remote_version in remote_versions {
                let installed = installed_versions.contains(&remote_version);
                let remote_version = Local::Installed(remote_version);
                let used = Some(&remote_version) == current_version.as_ref();
                println!("{}", remote_version.to_string_by(installed, used, config))
            }
        }
        Ok(())
    }
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
