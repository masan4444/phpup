use super::{Command, Config};
use crate::releases;
use crate::version;
use crate::version::Local;
use crate::version::Version;
use colored::Colorize;
use itertools::Itertools;
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct ListRemote {
    version: Option<Version>,
    #[arg(
        long = "latest-patch",
        visible_alias = "lp",
        help = "List latest patch release (avairable only if patch number is NOT specified)"
    )]
    only_latest_patch: bool,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FailedFetchRelease(#[from] releases::FetchError),
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
                vec![
                    Version::from_major(3),
                    Version::from_major(4),
                    Version::from_major(5),
                    Version::from_major(7),
                    Version::from_major(8),
                ]
            }
        };

        let installed_versions = version::installed(config).collect_vec();
        let current_version = Local::current(config);
        let pre_releases = releases::pre_release::fetch_all()?;

        for query_version in query_versions {
            let remote_versions = if query_version.pre_type().is_some() {
                match pre_releases.get(&query_version) {
                    Some(v) => Ok(vec![v.version]),
                    None => Err(releases::FetchError::NotFoundRelease(query_version)),
                }
            } else {
                let releases = releases::release::fetch_all(query_version);
                let pre_release_keys = pre_releases
                    .get_versions_included_by(&query_version)
                    .sorted();
                match releases {
                    Ok(r) => {
                        let mut keys: BTreeSet<Version> = r.keys().copied().collect();
                        keys.extend(pre_release_keys);
                        if self.only_latest_patch {
                            Ok(filter_latest_patch(keys.iter()).copied().collect_vec())
                        } else {
                            Ok(keys.iter().copied().collect_vec())
                        }
                    }
                    Err(_) if pre_release_keys.len() > 0 => Ok(pre_release_keys.copied().collect()),
                    Err(e) => Err(e),
                }
            }?;

            for remote_version in remote_versions {
                let installed = installed_versions.contains(&remote_version);
                let remote_version = Local::Installed(remote_version);
                let used = Some(&remote_version) == current_version.as_ref();
                println!("{}", remote_version.to_string_by(installed, used))
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
