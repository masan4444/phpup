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
