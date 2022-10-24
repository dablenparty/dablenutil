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

/// Zip up the previous logs and start a new log file.
/// This returns the path to the new log file.
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
fn rotate_logs(log_folder: &Path) -> crate::Result<PathBuf> {
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

/// Initialize the logger with `simplelog`.
///
/// # Arguments
///
/// * `log_folder` - The path to the folder where the log files will be stored.
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
/// let path = std::path::Path::new("path/to/logs");
/// # assert_eq!(false, path.exists());
/// init_simple_logger(path, log::LevelFilter::Info)?;
/// # assert_eq!(true, path.exists());
/// # assert_eq!(true, path.join("latest.log").exists());
/// # std::fs::remove_dir_all("path")?;
/// # Ok(())
/// # }
/// ```
pub fn init_simple_logger(log_folder: &Path, level_filter: LevelFilter) -> crate::Result<()> {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(format_description!("[[[hour]:[minute]:[second]]"))
        .set_thread_mode(ThreadLogMode::Both)
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Error)
        .build();
    let log_path = rotate_logs(log_folder)?;
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
