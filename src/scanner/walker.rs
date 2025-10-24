//! Directory traversal and file discovery

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Configuration for directory walking
#[derive(Debug, Clone)]
pub struct WalkConfig {
    /// Follow symbolic links
    pub follow_links: bool,
    /// Respect .gitignore files
    pub respect_gitignore: bool,
    /// Respect hidden files
    pub respect_hidden: bool,
    /// Maximum depth (None for unlimited)
    pub max_depth: Option<usize>,
    /// Number of threads for parallel walking
    pub threads: usize,
}

impl Default for WalkConfig {
    fn default() -> Self {
        Self {
            follow_links: false,
            respect_gitignore: true,
            respect_hidden: true,
            max_depth: None,
            threads: num_cpus::get(),
        }
    }
}

/// Walk directories and collect files to scan
pub fn walk_paths(paths: &[PathBuf], config: &WalkConfig) -> Result<Vec<PathBuf>, crate::Error> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            let dir_files = walk_directory(path, config)?;
            files.extend(dir_files);
        } else {
            return Err(crate::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Path not found: {}", path.display()),
            )));
        }
    }

    Ok(files)
}

/// Walk a single directory
fn walk_directory(path: &Path, config: &WalkConfig) -> Result<Vec<PathBuf>, crate::Error> {
    let mut builder = WalkBuilder::new(path);

    builder
        .follow_links(config.follow_links)
        .git_ignore(config.respect_gitignore)
        .hidden(config.respect_hidden)
        .threads(config.threads);

    if let Some(depth) = config.max_depth {
        builder.max_depth(Some(depth));
    }

    let mut files = Vec::new();

    for result in builder.build() {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    files.push(path.to_path_buf());
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to access path: {}", e);
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_walk_single_file() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        fs::write(&file, "test").unwrap();

        let config = WalkConfig::default();
        let files = walk_paths(std::slice::from_ref(&file), &config).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file);
    }

    #[test]
    fn test_walk_directory() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("file1.txt"), "test1").unwrap();
        fs::write(temp.path().join("file2.txt"), "test2").unwrap();

        let subdir = temp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file3.txt"), "test3").unwrap();

        let config = WalkConfig::default();
        let files = walk_paths(&[temp.path().to_path_buf()], &config).unwrap();

        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_walk_nonexistent_path() {
        let config = WalkConfig::default();
        let result = walk_paths(&[PathBuf::from("/nonexistent/path")], &config);

        assert!(result.is_err());
    }

    #[test]
    fn test_walk_config_max_depth() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("root.txt"), "root").unwrap();

        let subdir = temp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("sub.txt"), "sub").unwrap();

        let config = WalkConfig {
            max_depth: Some(1),
            ..Default::default()
        };

        let files = walk_paths(&[temp.path().to_path_buf()], &config).unwrap();

        // Should only find root.txt, not sub.txt due to depth limit
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("root.txt"));
    }

    #[test]
    fn test_walk_multiple_paths() {
        let temp1 = TempDir::new().unwrap();
        let temp2 = TempDir::new().unwrap();

        fs::write(temp1.path().join("file1.txt"), "test1").unwrap();
        fs::write(temp2.path().join("file2.txt"), "test2").unwrap();

        let config = WalkConfig::default();
        let files = walk_paths(
            &[temp1.path().to_path_buf(), temp2.path().to_path_buf()],
            &config,
        )
        .unwrap();

        assert_eq!(files.len(), 2);
    }
}
