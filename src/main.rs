use std::path::PathBuf;
use std::process;
use std::time::Instant;
use unicleaner::cli::args::{Args, Command, OutputFormat};
use unicleaner::cli::output::{should_use_color, ColorStream};
use unicleaner::config::presets;
use unicleaner::report::formatter::format_human;
use unicleaner::report::json::{format_json, format_json_compact};
use unicleaner::report::ScanResult;
use unicleaner::scanner::git_diff;
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

    // Execute command
    let exit_code = match args.get_command() {
        Command::Scan { .. } => match run_scan(&args) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("Fatal error: {}", e);
                2
            }
        },
        Command::Init { output, force } => match run_init(&output, force) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("Error: {}", e);
                1
            }
        },
        Command::ListPresets => {
            run_list_presets();
            0
        }
    };

    process::exit(exit_code);
}

fn run_scan(args: &Args) -> Result<i32, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Extract scan parameters from command
    let (paths, jobs, diff) = match args.get_command() {
        Command::Scan {
            paths, jobs, diff, ..
        } => (paths, jobs, diff),
        _ => unreachable!(),
    };

    // Configure directory walker
    let walk_config = WalkConfig {
        follow_links: false,
        respect_gitignore: true,
        respect_hidden: true,
        max_depth: None,
        threads: jobs.unwrap_or_else(num_cpus::get),
    };

    // Collect files to scan
    if args.verbose {
        eprintln!("Collecting files to scan...");
    }

    let mut files = walk_paths(&paths, &walk_config)?;

    // Filter to only changed files if in diff mode
    if diff {
        if args.verbose {
            eprintln!("Diff mode enabled - scanning only changed files...");
        }

        // Determine repository root (use first path as base)
        let default_path = PathBuf::from(".");
        let repo_path = paths.first().unwrap_or(&default_path);

        if !git_diff::is_git_repository(repo_path) {
            eprintln!("Error: --diff flag requires a Git repository");
            return Ok(2);
        }

        files = git_diff::filter_changed_files(files, repo_path)?;

        if args.verbose {
            eprintln!("Found {} changed files", files.len());
        }
    } else if args.verbose {
        eprintln!("Found {} files to scan", files.len());
    }

    if files.is_empty() {
        if diff {
            if !args.quiet {
                println!("No changed files to scan");
            }
            return Ok(0);
        } else {
            eprintln!("Warning: No files found to scan");
            return Ok(0);
        }
    }

    // Scan files in parallel
    if args.verbose {
        eprintln!("Scanning files...");
    }

    let (violations, errors) = scan_files_parallel(files.clone(), jobs);

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
    let mut result = ScanResult {
        violations,
        files_scanned,
        files_clean,
        files_with_violations,
        errors,
        duration: start_time.elapsed(),
        config_used: config_path,
    };

    // Apply severity filtering if specified
    if let Some(min_severity) = args.severity {
        result = result.filter_by_severity(min_severity.to_severity());
    }

    // Format and display output
    let color_mode = args.get_color_mode();
    let use_color = should_use_color(color_mode, ColorStream::Stdout);

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
            // Use compact JSON for piping, pretty JSON for interactive use
            let json_output = if args.quiet {
                format_json_compact(&result)
            } else {
                format_json(&result)
            };

            match json_output {
                Ok(json) => println!("{}", json),
                Err(e) => {
                    eprintln!("Error formatting JSON output: {}", e);
                    return Ok(2);
                }
            }
        }
        OutputFormat::Github => {
            // GitHub Actions output format: ::error file={},line={},col={}::{}
            for violation in &result.violations {
                println!(
                    "::error file={},line={},col={}::{}",
                    violation.file_path.display(),
                    violation.line,
                    violation.column,
                    violation.message
                );
            }
            if !args.quiet {
                eprintln!(
                    "\nScan complete: {} violations found",
                    result.violations.len()
                );
            }
        }
        OutputFormat::Gitlab => {
            // GitLab CI uses JSON format with specific schema
            // For now, use standard JSON format
            let json_output = format_json(&result);
            match json_output {
                Ok(json) => println!("{}", json),
                Err(e) => {
                    eprintln!("Error formatting GitLab output: {}", e);
                    return Ok(2);
                }
            }
        }
    }

    Ok(result.exit_code())
}

fn run_init(output: &PathBuf, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Check if file exists
    if output.exists() && !force {
        return Err(format!(
            "Configuration file '{}' already exists. Use --force to overwrite.",
            output.display()
        )
        .into());
    }

    // Generate default configuration
    let config_content = r#"# Unicleaner Configuration File
# Deny malicious Unicode by default

deny_by_default = true

# Language presets (uncomment to enable)
# [language_presets]
# rust = "rust-default"
# python = "python-default"
# javascript = "js-default"

# File-specific rules
# [[file_rules]]
# pattern = "**/*.rs"
# allowed_ranges = [
#     { start = 0x0000, end = 0x007F, name = "Basic Latin" }
# ]
# denied_code_points = []
# priority = 1
"#;

    // Write to file
    std::fs::write(output, config_content)?;

    println!("Created default configuration file: {}", output.display());
    println!("Edit this file to customize Unicode detection rules.");

    Ok(())
}

fn run_list_presets() {
    println!("Available Language Presets:\n");

    let all_presets = presets::get_all_presets();
    for (name, preset) in all_presets.iter() {
        println!("  {} - {}", name, preset.description);
        println!("    Allowed ranges: {}", preset.allowed_ranges.len());
        println!();
    }

    println!("Use these presets in your unicleaner.toml:");
    println!("  [language_presets]");
    println!("  rust = \"rust-default\"");
}
