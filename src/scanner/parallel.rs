//! Parallel file scanning with rayon

use crate::report::{ScanError, Violation};
use crate::scanner::file_scanner::scan_file;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Scan multiple files in parallel
pub fn scan_files_parallel(
    files: Vec<PathBuf>,
    num_threads: Option<usize>,
) -> (Vec<Violation>, Vec<ScanError>) {
    // Configure rayon thread pool if specified
    if let Some(threads) = num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("Failed to build thread pool");
    }

    let violations = Arc::new(Mutex::new(Vec::new()));
    let errors = Arc::new(Mutex::new(Vec::new()));

    files.par_iter().for_each(|file| match scan_file(file) {
        Ok(file_violations) => {
            if !file_violations.is_empty() {
                let mut v = violations.lock().unwrap();
                v.extend(file_violations);
            }
        }
        Err(e) => {
            let mut errs = errors.lock().unwrap();
            errs.push(ScanError::new(
                file.clone(),
                crate::report::violation::ErrorType::IoError,
                e.to_string(),
            ));
        }
    });

    let final_violations = Arc::try_unwrap(violations).unwrap().into_inner().unwrap();
    let final_errors = Arc::try_unwrap(errors).unwrap().into_inner().unwrap();

    (final_violations, final_errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_files_parallel_clean() {
        let temp = TempDir::new().unwrap();
        let file1 = temp.path().join("file1.txt");
        let file2 = temp.path().join("file2.txt");

        fs::write(&file1, "clean content").unwrap();
        fs::write(&file2, "also clean").unwrap();

        let files = vec![file1, file2];
        let (violations, errors) = scan_files_parallel(files, Some(2));

        assert_eq!(violations.len(), 0);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_scan_files_parallel_with_violations() {
        let temp = TempDir::new().unwrap();
        let file1 = temp.path().join("file1.txt");
        let file2 = temp.path().join("file2.txt");

        fs::write(&file1, "clean content").unwrap();
        // Create file with zero-width space
        let zwsp = char::from_u32(0x200B).unwrap();
        fs::write(&file2, format!("bad{}", zwsp)).unwrap();

        let files = vec![file1, file2];
        let (violations, errors) = scan_files_parallel(files, Some(2));

        assert_eq!(violations.len(), 1);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_scan_files_parallel_with_errors() {
        let files = vec![PathBuf::from("/nonexistent/file.txt")];
        let (violations, errors) = scan_files_parallel(files, Some(1));

        assert_eq!(violations.len(), 0);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_scan_files_parallel_empty() {
        let files = vec![];
        let (violations, errors) = scan_files_parallel(files, Some(1));

        assert_eq!(violations.len(), 0);
        assert_eq!(errors.len(), 0);
    }
}
