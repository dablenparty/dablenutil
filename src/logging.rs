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
    log_folder: PathBuf,
    filename: String,
    term_level_filter: LevelFilter,
    file_level_filter: LevelFilter,
    package_name: Option<String>,
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
    /// let config = LoggingConfig::new(PathBuf::from("./path/to/logs"));
    /// assert_eq!(config.get_log_folder(), PathBuf::from("./path/to/logs"));
    /// assert_eq!(config.get_term_level_filter(), log::LevelFilter::Info);
    /// assert_eq!(config.get_file_level_filter(), log::LevelFilter::Info);
    /// assert_eq!(config.get_package_name(), Some(env!("CARGO_PKG_NAME")));
    /// ```
    pub fn new(path: PathBuf) -> Self {
        Self {
            log_folder: path,
            filename: "latest.log".to_string(),
            term_level_filter: LevelFilter::Info,
            file_level_filter: LevelFilter::Info,
            package_name: Some(env!("CARGO_PKG_NAME").to_string()),
        }
    }

    /// Get the path to the log file.
    pub fn get_log_folder(&self) -> &Path {
        &self.log_folder
    }

    /// Gets the current level filter for the terminal logger.
    pub fn get_term_level_filter(&self) -> LevelFilter {
        self.term_level_filter
    }

    /// Gets the current filename for the log file.
    pub fn get_filename(&self) -> &str {
        &self.filename
    }

    /// Sets the filename for the log file.
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dablenutil::logging::LoggingConfig;
    /// # use log::LevelFilter;
    /// # use std::path::PathBuf;
    /// let log_folder = PathBuf::from("./path/to/logs");
    /// let mut config = LoggingConfig::new(log_folder).filename("my_log_file.log");
    /// # assert_eq!(config.get_filename(), "my_log_file.log");
    pub fn filename<S: Into<String>>(mut self, filename: S) -> Self {
        self.filename = filename.into();
        self
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
    pub fn get_package_name(&self) -> Option<&str> {
        self.package_name.as_deref()
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
    /// let mut config = LoggingConfig::new(log_file).package_name(Some("my_package"));
    /// assert_eq!(config.get_package_name(), Some("my_package"));
    /// ```
    pub fn package_name<S: Into<String>>(mut self, name: Option<S>) -> Self {
        self.package_name = name.map(Into::into);
        self
    }
}

/// Compresses the log file found at `{config.log_folder}/{config.filename}`..
///
/// The logs are compressed with `gzip` and `flate2`.
///
/// # Arguments
///
/// * `config` - The `LoggingConfig` to use.
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
/// # use dablenutil::logging::{LoggingConfig, rotate_logs};
/// # use log::info;
/// # use std::path::PathBuf;
/// # use std::fs;
///
/// # fn main() -> dablenutil::Result<()> {
/// let log_folder = PathBuf::from("./path/to/logs");
/// // path cloned for testing purposes
/// let config = LoggingConfig::new(log_folder.clone());
/// # assert!(!log_folder.exists());
/// rotate_logs(&config)?;
/// let log_file = log_folder.join(config.get_filename());
/// # assert!(log_file.ends_with("latest.log"));
/// # assert!(log_folder.exists());
/// # fs::write(&log_file, "Hello, world!")?;
/// # rotate_logs(&config)?;
/// # let prefix = format!("{}_", config.get_package_name().unwrap());
/// # let zipped_archive_exists = log_folder
/// #     .read_dir()?
/// #     .filter_map(|r| r.ok())
/// #     .any(|e| {
/// #         let file_name = e.file_name();
/// #         let encoded = file_name.to_string_lossy();
/// #         encoded.starts_with(&prefix)
/// #         && encoded.ends_with(".log.gz")
/// #     });
/// # assert!(zipped_archive_exists);
/// # fs::remove_dir_all(&log_folder)?;
/// # Ok(())
/// # }
/// ```
pub fn rotate_logs(config: &LoggingConfig) -> crate::Result<()> {
    let log_folder = config.get_log_folder();
    create_dir_if_not_exists(log_folder)?;
    let log_filename = config.get_filename();
    let latest_log_file = log_folder.join(log_filename);
    if latest_log_file.exists() {
        let create_time = latest_log_file
            .metadata()?
            .created()
            .map_or_else(|_| Local::now(), DateTime::<Local>::from);
        let prefix = {
            let package_name = config.get_package_name();
            package_name.map_or(String::default(), |s| format!("{}_", s))
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
    Ok(())
}

/// Initialize the logger with `simplelog`. Logs are outputted to the terminal
/// as well as the specified file.
///
/// This will create a new log file at the given path, but will not rotate the
/// logs. There is a dedicated function for that, [`rotate_logs`](fn@rotate_logs)
///
/// # Arguments
///
/// * `config` - The `LoggingConfig` to use.
///
/// # Errors
///
/// An error is returned if the log files could not be created for some reason.
///
/// # Examples
/// ```
/// # use dablenutil::logging::{LoggingConfig, init_simple_logger};
///
/// # fn main() -> dablenutil::Result<()> {
/// let path = std::path::PathBuf::from("./path/to/logs");
/// # assert!(!path.exists());
/// // path cloned for testing purposes
/// let config = LoggingConfig::new(path.clone());
/// init_simple_logger(&config)?;
/// log::info!("Hello, world!");
/// # assert!(path.exists());
/// # std::fs::remove_dir_all("path")?;
/// # Ok(())
/// # }
/// ```
pub fn init_simple_logger(config: &LoggingConfig) -> crate::Result<()> {
    let simplelog_config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(format_description!("[[[hour]:[minute]:[second]]"))
        .set_thread_mode(ThreadLogMode::Both)
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Error)
        .build();
    let log_path = config.get_log_folder();
    create_dir_if_not_exists(log_path)?;
    let log_file = fs::File::create(log_path.join(config.get_filename()))?;
    CombinedLogger::init(vec![
        TermLogger::new(
            config.get_term_level_filter(),
            simplelog_config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(config.get_file_level_filter(), simplelog_config, log_file),
    ])?;
    Ok(())
}
