mod unix;
mod windos;

#[cfg(windows)]
pub use windos::Install;
