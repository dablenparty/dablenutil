use std::path::Path;

/// Asynchronously creates a directory and all of its parent directories if they don't exist.
/// If the directory already exists, the error is ignored.
///
/// # Arguments
///
/// * `dir_path`: The path to the directory to create.
///
/// # Errors
///
/// An error is returned if the directory could not be created for some reason
/// (see `fs::create_dir_all` for more information), ignoring the error when the
/// directory already exists.
///
/// # Examples
/// ```
/// use dablenutil::tokio::async_create_dir_if_not_exists;
///
/// # #[tokio::main]
/// # async fn main() -> dablenutil::Result<()> {
/// let path = std::path::Path::new("path/to/dir");
/// assert_eq!(false, path.exists());
/// async_create_dir_if_not_exists(path).await?;
/// assert_eq!(true, path.exists());
/// # tokio::fs::remove_dir_all("path").await?;
/// # Ok(())
/// # }
/// ```
pub async fn async_create_dir_if_not_exists(dir_path: &Path) -> crate::Result<()> {
    if let Err(e) = tokio::fs::create_dir_all(dir_path).await {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            Ok(())
        } else {
            Err(e.into())
        }
    } else {
        Ok(())
    }
}
