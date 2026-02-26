use std::path::PathBuf;
use std::process;
use std::time::Instant;
use unicleaner::cli::args::{Args, Command, OutputFormat};
use unicleaner::cli::output::{ColorStream, should_use_color};
use unicleaner::config::Configuration;
use unicleaner::config::presets;
use unicleaner::report::ScanResult;
use unicleaner::report::formatter::format_human;
use unicleaner::report::json::{format_json, format_json_compact};
use unicleaner::report::markdown::format_markdown;
use unicleaner::scanner::git_diff;
use unicleaner::scanner::parallel::scan_files_parallel;
use unicleaner::scanner::walker::{WalkConfig, walk_paths};
use unicleaner::unicode::blocks::BlockRegistry;

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
        Command::ListBlocks { filter } => {
            run_list_blocks(filter.as_deref());
            0
        }
    };

    process::exit(exit_code);
}

fn run_scan(args: &Args) -> Result<i32, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Extract scan parameters from command
    let (paths, jobs, diff, encoding) = match args.get_command() {
        Command::Scan {
            paths,
            jobs,
            diff,
            encoding,
        } => (paths, jobs, diff, encoding),
        _ => unreachable!(),
    };

    // Convert encoding option to DetectedEncoding
    let encoding_override = encoding.map(|e| e.to_detected_encoding());

    // Load configuration
    let config = if let Some(ref config_path) = args.config {
        // Explicit --config flag: load or error
        if !config_path.exists() {
            eprintln!(
                "Error: Configuration file '{}' not found",
                config_path.display()
            );
            return Ok(2);
        }
        match Configuration::from_file(config_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Error: Failed to load configuration '{}': {}",
                    config_path.display(),
                    e
                );
                return Ok(2);
            }
        }
    } else {
        // Auto-discover unicleaner.toml in CWD
        let default_config = PathBuf::from("unicleaner.toml");
        if default_config.exists() {
            match Configuration::from_file(&default_config) {
                Ok(c) => {
                    if args.verbose {
                        eprintln!("Loaded configuration from unicleaner.toml");
                    }
                    c
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load unicleaner.toml: {}", e);
                    Configuration::default()
                }
            }
        } else {
            Configuration::default()
        }
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

    let files = if diff {
        if args.verbose {
            eprintln!("Diff mode enabled - scanning only changed files...");
        }

        // Validate provided paths exist (match walker behavior)
        for path in &paths {
            if !path.is_file() && !path.is_dir() {
                return Err(unicleaner::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Path not found: {}", path.display()),
                ))
                .into());
            }
        }

        let default_path = PathBuf::from(".");
        let repo_hint = paths.first().unwrap_or(&default_path);
        let repo_root = match git_diff::get_repo_root(repo_hint) {
            Ok(root) => root,
            Err(_) => {
                eprintln!("Error: --diff flag requires a Git repository");
                return Ok(2);
            }
        };

        // Canonicalize roots so starts_with comparisons are stable.
        let allowed_roots: Vec<PathBuf> = paths
            .iter()
            .map(|p| p.canonicalize().unwrap_or_else(|_| p.clone()))
            .collect();

        let changed_files = git_diff::get_changed_files(&repo_root)?;
        let mut files: Vec<PathBuf> = changed_files
            .into_iter()
            .filter(|p| p.is_file())
            .filter(|p| {
                let canonical = p.canonicalize().unwrap_or_else(|_| p.clone());
                allowed_roots.iter().any(|root| {
                    if root.is_dir() {
                        canonical.starts_with(root)
                    } else {
                        canonical == *root
                    }
                })
            })
            .collect();

        files.sort();

        if args.verbose {
            eprintln!("Found {} changed files", files.len());
        }

        files
    } else {
        let files = walk_paths(&paths, &walk_config)?;
        if args.verbose {
            eprintln!("Found {} files to scan", files.len());
        }
        files
    };

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

    let (violations, errors) = scan_files_parallel(files.clone(), jobs, &config, encoding_override);

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
        OutputFormat::Markdown => {
            let output = format_markdown(&result, args.verbose);
            if !args.quiet {
                print!("{}", output);
            } else {
                // In quiet mode, only show summary table
                let lines: Vec<&str> = output.lines().collect();
                if let Some(summary_start) = lines.iter().position(|l| l.starts_with("## Summary"))
                {
                    for line in &lines[summary_start..] {
                        println!("{}", line);
                    }
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
# Detect malicious Unicode characters in source code

# Global Settings
[global]
# Deny-by-default security model: only explicitly allowed characters pass
deny_by_default = true

# Language-Specific Presets
# Map file extensions to built-in language presets
# Available presets: rust, javascript, typescript, python, java, c, cpp, go, ruby, php
#
# [languages.rs]
# preset = "rust"
#
# [languages.py]
# preset = "python"
#
# [languages.js]
# preset = "javascript"

# File-Specific Rules
# Rules use glob patterns and are applied in priority order

# Example: Allow extended Latin in documentation
# [[rules]]
# pattern = "docs/**/*.md"
# allowed_blocks = ["Basic Latin", "Latin-1 Supplement"]

# Example: Strict ASCII-only for security-critical code
# [[rules]]
# pattern = "src/auth/**/*.rs"
# allowed_blocks = ["ascii"]
"#;

    // Write to file
    std::fs::write(output, config_content)?;

    println!("Created default configuration file: {}", output.display());
    println!("Edit this file to customize Unicode detection rules.");

    Ok(())
}

fn run_list_blocks(filter: Option<&str>) {
    let blocks = BlockRegistry::list_blocks(filter);

    if blocks.is_empty() {
        if let Some(f) = filter {
            println!("No Unicode blocks matching \"{}\"", f);
        } else {
            println!("No Unicode blocks found");
        }
        return;
    }

    println!(
        "Unicode Blocks (Unicode {}){}\n",
        unicode_blocks::VERSION,
        filter
            .map(|f| format!(", filter: \"{}\"", f))
            .unwrap_or_default()
    );
    println!("  {:<45} {:<20} Aliases", "Name", "Range");
    println!("  {}", "-".repeat(80));

    for block in &blocks {
        let range = format!("U+{:04X}-U+{:04X}", block.start, block.end);
        let aliases = if block.aliases.is_empty() {
            String::new()
        } else {
            block.aliases.join(", ")
        };
        println!("  {:<45} {:<20} {}", block.name, range, aliases);
    }

    println!("\n  Total: {} blocks", blocks.len());
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
    println!("  [languages.rs]");
    println!("  preset = \"rust\"");
    println!();
    println!("  [languages.js]");
    println!("  preset = \"javascript\"");
}
