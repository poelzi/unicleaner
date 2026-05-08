//! Handler for the `unicleaner clean` subcommand.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::cleaner::{CleanAction, CleanPolicy, clean};
use crate::cli::args::CleanPolicyPreset;
use crate::cli::exit_codes;
use crate::config::Configuration;

/// Run the `clean` subcommand.
///
/// Reads from `path` (or stdin when `path` is `None` or `-`), runs
/// [`crate::cleaner::clean`] with the resolved policy, then either:
///
/// - writes the cleaned content to stdout (default), or
/// - rewrites the file in place atomically (`--in-place`): write to a
///   temp sibling, fsync, rename onto the original path.
///
/// When `config_path` is set, the policy from the config's `[cleaner]`
/// block (if present) takes precedence over `preset`. The `--nfc` CLI
/// flag still applies on top.
///
/// Returns an exit code per [`exit_codes`].
pub fn run(
    path: Option<PathBuf>,
    in_place: bool,
    preset: CleanPolicyPreset,
    nfc: bool,
    config_path: Option<&Path>,
) -> i32 {
    let policy = match resolve_policy(preset, nfc, config_path) {
        Ok(p) => p,
        Err(code) => return code,
    };

    // Resolve input source.
    let read_from_stdin = match path.as_deref() {
        None => true,
        Some(p) => p.as_os_str() == "-",
    };

    if in_place && read_from_stdin {
        eprintln!("Error: --in-place requires a file path, not stdin");
        return exit_codes::ERROR;
    }

    let (input, source_path): (String, Option<PathBuf>) = if read_from_stdin {
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("Error: failed to read stdin: {}", e);
            return exit_codes::ERROR;
        }
        (buf, None)
    } else {
        let p = path.expect("path is Some when not reading from stdin");
        match std::fs::read_to_string(&p) {
            Ok(s) => (s, Some(p)),
            Err(e) => {
                eprintln!("Error: failed to read '{}': {}", p.display(), e);
                return exit_codes::ERROR;
            }
        }
    };

    let result = clean(&input, &policy);

    let report_only = is_report_only_policy(&policy);

    if in_place {
        let target = source_path.expect("source_path is Some for in-place mode");
        if let Err(e) = atomic_write(&target, result.output.as_bytes()) {
            eprintln!("Error: failed to rewrite '{}': {}", target.display(), e);
            return exit_codes::ERROR;
        }
        if report_only && !result.violations.is_empty() {
            return exit_codes::VIOLATIONS_FOUND;
        }
        return exit_codes::SUCCESS;
    }

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    if let Err(e) = handle.write_all(result.output.as_bytes()) {
        eprintln!("Error: failed to write stdout: {}", e);
        return exit_codes::ERROR;
    }

    // Report-only mode that found violations: signal with VIOLATIONS_FOUND.
    if report_only && !result.violations.is_empty() {
        return exit_codes::VIOLATIONS_FOUND;
    }

    exit_codes::SUCCESS
}

fn is_report_only_policy(policy: &CleanPolicy) -> bool {
    policy.default_action == CleanAction::KeepWithMark
        && policy
            .per_category
            .values()
            .all(|action| *action == CleanAction::KeepWithMark)
}

fn resolve_policy(
    preset: CleanPolicyPreset,
    nfc: bool,
    config_path: Option<&Path>,
) -> Result<CleanPolicy, i32> {
    if let Some(p) = config_path {
        match Configuration::from_file(p) {
            Ok(cfg) => {
                if let Some(policy) = cfg.cleaner {
                    let want_nfc = nfc || policy.normalize_nfc;
                    return Ok(policy.with_nfc(want_nfc));
                }
            }
            Err(e) => {
                eprintln!("Error: failed to load config '{}': {}", p.display(), e);
                return Err(exit_codes::ERROR);
            }
        }
    }

    let p = match preset {
        CleanPolicyPreset::Strict => CleanPolicy::strict(),
        CleanPolicyPreset::Lossy => CleanPolicy::lossy(),
        CleanPolicyPreset::ReportOnly => CleanPolicy::report_only(),
    };
    Ok(p.with_nfc(nfc))
}

/// Atomically write `bytes` to `target`: write to `<target>.tmp.<pid>`,
/// fsync, then rename. Removes the temp file on error.
fn atomic_write(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let permissions = std::fs::metadata(target)?.permissions();

    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let file_name = target
        .file_name()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "target path has no file name",
            )
        })?
        .to_owned();

    let mut tmp_name = file_name.clone();
    tmp_name.push(format!(".tmp.{}", std::process::id()));
    let tmp_path = parent.join(&tmp_name);

    let write_result = (|| -> std::io::Result<()> {
        let mut tmp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)?;
        tmp.set_permissions(permissions)?;
        tmp.write_all(bytes)?;
        tmp.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_result {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e);
    }

    if let Err(e) = std::fs::rename(&tmp_path, target) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e);
    }

    // Best-effort fsync the directory entry.
    if let Ok(dir) = File::open(&parent) {
        let _ = dir.sync_all();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cleaner::{CleanAction, CleanPolicy};
    use crate::unicode::malicious::MaliciousCategory;

    #[test]
    fn resolve_policy_preset_strict() {
        let p = resolve_policy(CleanPolicyPreset::Strict, false, None).unwrap();
        assert_eq!(p.default_action, CleanAction::Strip);
        assert!(!p.normalize_nfc);
    }

    #[test]
    fn resolve_policy_preset_lossy_with_nfc() {
        let p = resolve_policy(CleanPolicyPreset::Lossy, true, None).unwrap();
        assert_eq!(p.default_action, CleanAction::Replace('\u{FFFD}'));
        assert!(p.normalize_nfc);
    }

    #[test]
    fn resolve_policy_preset_report_only() {
        let p = resolve_policy(CleanPolicyPreset::ReportOnly, false, None).unwrap();
        assert_eq!(p.default_action, CleanAction::KeepWithMark);
    }

    #[test]
    fn is_report_only_policy_detects_keep_with_mark_default() {
        assert!(is_report_only_policy(&CleanPolicy::report_only()));
    }

    #[test]
    fn is_report_only_policy_rejects_strict() {
        assert!(!is_report_only_policy(&CleanPolicy::strict()));
    }

    #[test]
    fn is_report_only_policy_rejects_mixed_overrides() {
        // KeepWithMark default but one override that mutates → not report-only.
        let p = CleanPolicy::report_only()
            .with_action(MaliciousCategory::ZeroWidth, CleanAction::Strip);
        assert!(!is_report_only_policy(&p));
    }

    #[test]
    fn is_report_only_policy_accepts_all_keep_overrides() {
        let p = CleanPolicy::report_only()
            .with_action(MaliciousCategory::ZeroWidth, CleanAction::KeepWithMark);
        assert!(is_report_only_policy(&p));
    }

    #[test]
    fn resolve_policy_uses_preset_without_config() {
        let p = resolve_policy(CleanPolicyPreset::Lossy, false, None).unwrap();
        assert_eq!(p.default_action, CleanAction::Replace('\u{FFFD}'));
    }

    #[test]
    fn resolve_policy_falls_back_when_config_lacks_cleaner_block() {
        let dir = tempfile::TempDir::new().unwrap();
        let cfg = dir.path().join("unicleaner.toml");
        std::fs::write(&cfg, "[global]\ndeny_by_default = false\n").unwrap();

        let p = resolve_policy(CleanPolicyPreset::Lossy, false, Some(&cfg)).unwrap();
        // No `[cleaner]` block → preset wins.
        assert_eq!(p.default_action, CleanAction::Replace('\u{FFFD}'));
    }

    #[test]
    fn resolve_policy_uses_config_cleaner_block() {
        let dir = tempfile::TempDir::new().unwrap();
        let cfg = dir.path().join("unicleaner.toml");
        std::fs::write(
            &cfg,
            "[cleaner]\ndefault_action = { kind = \"keep_with_mark\" }\n",
        )
        .unwrap();

        let p = resolve_policy(CleanPolicyPreset::Strict, false, Some(&cfg)).unwrap();
        assert_eq!(p.default_action, CleanAction::KeepWithMark);
    }

    #[test]
    fn resolve_policy_errors_on_invalid_config() {
        let dir = tempfile::TempDir::new().unwrap();
        let cfg = dir.path().join("nope.toml");
        // File doesn't exist → expect an exit-code error.
        let r = resolve_policy(CleanPolicyPreset::Strict, false, Some(&cfg));
        assert_eq!(r.unwrap_err(), exit_codes::ERROR);
    }

    #[test]
    fn atomic_write_replaces_file_contents() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("data.txt");
        std::fs::write(&target, b"original\n").unwrap();

        atomic_write(&target, b"replaced\n").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"replaced\n");
    }

    #[test]
    fn atomic_write_errors_on_missing_target() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("does-not-exist.txt");
        // metadata() fails because the target doesn't exist yet.
        let err = atomic_write(&target, b"x").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}
