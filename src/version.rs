//! Version information for Goofy
//! 
//! This module provides version information that can be set at build time
//! using environment variables or extracted from Cargo.toml.

/// The version of Goofy, set at build time
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the application
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/// The description of the application
pub const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// The authors of the application
pub const APP_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// The homepage URL
pub const APP_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

/// The repository URL
pub const APP_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Get the full version string with build information
pub fn full_version() -> String {
    format!("{} v{}", APP_NAME, VERSION)
}

/// Get a formatted version string for display
pub fn display_version() -> String {
    format!("v{}", VERSION)
}

/// Get build information string
pub fn build_info() -> String {
    format!(
        "{} v{}\nBuilt with Rust {}",
        APP_NAME,
        VERSION,
        rustc_version()
    )
}

/// Get the Rust compiler version used to build this binary
fn rustc_version() -> &'static str {
    // Use a default version if RUSTC_VERSION is not available
    option_env!("RUSTC_VERSION").unwrap_or("unknown")
}

/// Check if this is a development build
pub fn is_dev_build() -> bool {
    VERSION.contains("dev") || VERSION.ends_with("-SNAPSHOT")
}

/// Get a short version identifier suitable for logging
pub fn short_version() -> String {
    VERSION.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert!(!VERSION.is_empty());
        assert!(!APP_NAME.is_empty());
        assert_eq!(APP_NAME, "goofy");
    }

    #[test]
    fn test_version_functions() {
        let full = full_version();
        assert!(full.contains(APP_NAME));
        assert!(full.contains(VERSION));
        
        let display = display_version();
        assert!(display.starts_with('v'));
        assert!(display.contains(VERSION));
        
        let short = short_version();
        assert_eq!(short, VERSION);
    }

    #[test]
    fn test_build_info() {
        let info = build_info();
        assert!(info.contains(APP_NAME));
        assert!(info.contains(VERSION));
        assert!(info.contains("Rust"));
    }
}