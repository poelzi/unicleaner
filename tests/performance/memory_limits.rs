//! Memory usage limit tests
//!
//! These tests verify that the scanner stays within acceptable memory limits
//! when processing large repositories and files.

use std::fs;
use tempfile::TempDir;

#[cfg(target_os = "linux")]
use std::io::Read;

/// Get current process memory usage in bytes (Linux only)
#[cfg(target_os = "linux")]
fn get_memory_usage() -> Option<usize> {
    let mut status = String::new();
    fs::File::open("/proc/self/status")
        .ok()?
        .read_to_string(&mut status)
        .ok()?;

    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // VmRSS is in KB, convert to bytes
                return parts[1].parse::<usize>().ok().map(|kb| kb * 1024);
            }
        }
    }
    None
}

/// Get current process memory usage in bytes (macOS)
#[cfg(target_os = "macos")]
fn get_memory_usage() -> Option<usize> {
    use std::process::Command;

    let pid = std::process::id().to_string();

    let output = Command::new("ps")
        .arg("-o")
        .arg("rss=")
        .arg("-p")
        .arg(&pid)
        .output()
        .ok()?;

    let rss_kb = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<usize>()
        .ok()?;

    Some(rss_kb * 1024) // Convert KB to bytes
}

/// Get current process memory usage in bytes (Windows)
#[cfg(target_os = "windows")]
fn get_memory_usage() -> Option<usize> {
    // Windows memory tracking would require winapi
    // For now, return None on Windows (test will be skipped)
    None
}

fn scan_file(path: &std::path::Path) -> Result<(), String> {
    unicleaner::scanner::file_scanner::scan_file(path)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn test_memory_under_500mb_for_large_repo() {
    // Skip test on platforms where we can't measure memory
    let baseline_memory = match get_memory_usage() {
        Some(mem) => mem,
        None => {
            eprintln!("Memory tracking not available on this platform, skipping test");
            return;
        }
    };

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut files = Vec::new();

    // Create a large repository: 5000 files with mixed content
    for i in 0..5000 {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));

        let content = if i % 3 == 0 {
            // Unicode-heavy files
            format!(
                "/* Unicode test {} */\nfn test() {{\n    let x = \"{}\";\n}}\n",
                i,
                "🔥".repeat(100)
            )
        } else if i % 3 == 1 {
            // Regular ASCII files
            format!(
                "// Regular file {}\nfn main() {{\n    println!(\"test\");\n}}\n",
                i
            )
        } else {
            // Mixed content
            format!(
                "// File {}\nlet α = 42; // Greek alpha\nlet Ω = 100; // Omega\n",
                i
            )
        };

        fs::write(&file_path, content).expect("Failed to write file");
        files.push(file_path);
    }

    // Scan all files
    for file in &files {
        let _ = scan_file(file);
    }

    // Check memory usage
    if let Some(current_memory) = get_memory_usage() {
        let memory_used = current_memory.saturating_sub(baseline_memory);
        let memory_mb = memory_used / (1024 * 1024);

        assert!(
            memory_mb < 500,
            "Memory usage should stay under 500MB for large repo, used {}MB",
            memory_mb
        );

        println!("Memory used for 5000-file scan: {}MB", memory_mb);
    }
}

#[test]
fn test_memory_stable_across_multiple_scans() {
    // Skip test on platforms where we can't measure memory
    let baseline_memory = match get_memory_usage() {
        Some(mem) => mem,
        None => {
            eprintln!("Memory tracking not available on this platform, skipping test");
            return;
        }
    };

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create 100 test files
    let mut files = Vec::new();
    for i in 0..100 {
        let file_path = temp_dir.path().join(format!("test_{}.rs", i));
        let content = format!(
            "// Test file {}\nfn test_{}() {{\n    let data = \"{}\";\n}}\n",
            i,
            i,
            "test".repeat(50)
        );
        fs::write(&file_path, content).expect("Failed to write file");
        files.push(file_path);
    }

    // Scan the same files 10 times
    for round in 0..10 {
        for file in &files {
            let _ = scan_file(file);
        }

        if let Some(current_memory) = get_memory_usage() {
            let memory_used = current_memory.saturating_sub(baseline_memory);
            let memory_mb = memory_used / (1024 * 1024);

            // Memory should not grow significantly across scans (no leaks)
            assert!(
                memory_mb < 300,
                "Memory grew unexpectedly on scan round {}: {}MB",
                round + 1,
                memory_mb
            );
        }
    }
}

#[test]
fn test_memory_single_large_file() {
    // Skip test on platforms where we can't measure memory
    let baseline_memory = match get_memory_usage() {
        Some(mem) => mem,
        None => {
            eprintln!("Memory tracking not available on this platform, skipping test");
            return;
        }
    };

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("large_file.txt");

    // Create a 50MB file
    let mut content = String::with_capacity(50 * 1024 * 1024);
    for i in 0..100000 {
        content.push_str(&format!(
            "// Line {} with some Unicode: α β γ δ ε ζ η θ\n",
            i
        ));
    }

    fs::write(&file_path, content).expect("Failed to write large file");

    // Scan the large file
    let _ = scan_file(&file_path);

    // Check memory usage
    if let Some(current_memory) = get_memory_usage() {
        let memory_used = current_memory.saturating_sub(baseline_memory);
        let memory_mb = memory_used / (1024 * 1024);

        // Memory includes file content plus scanner data structures
        assert!(
            memory_mb < 500,
            "Memory usage for 50MB file should be reasonable, used {}MB",
            memory_mb
        );

        println!("Memory used for 50MB file scan: {}MB", memory_mb);
    }
}

#[test]
fn test_memory_many_violations() {
    // Skip test on platforms where we can't measure memory
    let baseline_memory = match get_memory_usage() {
        Some(mem) => mem,
        None => {
            eprintln!("Memory tracking not available on this platform, skipping test");
            return;
        }
    };

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("many_violations.txt");

    // Create a file with many potential violations
    let mut content = String::new();
    for i in 0..10000 {
        // Mix of various Unicode attacks
        content.push_str(&format!(
            "let var{} = \"test\"; // αβγ Cyrillic: а е о\n",
            i
        ));
        content.push_str("let x = \u{202E}reversed\u{202D};\n"); // Bidi override
        content.push_str("let z\u{200B}w\u{200C}k\u{200D} = 42;\n"); // Zero-width chars
    }

    fs::write(&file_path, &content).expect("Failed to write file");

    // Scan file with many violations
    let _ = scan_file(&file_path);

    // Check memory usage
    if let Some(current_memory) = get_memory_usage() {
        let memory_used = current_memory.saturating_sub(baseline_memory);
        let memory_mb = memory_used / (1024 * 1024);

        // Real scanning with many violations uses more memory for violation storage
        assert!(
            memory_mb < 500,
            "Memory usage for file with many violations should be reasonable, used {}MB",
            memory_mb
        );

        println!("Memory used for file with many violations: {}MB", memory_mb);
    }
}
