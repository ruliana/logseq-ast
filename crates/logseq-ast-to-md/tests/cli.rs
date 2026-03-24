use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(path: &str) -> String {
    format!("../../fixtures/ast_to_md/{path}")
}

#[test]
fn stdin_is_default_when_no_args() {
    let input = std::fs::read_to_string(fixture("prose_basic.json")).unwrap();

    let mut cmd = Command::cargo_bin("logseq-ast-to-md").expect("binary");
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World."));
}

#[test]
fn reads_from_file_when_path_is_provided() {
    let expected = std::fs::read_to_string(fixture("list_mode.md")).unwrap();

    let mut cmd = Command::cargo_bin("logseq-ast-to-md").expect("binary");
    cmd.arg(fixture("list_mode.json"))
        .assert()
        .success()
        .stdout(predicate::str::diff(expected));
}
