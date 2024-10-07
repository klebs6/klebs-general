use rck::*;
use assert_cmd::Command;
use std::process::Command as StdCommand;

#[test]
fn test_status_command() {
    let mut cmd = Command::cargo_bin("rck").unwrap();
    cmd.arg("status")
       .arg("-m")
       .arg("mock/mock_manifest.json");
    cmd.assert().success();
}

#[test]
fn test_sync_command() {
    let mut cmd = Command::cargo_bin("rck").unwrap();
    cmd.arg("sync")
       .arg("-m")
       .arg("mock/mock_manifest.json");
    cmd.assert().success();
}

