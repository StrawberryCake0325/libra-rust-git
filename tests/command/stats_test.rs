use std::fs;

use tempfile::tempdir;

use super::*;

#[test]
fn test_stats_counts_extensions_in_workdir() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path();

    fs::write(dir.join("main.rs"), "fn main() {}").unwrap();
    fs::write(dir.join("lib.rs"), "pub fn foo() {}").unwrap();
    fs::write(dir.join("index.js"), "console.log(1)").unwrap();
    fs::write(dir.join("style.css"), "body { margin: 0 }").unwrap();
    fs::write(dir.join("LICENSE"), "MIT").unwrap();
    fs::write(dir.join("Makefile"), "all:").unwrap();

    let output = run_libra_command(&["stats"], dir);
    assert_cli_success(&output, "stats");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("rs"), "should contain rs extension");
    assert!(stdout.contains("js"), "should contain js extension");
    assert!(stdout.contains("css"), "should contain css extension");
    assert!(stdout.contains("no_extension"), "should contain no_extension");
}

#[test]
fn test_stats_json_output() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path();

    fs::write(dir.join("a.rs"), "").unwrap();
    fs::write(dir.join("b.rs"), "").unwrap();
    fs::write(dir.join("c.py"), "").unwrap();

    let output = run_libra_command(&["--json", "stats"], dir);
    assert_cli_success(&output, "stats --json");

    let parsed = parse_json_stdout(&output);
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["command"], "stats");

    let exts = &parsed["data"]["extensions"];
    assert_eq!(exts["rs"].as_u64(), Some(2));
    assert_eq!(exts["py"].as_u64(), Some(1));
}

#[test]
fn test_stats_empty_directory() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path();

    let output = run_libra_command(&["stats"], dir);
    assert_cli_success(&output, "stats empty dir");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty(), "empty directory should produce no output");
}

#[test]
fn test_stats_ignores_libra_and_target_directories() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path();

    fs::write(dir.join("real.rs"), "").unwrap();
    fs::write(dir.join("real.py"), "").unwrap();

    let libra_dir = dir.join(".libra");
    fs::create_dir_all(&libra_dir).unwrap();
    fs::write(libra_dir.join("config.toml"), "[core]").unwrap();
    fs::write(libra_dir.join("data.db"), b"\x00\x01").unwrap();

    let target_dir = dir.join("target");
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(target_dir.join("output.bin"), b"\x00\x01").unwrap();
    fs::write(target_dir.join("debug.log"), "debug").unwrap();

    let output = run_libra_command(&["stats"], dir);
    assert_cli_success(&output, "stats ignore dirs");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("rs"), "should count files outside ignored dirs");
    assert!(stdout.contains("py"), "should count files outside ignored dirs");
    assert!(!stdout.contains("toml"), "should NOT count .libra files");
    assert!(!stdout.contains("bin"), "should NOT count target files");
    assert!(!stdout.contains("debug"), "should NOT count target files");
}
