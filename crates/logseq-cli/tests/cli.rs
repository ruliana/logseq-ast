use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(path: &str) -> String {
    // tests run with cwd at the crate
    format!("../../fixtures/{path}")
}

#[test]
fn stdin_is_default_when_no_args() {
    let mut cmd = Command::cargo_bin("logseq-ast").expect("binary");

    cmd.write_stdin(std::fs::read_to_string(fixture("wiki_and_refs.md")).unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"version\": 1"));
}

#[test]
fn reads_from_file_when_path_is_provided() {
    let expected = std::fs::read_to_string(fixture("wiki_and_refs.json")).unwrap();

    let mut cmd = Command::cargo_bin("logseq-ast").expect("binary");
    cmd.arg(fixture("wiki_and_refs.md"))
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}

#[test]
fn explicit_dash_reads_stdin() {
    let input = std::fs::read_to_string(fixture("code_block.md")).unwrap();

    let mut cmd = Command::cargo_bin("logseq-ast").expect("binary");
    cmd.arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"type\": \"code_block\""));
}

#[test]
fn debug_tokens_prints_to_stderr() {
    let mut cmd = Command::cargo_bin("logseq-ast").expect("binary");
    cmd.arg("--debug-tokens")
        .write_stdin("- Hello [[World]] #tag\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("debug_tokens"));
}
