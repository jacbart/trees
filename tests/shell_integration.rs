use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_shell_script_handles_add_command() {
    let mut cmd = Command::cargo_bin("trees-bin").unwrap();
    cmd.args(["shell", "zsh"]);
    let output = cmd.output().unwrap();
    let script = String::from_utf8_lossy(&output.stdout);

    // Verify that add command is handled correctly
    assert!(script.contains("add|rm|merge|pull|list"));
    assert!(script.contains("trees-bin \"$@\""));
    assert!(script.contains("return $STATUS"));
}

#[test]
fn test_shell_script_handles_no_args() {
    let mut cmd = Command::cargo_bin("trees-bin").unwrap();
    cmd.args(["shell", "zsh"]);
    let output = cmd.output().unwrap();
    let script = String::from_utf8_lossy(&output.stdout);

    // Verify that no args case is handled correctly
    assert!(script.contains("DIR=$(trees-bin --dir-only)"));
}

#[test]
fn test_bash_script_handles_add_command() {
    let mut cmd = Command::cargo_bin("trees-bin").unwrap();
    cmd.args(["shell", "bash"]);
    let output = cmd.output().unwrap();
    let script = String::from_utf8_lossy(&output.stdout);

    // Verify that add command is handled correctly
    assert!(script.contains("add|rm|merge|pull|list"));
    assert!(script.contains("trees-bin \"$@\""));
    assert!(script.contains("return $STATUS"));
}

#[test]
fn test_fish_script_handles_add_command() {
    let mut cmd = Command::cargo_bin("trees-bin").unwrap();
    cmd.args(["shell", "fish"]);
    let output = cmd.output().unwrap();
    let script = String::from_utf8_lossy(&output.stdout);

    // Verify that add command is handled correctly
    assert!(script.contains("add\" \"rm\" \"merge\" \"pull\" \"list"));
    assert!(script.contains("trees-bin $argv"));
    assert!(script.contains("return $STATUS"));
}
