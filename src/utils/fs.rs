// File system utilities

use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    collections::HashSet,
};
use walkdir::{DirEntry, WalkDir};
use tracing::{debug, warn};

/// Configuration for directory traversal
#[derive(Debug, Clone)]
pub struct WalkConfig {
    /// Maximum depth to traverse
    pub max_depth: Option<usize>,
    /// Patterns to ignore (gitignore-style)
    pub ignore_patterns: Vec<String>,
    /// File extensions to include (if empty, include all)
    pub include_extensions: Vec<String>,
    /// Whether to follow symbolic links
    pub follow_links: bool,
    /// Whether to include hidden files
    pub include_hidden: bool,
}

impl Default for WalkConfig {
    fn default() -> Self {
        Self {
            max_depth: Some(10),
            ignore_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".cache".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ],
            include_extensions: vec![],
            follow_links: false,
            include_hidden: false,
        }
    }
}

/// File information structure
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub modified: Option<std::time::SystemTime>,
}

/// Walk a directory and return filtered file information
pub fn walk_directory<P: AsRef<Path>>(
    root: P, 
    config: Option<WalkConfig>
) -> Result<Vec<FileInfo>> {
    let root = root.as_ref();
    let config = config.unwrap_or_default();
    
    debug!("Walking directory: {} with config: {:?}", root.display(), config);
    
    let mut files = Vec::new();
    let mut walker = WalkDir::new(root);
    
    if let Some(max_depth) = config.max_depth {
        walker = walker.max_depth(max_depth);
    }
    
    if config.follow_links {
        walker = walker.follow_links(true);
    }
    
    // Convert ignore patterns to a more efficient lookup
    let ignore_patterns: HashSet<String> = config.ignore_patterns.iter().cloned().collect();
    let include_extensions: HashSet<String> = config.include_extensions.iter().cloned().collect();
    
    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Error walking directory: {}", e);
                continue;
            }
        };
        
        // Skip if should be ignored
        if should_ignore_entry(&entry, &config, &ignore_patterns) {
            continue;
        }
        
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(e) => {
                warn!("Error getting metadata for {}: {}", entry.path().display(), e);
                continue;
            }
        };
        
        let relative_path = entry.path().strip_prefix(root)
            .unwrap_or(entry.path())
            .to_path_buf();
        
        let extension = entry.path()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase());
        
        // Filter by extensions if specified
        if !include_extensions.is_empty() {
            if let Some(ref ext) = extension {
                if !include_extensions.contains(ext) {
                    continue;
                }
            } else if !metadata.is_dir() {
                // Skip files without extensions if we have an extension filter
                continue;
            }
        }
        
        let file_info = FileInfo {
            path: entry.path().to_path_buf(),
            relative_path,
            size: metadata.len(),
            is_dir: metadata.is_dir(),
            extension,
            modified: metadata.modified().ok(),
        };
        
        files.push(file_info);
    }
    
    debug!("Found {} files/directories", files.len());
    Ok(files)
}

/// Check if a directory entry should be ignored
fn should_ignore_entry(
    entry: &DirEntry,
    config: &WalkConfig,
    ignore_patterns: &HashSet<String>
) -> bool {
    let path = entry.path();
    let file_name = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    
    // Skip hidden files if not included
    if !config.include_hidden && file_name.starts_with('.') && file_name != "." {
        return true;
    }
    
    // Check against ignore patterns
    for pattern in ignore_patterns {
        if pattern.contains('*') {
            // Simple glob matching
            if matches_glob_pattern(file_name, pattern) {
                return true;
            }
        } else if file_name == pattern || path.to_string_lossy().contains(pattern) {
            return true;
        }
    }
    
    false
}

/// Simple glob pattern matching
fn matches_glob_pattern(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    
    if pattern.starts_with('*') && pattern.ends_with('*') {
        let middle = &pattern[1..pattern.len() - 1];
        return text.contains(middle);
    }
    
    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        return text.ends_with(suffix);
    }
    
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return text.starts_with(prefix);
    }
    
    text == pattern
}

/// Read file contents with error handling
pub fn read_file_safe<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    debug!("Reading file: {}", path.display());
    
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))
}

/// Write file contents with error handling and backup
pub fn write_file_safe<P: AsRef<Path>>(path: P, contents: &str) -> Result<()> {
    let path = path.as_ref();
    debug!("Writing file: {}", path.display());
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directories for: {}", path.display()))?;
    }
    
    fs::write(path, contents)
        .with_context(|| format!("Failed to write file: {}", path.display()))
}

/// Ensure a directory exists
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        debug!("Creating directory: {}", path.display());
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Get file extension in lowercase
pub fn get_file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

/// Check if a file is likely a text file based on extension
pub fn is_text_file<P: AsRef<Path>>(path: P) -> bool {
    const TEXT_EXTENSIONS: &[&str] = &[
        "txt", "md", "markdown", "rst", "adoc", "asciidoc",
        "rs", "go", "py", "js", "ts", "jsx", "tsx", "vue", "svelte",
        "c", "cpp", "cc", "cxx", "h", "hpp", "hxx",
        "java", "scala", "kt", "swift", "dart",
        "php", "rb", "perl", "lua", "sh", "bash", "zsh", "fish",
        "html", "htm", "xml", "svg", "css", "scss", "sass", "less",
        "json", "yaml", "yml", "toml", "ini", "cfg", "conf",
        "sql", "graphql", "gql",
        "dockerfile", "makefile", "cmake",
        "gitignore", "gitattributes", "gitmodules",
        "license", "readme", "changelog", "todo",
    ];
    
    if let Some(ext) = get_file_extension(path) {
        TEXT_EXTENSIONS.contains(&ext.as_str())
    } else {
        false
    }
}

/// Check if a file is likely a binary file
pub fn is_binary_file<P: AsRef<Path>>(path: P) -> bool {
    const BINARY_EXTENSIONS: &[&str] = &[
        "exe", "dll", "so", "dylib", "a", "o", "obj",
        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "ico",
        "mp3", "wav", "flac", "ogg", "m4a", "aac",
        "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm",
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "zip", "tar", "gz", "bz2", "xz", "7z", "rar",
        "bin", "dat", "db", "sqlite", "sqlite3",
    ];
    
    if let Some(ext) = get_file_extension(path) {
        BINARY_EXTENSIONS.contains(&ext.as_str())
    } else {
        false
    }
}

/// Get relative path from one path to another
pub fn get_relative_path<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> PathBuf {
    let from = from.as_ref();
    let to = to.as_ref();
    
    match to.strip_prefix(from) {
        Ok(relative) => relative.to_path_buf(),
        Err(_) => to.to_path_buf(),
    }
}

/// Find files by pattern in a directory
pub fn find_files_by_pattern<P: AsRef<Path>>(
    root: P,
    pattern: &str,
    case_sensitive: bool,
) -> Result<Vec<PathBuf>> {
    let config = WalkConfig {
        max_depth: Some(20),
        ..Default::default()
    };
    
    let files = walk_directory(root, Some(config))?;
    let pattern = if case_sensitive { pattern.to_string() } else { pattern.to_lowercase() };
    
    Ok(files
        .into_iter()
        .filter_map(|file| {
            let file_name = file.path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            
            let search_name = if case_sensitive { 
                file_name.to_string() 
            } else { 
                file_name.to_lowercase() 
            };
            
            if search_name.contains(&pattern) {
                Some(file.path)
            } else {
                None
            }
        })
        .collect())
}

/// Calculate directory size recursively
pub fn calculate_dir_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let config = WalkConfig {
        include_hidden: true,
        ..Default::default()
    };
    
    let files = walk_directory(path, Some(config))?;
    let total_size = files
        .iter()
        .filter(|file| !file.is_dir)
        .map(|file| file.size)
        .sum();
    
    Ok(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_walk_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        // Create test structure
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]").unwrap();
        fs::write(root.join("target/debug"), "binary").unwrap();
        fs::write(root.join(".git/config"), "git config").unwrap();
        
        let files = walk_directory(root, None).unwrap();
        
        // Should find src/main.rs and Cargo.toml, but not target or .git
        let file_names: Vec<String> = files
            .iter()
            .map(|f| f.relative_path.to_string_lossy().to_string())
            .collect();
        
        assert!(file_names.iter().any(|name| name.contains("main.rs")));
        assert!(file_names.iter().any(|name| name.contains("Cargo.toml")));
        assert!(!file_names.iter().any(|name| name.contains("target")));
    }

    #[test]
    fn test_glob_pattern_matching() {
        assert!(matches_glob_pattern("test.txt", "*.txt"));
        assert!(matches_glob_pattern("test.txt", "test.*"));
        assert!(matches_glob_pattern("test.txt", "*test*"));
        assert!(!matches_glob_pattern("test.rs", "*.txt"));
    }

    #[test]
    fn test_file_type_detection() {
        assert!(is_text_file("test.rs"));
        assert!(is_text_file("README.md"));
        assert!(is_binary_file("image.png"));
        assert!(is_binary_file("binary.exe"));
        assert!(!is_text_file("binary.exe"));
        assert!(!is_binary_file("test.rs"));
    }
}