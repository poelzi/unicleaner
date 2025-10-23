use std::path::PathBuf;
use std::process;
use std::time::Instant;
use unicleaner::cli::args::{Args, OutputFormat};
use unicleaner::report::formatter::format_human;
use unicleaner::report::ScanResult;
use unicleaner::scanner::parallel::scan_files_parallel;
use unicleaner::scanner::walker::{walk_paths, WalkConfig};

fn main() {
    // Parse command line arguments
    let args = Args::parse_args();

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        process::exit(2);
    }

    // Run the scan
    let exit_code = match run_scan(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Fatal error: {}", e);
            2
        }
    };

    process::exit(exit_code);
}

fn run_scan(args: &Args) -> Result<i32, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Configure directory walker
    let walk_config = WalkConfig {
        follow_links: false,
        respect_gitignore: true,
        respect_hidden: true,
        max_depth: None,
        threads: args.jobs.unwrap_or_else(num_cpus::get),
    };

    // Collect files to scan
    if args.verbose {
        eprintln!("Collecting files to scan...");
    }

    let files = walk_paths(&args.paths, &walk_config)?;

    if args.verbose {
        eprintln!("Found {} files to scan", files.len());
    }

    if files.is_empty() {
        eprintln!("Warning: No files found to scan");
        return Ok(0);
    }

    // Scan files in parallel
    if args.verbose {
        eprintln!("Scanning files...");
    }

    let (violations, errors) = scan_files_parallel(files.clone(), args.jobs);

    // Calculate statistics
    let files_scanned = files.len();
    let files_with_violations = violations
        .iter()
        .map(|v| &v.file_path)
        .collect::<std::collections::HashSet<_>>()
        .len();
    let files_clean = files_scanned - files_with_violations - errors.len();

    // Build scan result
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from("unicleaner.toml"));
    let result = ScanResult {
        violations,
        files_scanned,
        files_clean,
        files_with_violations,
        errors,
        duration: start_time.elapsed(),
        config_used: config_path,
    };

    // Format and display output
    let use_color = !args.no_color && supports_color::on(supports_color::Stream::Stdout).is_some();

    match args.format {
        OutputFormat::Human => {
            let output = format_human(&result, use_color, args.verbose);
            if !args.quiet {
                print!("{}", output);
            } else {
                // In quiet mode, only show summary
                let lines: Vec<&str> = output.lines().collect();
                if let Some(summary_start) = lines
                    .iter()
                    .position(|l| l.starts_with("\nScan Result:") || l.starts_with("Scan Result:"))
                {
                    for line in &lines[summary_start..] {
                        println!("{}", line);
                    }
                }
            }
        }
        OutputFormat::Json => {
            // JSON output will be implemented in Phase 6
            eprintln!("JSON output format not yet implemented");
            return Ok(2);
        }
        OutputFormat::Github => {
            // GitHub Actions output will be implemented in Phase 5
            eprintln!("GitHub Actions output format not yet implemented");
            return Ok(2);
        }
        OutputFormat::Gitlab => {
            // GitLab CI output will be implemented in Phase 5
            eprintln!("GitLab CI output format not yet implemented");
            return Ok(2);
        }
    }

    Ok(result.exit_code())
}
