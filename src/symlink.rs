use std::path::Path;

#[cfg(unix)]
pub fn link<P: AsRef<Path>, U: AsRef<Path>>(original: P, link: U) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)?;
    Ok(())
}

#[cfg(unix)]
pub fn remove<P: AsRef<Path>>(symlink_file: P) -> std::io::Result<()> {
    if std::fs::symlink_metadata(&symlink_file).is_ok() {
        std::fs::remove_file(symlink_file)?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn link<P: AsRef<Path>, U: AsRef<Path>>(from: P, to: U) -> std::io::Result<()> {
    junction::create(from, to)
}

#[cfg(windows)]
pub fn remove<P: AsRef<Path>>(junction: P) -> std::io::Result<()> {
    if junction::exists(&junction).is_ok() {
        std::fs::remove_dir(junction)?;
    }
    Ok(())
}
