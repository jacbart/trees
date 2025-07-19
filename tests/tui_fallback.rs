use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_shell_integration_zsh() {
    let mut cmd = Command::cargo_bin("trees-bin").unwrap();
    cmd.args(["shell", "zsh"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("trees()"))
        .stdout(predicate::str::contains("if [ $# -gt 0 ]"))
        .stdout(predicate::str::contains("if [ -n \"$DIR\" ]"));
}

#[test]
fn test_shell_integration_bash() {
    let mut cmd = Command::cargo_bin("trees-bin").unwrap();
    cmd.args(["shell", "bash"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("trees()"))
        .stdout(predicate::str::contains("if [ -n \"$DIR\" ]"));
}

#[test]
fn test_shell_integration_fish() {
    let mut cmd = Command::cargo_bin("trees-bin").unwrap();
    cmd.args(["shell", "fish"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("function trees"))
        .stdout(predicate::str::contains("if [ -n \"$DIR\" ]"));
}
