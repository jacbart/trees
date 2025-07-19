use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("trees-bin").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("trees-bin"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("rm"))
        .stdout(predicate::str::contains("merge"))
        .stdout(predicate::str::contains("pull"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("trees-bin").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("trees"));
}
