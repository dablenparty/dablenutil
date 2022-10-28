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

/// Zip up the previous logs and start a new log file, returning
/// the path to the new log file.
///
/// The logs are zipped with `gzip` and `flate2`, 
///
/// returns: `io::Result<PathBuf>`
///
/// # Arguments
///
/// * `log_folder` - The path to the directory where the logs are stored.
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
/// # assert_eq!(false, log_path.exists());
/// let log_file = rotate_logs(&log_path)?;
/// # assert!(log_file.ends_with("latest.log"));
/// # assert!(log_path.exists());
/// # fs::write(&log_file, "Hello, world!")?;
/// # let second_log_file = rotate_logs(&log_path)?;
/// # let zipped_archive_exists = log_path
/// #     .read_dir()?
/// #     .filter_map(|r| r.ok())
/// #     .any(|e| e.file_name().to_string_lossy().ends_with(".log.gz"));
/// # assert!(zipped_archive_exists);
/// # fs::remove_dir_all(&log_path)?;
/// # Ok(())
/// # }
/// ```
pub fn rotate_logs(log_folder: &Path) -> crate::Result<PathBuf> {
    create_dir_if_not_exists(&log_folder)?;
    let latest_log_file = log_folder.join("latest.log");
    if latest_log_file.exists() {
        let create_time = latest_log_file
            .metadata()?
            .created()
            .map_or_else(|_| Local::now(), DateTime::<Local>::from);
        let package_name = env!("CARGO_PKG_NAME");
        let dated_name = create_time
            .format(&format!("{}_%Y-%m-%d_%H-%M-%S.log", package_name))
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
/// # assert_eq!(false, path.exists());
/// init_simple_logger(path, log::LevelFilter::Info)?;
/// log::info!("Hello, world!");
/// # assert_eq!(true, path.exists());
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
