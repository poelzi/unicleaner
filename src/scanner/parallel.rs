//! Parallel file scanning with rayon

use crate::config::Configuration;
use crate::report::{ScanError, Violation};
use crate::scanner::encoding::DetectedEncoding;
use crate::scanner::file_scanner::scan_file_with_config;
use rayon::prelude::*;
use std::path::PathBuf;

/// Scan multiple files in parallel
pub fn scan_files_parallel(
    files: Vec<PathBuf>,
    num_threads: Option<usize>,
    config: &Configuration,
    encoding_override: Option<DetectedEncoding>,
) -> (Vec<Violation>, Vec<ScanError>) {
    let scan_closure = || {
        files
            .par_iter()
            .fold(
                || (Vec::new(), Vec::new()),
                |mut acc, file| {
                    match scan_file_with_config(file, config, encoding_override) {
                        Ok(file_violations) => {
                            acc.0.extend(file_violations);
                        }
                        Err(e) => {
                            let error_type = classify_error(&e);
                            acc.1
                                .push(ScanError::new(file.clone(), error_type, e.to_string()));
                        }
                    }
                    acc
                },
            )
            .reduce(
                || (Vec::new(), Vec::new()),
                |mut a, b| {
                    a.0.extend(b.0);
                    a.1.extend(b.1);
                    a
                },
            )
    };

    // Use a local thread pool if specified, otherwise use the global pool
    let mut result = if let Some(threads) = num_threads {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("Failed to build thread pool");
        pool.install(scan_closure)
    } else {
        scan_closure()
    };

    result.0.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then(a.line.cmp(&b.line))
            .then(a.column.cmp(&b.column))
            .then(a.code_point.cmp(&b.code_point))
    });

    result.1.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then(error_type_rank(a.error_type).cmp(&error_type_rank(b.error_type)))
            .then(a.message.cmp(&b.message))
    });

    result
}

/// Classify an error into the appropriate ErrorType
fn classify_error(error: &crate::Error) -> crate::report::violation::ErrorType {
    use crate::report::violation::ErrorType;

    match error {
        crate::Error::Encoding(_) => ErrorType::EncodingError,
        crate::Error::Io(io_err) => {
            if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                ErrorType::PermissionDenied
            } else {
                ErrorType::IoError
            }
        }
        crate::Error::Config(_) => ErrorType::ParseError,
        _ => ErrorType::IoError,
    }
}

fn error_type_rank(error_type: crate::report::violation::ErrorType) -> u8 {
    use crate::report::violation::ErrorType;

    match error_type {
        ErrorType::PermissionDenied => 0,
        ErrorType::IoError => 1,
        ErrorType::EncodingError => 2,
        ErrorType::ParseError => 3,
    }
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
        let config = Configuration::default();
        let (violations, errors) = scan_files_parallel(files, Some(2), &config, None);

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
        let config = Configuration::default();
        let (violations, errors) = scan_files_parallel(files, Some(2), &config, None);

        assert_eq!(violations.len(), 1);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_scan_files_parallel_with_errors() {
        let files = vec![PathBuf::from("/nonexistent/file.txt")];
        let config = Configuration::default();
        let (violations, errors) = scan_files_parallel(files, Some(1), &config, None);

        assert_eq!(violations.len(), 0);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_scan_files_parallel_empty() {
        let files = vec![];
        let config = Configuration::default();
        let (violations, errors) = scan_files_parallel(files, Some(1), &config, None);

        assert_eq!(violations.len(), 0);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_scan_files_parallel_no_thread_count() {
        // Tests the None branch (global rayon pool)
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        fs::write(&file, "clean content").unwrap();

        let config = Configuration::default();
        let (violations, errors) = scan_files_parallel(vec![file], None, &config, None);

        assert_eq!(violations.len(), 0);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_classify_error_encoding() {
        let e = crate::Error::Encoding("bad encoding".to_string());
        assert_eq!(
            classify_error(&e),
            crate::report::violation::ErrorType::EncodingError
        );
    }

    #[test]
    fn test_classify_error_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let e = crate::Error::Io(io_err);
        assert_eq!(
            classify_error(&e),
            crate::report::violation::ErrorType::PermissionDenied
        );
    }

    #[test]
    fn test_classify_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let e = crate::Error::Io(io_err);
        assert_eq!(
            classify_error(&e),
            crate::report::violation::ErrorType::IoError
        );
    }

    #[test]
    fn test_classify_error_config() {
        let e = crate::Error::Config("bad config".to_string());
        assert_eq!(
            classify_error(&e),
            crate::report::violation::ErrorType::ParseError
        );
    }

    #[test]
    fn test_classify_error_other() {
        let e = crate::Error::Git("git error".to_string());
        assert_eq!(
            classify_error(&e),
            crate::report::violation::ErrorType::IoError
        );
    }

    #[test]
    fn test_error_type_rank_ordering() {
        use crate::report::violation::ErrorType;
        assert!(error_type_rank(ErrorType::PermissionDenied) < error_type_rank(ErrorType::IoError));
        assert!(error_type_rank(ErrorType::IoError) < error_type_rank(ErrorType::EncodingError));
        assert!(error_type_rank(ErrorType::EncodingError) < error_type_rank(ErrorType::ParseError));
    }

    #[test]
    fn test_scan_parallel_sorts_violations() {
        let temp = TempDir::new().unwrap();
        let file_b = temp.path().join("b.txt");
        let file_a = temp.path().join("a.txt");

        let zwsp = char::from_u32(0x200B).unwrap();
        fs::write(&file_b, format!("bad{}", zwsp)).unwrap();
        fs::write(&file_a, format!("bad{}", zwsp)).unwrap();

        let files = vec![file_b, file_a.clone()];
        let config = Configuration::default();
        let (violations, _) = scan_files_parallel(files, Some(1), &config, None);

        assert!(violations.len() >= 2);
        // Should be sorted by file path: a.txt before b.txt
        assert_eq!(violations[0].file_path, file_a);
    }

    #[test]
    fn test_scan_parallel_multiple_errors_sorted() {
        let files = vec![
            PathBuf::from("/nonexistent/z_file.txt"),
            PathBuf::from("/nonexistent/a_file.txt"),
        ];
        let config = Configuration::default();
        let (_, errors) = scan_files_parallel(files, Some(1), &config, None);

        assert_eq!(errors.len(), 2);
        // Should be sorted by file path
        assert!(errors[0].file_path < errors[1].file_path);
    }
}
