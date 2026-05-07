//! Integration tests for the `unicleaner clean` CLI subcommand.

use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use assert_cmd::cargo_bin_cmd;
use tempfile::TempDir;

const FIXTURE_DIR: &str = "tests/integration/fixtures/cleaner";

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(FIXTURE_DIR).join(name)
}

fn read_bytes(p: &Path) -> Vec<u8> {
    fs::read(p).unwrap_or_else(|_| panic!("fixture missing: {}", p.display()))
}

#[test]
fn cli_clean_stdout_default() {
    let mut cmd = cargo_bin_cmd!("unicleaner");
    let assert = cmd.arg("clean").arg(fixture("zwsp.txt")).assert().success();

    let out = assert.get_output().stdout.clone();
    let expected = read_bytes(&fixture("zwsp.cleaned.txt"));
    assert_eq!(out, expected, "stdout must equal cleaned golden");
}

#[test]
fn cli_clean_clean_file_unchanged() {
    let input_path = fixture("clean.rs");
    let expected = read_bytes(&input_path);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let assert = cmd.arg("clean").arg(&input_path).assert().success();

    let out = assert.get_output().stdout.clone();
    assert_eq!(out, expected, "clean input must round-trip byte-identical");
}

#[test]
fn cli_clean_in_place_atomic() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("zwsp.txt");
    fs::copy(fixture("zwsp.txt"), &target).unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("clean")
        .arg("--in-place")
        .arg(&target)
        .assert()
        .success();

    let cleaned = read_bytes(&target);
    let expected = read_bytes(&fixture("zwsp.cleaned.txt"));
    assert_eq!(cleaned, expected, "in-place rewrite must equal golden");

    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
        .collect();
    assert!(leftovers.is_empty(), "no .tmp files should remain");
}

#[cfg(unix)]
#[test]
fn cli_clean_in_place_preserves_permissions() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("script.sh");
    fs::copy(fixture("zwsp.txt"), &target).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("clean")
        .arg("--in-place")
        .arg(&target)
        .assert()
        .success();

    let mode = fs::metadata(&target).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700, "in-place rewrite must preserve file mode");
}

#[test]
fn cli_clean_missing_file_errors() {
    // We pin to the exact stderr prefix produced by `cli::clean::run` so
    // the test is locale-independent.
    let mut cmd = cargo_bin_cmd!("unicleaner");
    let assert = cmd
        .arg("clean")
        .arg("does/not/exist.txt")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.starts_with("Error: failed to read 'does/not/exist.txt'"),
        "stderr must start with the expected error prefix, got: {}",
        stderr
    );
}

#[test]
fn cli_clean_policy_lossy_flag() {
    let mut cmd = cargo_bin_cmd!("unicleaner");
    let assert = cmd
        .arg("clean")
        .arg("--policy")
        .arg("lossy")
        .arg(fixture("bidi.txt"))
        .assert()
        .success();

    let out = assert.get_output().stdout.clone();
    let expected = read_bytes(&fixture("bidi.lossy.txt"));
    assert_eq!(out, expected);
}

#[test]
fn cli_clean_stdin_dash() {
    let input = read_bytes(&fixture("zwsp.txt"));
    let expected = read_bytes(&fixture("zwsp.cleaned.txt"));

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let assert = cmd
        .arg("clean")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success();

    let out = assert.get_output().stdout.clone();
    assert_eq!(out, expected);
}

#[test]
fn cli_clean_in_place_rejects_stdin() {
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("clean")
        .arg("--in-place")
        .arg("-")
        .write_stdin("hi")
        .assert()
        .failure();
}

#[test]
fn cli_clean_report_only_signals_violations_found() {
    let input = read_bytes(&fixture("zwsp.txt"));

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let assert = cmd
        .arg("clean")
        .arg("--policy")
        .arg("report-only")
        .arg(fixture("zwsp.txt"))
        .assert()
        .code(1);

    let out = assert.get_output().stdout.clone();
    assert_eq!(out, input, "report-only must not mutate stdout");
}

#[test]
fn cli_clean_config_overrides_preset() {
    // Config picks the lossy preset; without --policy we get FFFD substitution.
    let dir = TempDir::new().unwrap();
    let cfg = dir.path().join("unicleaner.toml");
    fs::write(
        &cfg,
        r#"
[cleaner]
default_action = { kind = "replace", value = "�" }
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let assert = cmd
        .arg("--config")
        .arg(&cfg)
        .arg("clean")
        .arg(fixture("bidi.txt"))
        .assert()
        .success();

    let out = assert.get_output().stdout.clone();
    let expected = read_bytes(&fixture("bidi.lossy.txt"));
    assert_eq!(out, expected);
}

#[test]
fn cli_clean_config_report_only_signals_violations_found() {
    let dir = TempDir::new().unwrap();
    let cfg = dir.path().join("unicleaner.toml");
    fs::write(
        &cfg,
        r#"
[cleaner]
default_action = { kind = "keep_with_mark" }
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let assert = cmd
        .arg("--config")
        .arg(&cfg)
        .arg("clean")
        .arg(fixture("zwsp.txt"))
        .assert()
        .code(1);

    let out = assert.get_output().stdout.clone();
    let expected = read_bytes(&fixture("zwsp.txt"));
    assert_eq!(out, expected, "config report-only must not mutate stdout");
}

#[test]
fn cli_clean_config_per_category_override() {
    // Strip default but replace bidi with '?' via per-category override.
    let dir = TempDir::new().unwrap();
    let cfg = dir.path().join("unicleaner.toml");
    fs::write(
        &cfg,
        r#"
[cleaner]
default_action = { kind = "strip" }

[cleaner.per_category]
BidiOverride = { kind = "replace", value = "?" }
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let assert = cmd
        .arg("--config")
        .arg(&cfg)
        .arg("clean")
        .arg(fixture("bidi.txt"))
        .assert()
        .success();

    let out = assert.get_output().stdout.clone();
    assert_eq!(out, b"admin?lortnoc\n");
}
