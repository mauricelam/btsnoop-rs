use assert_cmd::{assert::Assert, prelude::*};
use assert_fs::prelude::PathAssert;
use indoc::indoc;
use nix::{
    sys::{
        signal::{self, Signal},
        stat,
    },
    unistd::Pid,
};
use predicates::prelude::*;
use std::{process::Command, time::Duration};
use wait_timeout::ChildExt;

#[test]
fn list_interfaces() {
    let mut cmd = Command::cargo_bin("btsnoop-extcap").unwrap();
    cmd.arg("--extcap-interfaces")
        .arg("--extcap-version")
        .arg("v0_testing")
        .arg("--adb-path=mock");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(indoc! {"
        interface {value=btsnoop-TEST_SERIAL_1}{display=BTsnoop Test device 1 TEST_SERIAL_1}
        interface {value=btsnoop-TEST_SERIAL_2}{display=BTsnoop Test device 2 TEST_SERIAL_2}
        control {number=0}{type=button}{display=Turn on BT logging}
        "}));
}

fn contains(needle: &[u8]) -> impl Fn(&[u8]) -> bool + '_ {
    move |bytes| bytes.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn capture() {
    let tmpfile = assert_fs::NamedTempFile::new("fifo").unwrap();
    let mut cmd = Command::cargo_bin("btsnoop-extcap").unwrap();
    cmd.arg("--extcap-interface")
        .arg("btsnoop-SERIAL")
        .arg("--capture")
        .arg("--fifo")
        .arg(tmpfile.path())
        .arg("--btsnoop-log-file-path")
        .arg("local:tests/testdata/btsnoop_hci.log")
        .arg("--display-delay")
        .arg("0");
    cmd.assert().success();
    tmpfile.assert(predicate::function(contains(b"Pixel 6 Pro")).from_file_path());
}

#[test]
fn capture_sigterm() -> anyhow::Result<()> {
    let tempdir = assert_fs::TempDir::new()?;
    let fifopath = tempdir.path().join("fifo");
    let hci_log_path = tempdir.path().join("hci_log_path");
    nix::unistd::mkfifo(&fifopath, stat::Mode::S_IRWXU)?;
    let mut cmd = Command::cargo_bin("btsnoop-extcap")?;
    cmd.arg("--extcap-interface")
        .arg("btsnoop-SERIAL")
        .arg("--capture")
        .arg("--fifo")
        .arg(fifopath)
        .arg("--btsnoop-log-file-path")
        .arg(format!("local:{hci_log_path:?}"))
        .arg("--display-delay")
        .arg("0");
    let mut child_proc = cmd.spawn()?;
    let wait_result = child_proc.wait_timeout(Duration::from_millis(100))?;
    assert!(wait_result.is_none());
    println!("child_proc spawned. Sending kill to {:?}", child_proc.id());
    signal::kill(Pid::from_raw(child_proc.id().try_into()?), Signal::SIGTERM)?;
    println!("Kill sent");
    let output = child_proc.wait_with_output()?;
    Assert::new(output).failure();
    Ok(())
}

#[test]
fn missing_fifo() {
    let mut cmd = Command::cargo_bin("btsnoop-extcap").unwrap();
    cmd.arg("--extcap-interface")
        .arg("btsnoop-SERIAL")
        .arg("--capture");
    cmd.assert().failure().stderr(
        predicate::str::contains("the following required arguments were not provided:")
            .and(predicate::str::contains("--fifo <FIFO>")),
    );
}
