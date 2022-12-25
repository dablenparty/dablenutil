//! Contains logging utilities. This module is only available when the `logging` feature is enabled.
//!
//! Currently, this module contains a function to initialize a logger and a function to rotate logs.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use flate2::{Compression, GzBuilder};
use log::LevelFilter;
use simplelog::{
    format_description, ColorChoice, CombinedLogger, TermLogger, TerminalMode, ThreadLogMode,
    WriteLogger,
};

use crate::create_dir_if_not_exists;

pub struct LoggingConfig {
    log_path: PathBuf,
    term_level_filter: LevelFilter,
    file_level_filter: LevelFilter,
    package_name: String,
}

impl LoggingConfig {
    /// Constructs a new `LoggingConfig` with the default values.
    /// The default values are:
    /// * `log_path`: given as parameter
    /// * `term_level_filter`: `LevelFilter::Info`
    /// * `file_level_filter`: `LevelFilter::Info`
    /// * `package_name`: `env!("CARGO_PKG_NAME")`
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the log file.
    ///
    /// # Examples
    /// ```
    /// # use dablenutil::logging::LoggingConfig;
    /// # use log::LevelFilter;
    /// # use std::path::PathBuf;
    /// let config = LoggingConfig::new(PathBuf::from("./path/to/log/file.log"));
    /// assert_eq!(config.get_log_path(), PathBuf::from("./path/to/log/file.log"));
    /// assert_eq!(config.get_term_level_filter(), log::LevelFilter::Info);
    /// assert_eq!(config.get_file_level_filter(), log::LevelFilter::Info);
    /// assert_eq!(config.get_package_name(), env!("CARGO_PKG_NAME"));
    /// ```
    pub fn new(path: PathBuf) -> Self {
        Self {
            log_path: path,
            term_level_filter: LevelFilter::Info,
            file_level_filter: LevelFilter::Info,
            package_name: env!("CARGO_PKG_NAME").to_string(),
        }
    }

    /// Get the path to the log file.
    pub fn get_log_path(&self) -> &Path {
        &self.log_path
    }

    /// Gets the current level filter for the terminal logger.
    pub fn get_term_level_filter(&self) -> LevelFilter {
        self.term_level_filter
    }

    /// Sets the level filter for the terminal logger.
    ///
    /// # Arguments
    /// * `level` - The level filter to set.
    ///
    /// # Examples
    /// ```
    /// # use dablenutil::logging::LoggingConfig;
    /// # use log::LevelFilter;
    /// # use std::path::PathBuf;
    /// let log_file = PathBuf::from("./path/to/log/file.log");
    /// let mut config = LoggingConfig::new(log_file).term_level_filter(LevelFilter::Debug);
    /// assert_eq!(config.get_term_level_filter(), log::LevelFilter::Debug);
    pub fn term_level_filter(mut self, level: LevelFilter) -> Self {
        self.term_level_filter = level;
        self
    }

    /// Gets the current level filter for the file logger.
    pub fn get_file_level_filter(&self) -> LevelFilter {
        self.file_level_filter
    }

    /// Sets the level filter for the file logger.
    ///
    /// # Arguments
    /// * `level` - The level filter to set.
    ///
    /// # Examples
    /// ```
    /// # use dablenutil::logging::LoggingConfig;
    /// # use log::LevelFilter;
    /// # use std::path::PathBuf;
    /// let log_file = PathBuf::from("./path/to/log/file.log");
    /// let mut config = LoggingConfig::new(log_file).file_level_filter(LevelFilter::Debug);
    /// assert_eq!(config.get_file_level_filter(), log::LevelFilter::Debug);
    /// ```
    pub fn file_level_filter(mut self, level: LevelFilter) -> Self {
        self.file_level_filter = level;
        self
    }

    /// Gets the current package name.
    pub fn get_package_name(&self) -> &str {
        &self.package_name
    }

    /// Sets the package name to prepend to the log archives. If this is set to an empty string, then
    /// nothing will be prepended to the log archives.
    ///
    /// # Arguments
    /// * `name` - The package name to set.
    ///
    /// # Examples
    /// ```
    /// # use dablenutil::logging::LoggingConfig;
    /// # use log::LevelFilter;
    /// # use std::path::PathBuf;
    /// let log_file = PathBuf::from("./path/to/log/file.log");
    /// let mut config = LoggingConfig::new(log_file).package_name("my_package");
    /// assert_eq!(config.get_package_name(), "my_package");
    /// ```
    pub fn package_name(mut self, name: &str) -> Self {
        self.package_name = name.to_string();
        self
    }
}

/// Zip up the previous logs and start a new log file, returning
/// the path to the new log file.
///
/// The logs are zipped with `gzip` and `flate2`.
///
/// returns: `io::Result<PathBuf>`
///
/// # Arguments
///
/// * `log_folder` - The path to the directory where the logs are stored.
/// * `package_name` - An optional package name to prepend to the log archives.
/// An underscore `_` will be appended to it as well. See the Examples for more.
///
/// # Errors
///
/// An error is returned if the directory could not be created, the log file metadata could not be
/// retrieved, or the log file could not be renamed.
///
/// # Examples
///
/// ```
/// use dablenutil::logging::rotate_logs;
/// use log::info;
/// use std::path::PathBuf;
/// use std::fs;
///
/// # fn main() -> dablenutil::Result<()> {
/// let log_path = PathBuf::from("./path/to/logs");
/// # assert!(!log_path.exists());
/// let package_name = env!("CARGO_PKG_NAME");
/// let log_file = rotate_logs(&log_path, Some(package_name))?;
/// # assert!(log_file.ends_with("latest.log"));
/// # assert!(log_path.exists());
/// # fs::write(&log_file, "Hello, world!")?;
/// # let second_log_file = rotate_logs(&log_path, Some(package_name))?;
/// # let prefix = format!("{}_", package_name);
/// # let zipped_archive_exists = log_path
/// #     .read_dir()?
/// #     .filter_map(|r| r.ok())
/// #     .any(|e| {
/// #         let file_name = e.file_name();
/// #         let encoded = file_name.to_string_lossy();
/// #         encoded.starts_with(&prefix)
/// #         && encoded.ends_with(".log.gz")
/// #     });
/// # assert!(zipped_archive_exists);
/// # fs::remove_dir_all(&log_path)?;
/// # Ok(())
/// # }
/// ```
pub fn rotate_logs(log_folder: &Path, package_name: Option<&str>) -> crate::Result<PathBuf> {
    create_dir_if_not_exists(&log_folder)?;
    let latest_log_file = log_folder.join("latest.log");
    if latest_log_file.exists() {
        let create_time = latest_log_file
            .metadata()?
            .created()
            .map_or_else(|_| Local::now(), DateTime::<Local>::from);
        let prefix = if let Some(name) = package_name {
            format!("{}_", name)
        } else {
            "".to_string()
        };
        let dated_name = create_time
            .format(&format!("{}%Y-%m-%d_%H-%M-%S.log", prefix))
            .to_string();
        let archive_path = log_folder.join(format!("{}.gz", dated_name));
        let file_handle = fs::File::create(archive_path)?;
        let last_log_data = fs::read(&latest_log_file)?;
        let mut gz = GzBuilder::new()
            .filename(dated_name)
            .write(file_handle, Compression::default());
        gz.write_all(&last_log_data)?;
        gz.finish()?;
        fs::remove_file(&latest_log_file)?;
    }
    Ok(latest_log_file)
}

/// Initialize the logger with `simplelog`. Logs are outputted to the terminal
/// as well as the specified file.
///
/// This will create a new log file at the given path, but will not rotate the
/// logs. There is a dedicated function for that, [`rotate_logs`](fn@rotate_logs)
///
/// # Arguments
///
/// * `log_path` - The path to the log file.
/// * `level_filter` - The log level to use.
///
/// # Errors
///
/// An error is returned if the log files could not be created for some reason or if one
/// occurs while zipping up the previous logs.
///
/// # Examples
/// ```
/// use dablenutil::logging::init_simple_logger;
///
/// # fn main() -> dablenutil::Result<()> {
/// let path = std::path::Path::new("./path/to/file.log");
/// # assert!(!path.exists());
/// init_simple_logger(path, log::LevelFilter::Info)?;
/// log::info!("Hello, world!");
/// # assert!(path.exists());
/// # std::fs::remove_dir_all("path")?;
/// # Ok(())
/// # }
/// ```
pub fn init_simple_logger(log_path: &Path, level_filter: LevelFilter) -> crate::Result<()> {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(format_description!("[[[hour]:[minute]:[second]]"))
        .set_thread_mode(ThreadLogMode::Both)
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Error)
        .build();
    log_path
        .parent()
        .map_or_else(|| Ok(()), |p| create_dir_if_not_exists(p))?;
    let log_file = fs::File::create(log_path)?;
    CombinedLogger::init(vec![
        TermLogger::new(
            level_filter,
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(level_filter, config, log_file),
    ])?;
    Ok(())
}
