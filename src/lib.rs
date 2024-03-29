//! A collection of utilities for my Rust projects.
//!
//! This crate is a collection of utilities that I use in my Rust projects. It is not intended to be
//! used by anyone else, but I have made it public in case anyone else finds it useful.
//! While I try to keep this crate as stable as possible, I make no guarantees about the stability of
//! the API. As such, I recommend that you use a specific version of this crate in your `Cargo.toml`.
//! Being a student, I don't have time for maintaining a crate as insignificant as this one.
//!
//! # Features
//!
//! * `logging` - Enables the `logging` module.
//! * `tokio` - Enables the `tokio` module for async utils.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args, clippy::must_use_candidate, clippy::return_self_not_must_use)]

use const_format::formatcp;
use std::{env, error, fmt, fs::create_dir_all, io, path::Path};

#[cfg(feature = "logging")]
pub mod logging;
#[cfg(feature = "tokio")]
pub mod tokio;

/// A custom error type for this crate.
#[derive(Debug)]
pub enum Error {
    /// Wraps an `io::Error`.
    Io(io::Error),
    /// Wraps an error from `simplelog`.
    #[cfg(feature = "logging")]
    Logging(log::SetLoggerError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO Error: {}", e),
            #[cfg(feature = "logging")]
            Error::Logging(e) => write!(f, "Logging Error: {}", e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

#[cfg(feature = "logging")]
impl From<log::SetLoggerError> for Error {
    fn from(e: log::SetLoggerError) -> Self {
        Error::Logging(e)
    }
}

/// Gets a platform-specific executable name based on the `CARGO_PKG_NAME` environment variable.
///
/// This function is generated at compile-time and can be used in `const` contexts.
///
/// # Examples
///
/// ```
/// # use std::env::consts::*;
/// use dablenutil::platform_specific_executable_name;
/// # const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
///
/// let executable_name = platform_specific_executable_name();
/// let expected_name = format!("{}_{}_{}{}", CARGO_PKG_NAME, OS, ARCH, EXE_SUFFIX);
/// assert_eq!(expected_name, executable_name);
/// ```
#[must_use]
pub const fn platform_specific_executable_name() -> &'static str {
    const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
    formatcp!(
        "{}_{}_{}{}",
        PACKAGE_NAME,
        env::consts::OS,
        env::consts::ARCH,
        env::consts::EXE_SUFFIX
    )
}

/// Synchronously creates a directory and all of its parent directories if they don't exist.
/// If the directory already exists, the error is ignored.
///
/// # Arguments
///
/// * `dir` - The path to the directory to create.
///
/// # Errors
///
/// An error is returned if the directory could not be created for some reason
/// (see `fs::create_dir_all` for more information), ignoring the error when the
/// directory already exists.
///
/// # Examples
/// ```
/// use dablenutil::create_dir_if_not_exists;
///
/// # fn main() -> dablenutil::Result<()> {
/// let path = std::path::Path::new("path/to/dir");
/// # assert_eq!(false, path.exists());
/// create_dir_if_not_exists(path)?;
/// assert!(path.exists());
/// # std::fs::remove_dir_all("path")?;
/// # Ok(())
/// # }
/// ```
pub fn create_dir_if_not_exists(dir: &Path) -> Result<()> {
    if let Err(e) = create_dir_all(dir) {
        if e.kind() == io::ErrorKind::AlreadyExists {
            Ok(())
        } else {
            Err(e.into())
        }
    } else {
        Ok(())
    }
}
