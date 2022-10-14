use const_format::formatcp;
use std::env;

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
