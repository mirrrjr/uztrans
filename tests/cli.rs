//! End-to-end tests that invoke the actual `uztrans` binary, covering the
//! CLI surface (stdin/stdout, file args, --in-place, --dry-run, --output,
//! --recursive, --include/--exclude) rather than the internal library
//! functions (those are covered by unit tests in `src/`).

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cmd() -> Command {
    Command::cargo_bin("uztrans").unwrap()
}

#[test]
fn stdin_to_stdout() {
    cmd()
        .write_stdin("Toshkent shahri")
        .assert()
        .success()
        .stdout("Toşkent şahri");
}

#[test]
fn stdin_with_no_changes_passes_through() {
    cmd()
        .write_stdin("hello world")
        .assert()
        .success()
        .stdout("hello world");
}

#[test]
fn single_file_written_to_stdout() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.md");
    fs::write(&path, "Bu shahar haqida.").unwrap();

    cmd()
        .arg(&path)
        .assert()
        .success()
        .stdout("Bu şahar haqida.");
}

#[test]
fn in_place_edits_the_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.md");
    fs::write(&path, "Bu shahar haqida.").unwrap();

    cmd().arg("--in-place").arg(&path).assert().success();

    let contents = fs::read_to_string(&path).unwrap();
    assert_eq!(contents, "Bu şahar haqida.");
}

#[test]
fn dry_run_does_not_modify_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.md");
    fs::write(&path, "Bu shahar haqida.").unwrap();

    cmd()
        .arg("--in-place")
        .arg("--dry-run")
        .arg(&path)
        .assert()
        .success();

    let contents = fs::read_to_string(&path).unwrap();
    assert_eq!(
        contents, "Bu shahar haqida.",
        "dry-run must not touch the file"
    );
}

#[test]
fn output_flag_writes_to_new_file() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in.md");
    let output = dir.path().join("out.md");
    fs::write(&input, "shahar").unwrap();

    cmd().arg(&input).arg("-o").arg(&output).assert().success();

    assert_eq!(fs::read_to_string(&output).unwrap(), "şahar");
    assert_eq!(
        fs::read_to_string(&input).unwrap(),
        "shahar",
        "original file must be untouched when -o is used"
    );
}

#[test]
fn in_place_and_output_conflict() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.md");
    fs::write(&path, "shahar").unwrap();

    cmd()
        .arg("--in-place")
        .arg("-o")
        .arg(dir.path().join("out.md"))
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("in-place"));
}

#[test]
fn in_place_with_stdin_is_rejected() {
    cmd()
        .arg("--in-place")
        .write_stdin("shahar")
        .assert()
        .failure();
}

#[test]
fn recursive_processes_nested_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("top.md"), "shahar").unwrap();
    fs::create_dir(dir.path().join("nested")).unwrap();
    fs::write(dir.path().join("nested/deep.md"), "shahar").unwrap();

    cmd()
        .arg("--in-place")
        .arg("--recursive")
        .arg(dir.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("top.md")).unwrap(),
        "şahar"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("nested/deep.md")).unwrap(),
        "şahar"
    );
}

#[test]
fn non_recursive_skips_nested_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("top.md"), "shahar").unwrap();
    fs::create_dir(dir.path().join("nested")).unwrap();
    fs::write(dir.path().join("nested/deep.md"), "shahar").unwrap();

    cmd().arg("--in-place").arg(dir.path()).assert().success();

    assert_eq!(
        fs::read_to_string(dir.path().join("top.md")).unwrap(),
        "şahar"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("nested/deep.md")).unwrap(),
        "shahar",
        "non-recursive run must not touch nested files"
    );
}

#[test]
fn exclude_glob_skips_matching_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("keep.md"), "shahar").unwrap();
    fs::write(dir.path().join("skip.md"), "shahar").unwrap();

    cmd()
        .arg("--in-place")
        .arg("--exclude")
        .arg("*skip.md")
        .arg(dir.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("keep.md")).unwrap(),
        "şahar"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("skip.md")).unwrap(),
        "shahar"
    );
}

#[test]
fn unrecognized_extension_is_untouched_by_default() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "shahar").unwrap();

    cmd().arg("--in-place").arg(dir.path()).assert().success();

    assert_eq!(
        fs::read_to_string(dir.path().join("a.rs")).unwrap(),
        "shahar",
        ".rs files must never be touched"
    );
}

#[test]
fn extra_ext_flag_opts_in_new_extension() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rst"), "shahar").unwrap();

    cmd()
        .arg("--in-place")
        .arg("--ext")
        .arg("rst")
        .arg(dir.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("a.rst")).unwrap(),
        "şahar"
    );
}

#[test]
fn nonexistent_path_is_an_error() {
    cmd().arg("/no/such/path/at/all").assert().failure();
}

#[test]
fn html_file_preserves_tags_end_to_end() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.html");
    fs::write(&path, "<p class=\"shahar\">shahar</p>").unwrap();

    cmd()
        .arg(&path)
        .assert()
        .success()
        .stdout("<p class=\"shahar\">şahar</p>");
}

#[test]
fn markdown_code_fence_preserved_end_to_end() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.md");
    fs::write(&path, "shahar\n\n```\nsh\n```\n").unwrap();

    cmd()
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("```\nsh\n```"))
        .stdout(predicate::str::starts_with("şahar"));
}
