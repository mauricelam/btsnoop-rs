//! Utilities for functionalities related to Android Debug Bridge.

use std::{
    io::BufRead,
    path::{Path, PathBuf},
    process::Stdio,
};

use log::debug;
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum AdbRootError {
    #[error(
        "Unable to get root access. Make sure your device is rooted or on a userdebug/eng build."
    )]
    RootDeclined,

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

fn trim_end(input: &[u8]) -> &[u8] {
    match input.iter().rposition(|c| !u8::is_ascii_whitespace(c)) {
        Some(pos) => &input[..=pos],
        None => b"",
    }
}

#[test]
fn trim_end_test() {
    assert_eq!(trim_end(b"hi"), b"hi");
    assert_eq!(trim_end(b"hi\n"), b"hi");
    assert_eq!(trim_end(b"hi\r\n"), b"hi");
    assert_eq!(trim_end(b"\r\n"), b"");
}

/// An ADB shell that can be rooted.
pub struct RootShell {
    /// Path to the adb executable
    adb_path: PathBuf,
    /// The serial number of the Android device
    serial: String,
    /// Whether `su` is needed on subsequent shell invocations. This is true
    /// typically on a rooted production build. If false, we can run `adb root`
    /// to gain root access, so subsequent shell invocations don't need `su`.
    needs_su: bool,
}

impl RootShell {
    /// Run adb shell on the given device.
    ///
    /// Example:
    /// ```
    /// let cmd = adb::shell(serial, format!("echo {}", serial)).spawn()?;
    /// assert_eq!(cmd.wait_with_output().await?.stdout, serial);
    /// ```
    pub fn exec_out(&self, command: &str) -> Command {
        let mut cmd = Command::new(&self.adb_path);
        if !self.needs_su {
            cmd.args(["-s", &self.serial, "exec-out", command]);
        } else {
            cmd.args([
                "-s",
                &self.serial,
                "exec-out",
                // For some reason, `adb exec-out su -c <cmd>` still adds
                // newlines as if `adb shell` is being used, but that behavior
                // is suppressed when the output is piped to `cat`.
                &format!("su -c {} | cat", shlex::quote(command)),
            ]);
        }
        debug!("ADB shell [{cmd:?}]");
        cmd
    }
}

/// Run adb root on the given device.
pub async fn root(adb_path: &Path, serial: &str) -> Result<RootShell, AdbRootError> {
    Command::new(adb_path)
        .args(["-s", serial, "root"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait_with_output()
        .await?;
    let shell_uid = exec_out(adb_path, serial, "id -u")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?
        .wait_with_output()
        .await?
        .stdout;
    debug!("Shell UID={shell_uid:?}");
    // If only `adb root` will return a different exit code...
    if trim_end(&shell_uid) != b"0" {
        let shell_uid = exec_out(adb_path, serial, "su -c id -u")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?
            .wait_with_output()
            .await?
            .stdout;
        if trim_end(&shell_uid) != b"0" {
            Err(AdbRootError::RootDeclined)?;
        } else {
            return Ok(RootShell {
                adb_path: adb_path.to_owned(),
                serial: serial.to_string(),
                needs_su: true,
            });
        }
    }
    Ok(RootShell {
        adb_path: adb_path.to_owned(),
        serial: serial.to_string(),
        needs_su: false,
    })
}

/// Run adb shell on the given device.
///
/// Example:
/// ```
/// let cmd = adb::shell(serial, format!("echo {}", serial)).spawn()?;
/// assert_eq!(cmd.wait_with_output().await?.stdout, serial);
/// ```
pub fn exec_out(adb_path: &Path, serial: &str, command: &str) -> Command {
    let mut cmd = Command::new(adb_path);
    cmd.stderr(Stdio::null());
    cmd.args(["-s", serial, "exec-out", command]);
    cmd
}

/// A structure representing a device connected over ADB.
pub struct AdbDevice {
    /// The serial number of the device. Most functions in this module requires
    /// the serial number as input.
    pub serial: String,
    /// A user-friendly display name of the device. (e.g. Pixel 6)
    pub display_name: String,
}

/// Query `adb devices` for the list of devices, and return a vec of [`AdbDevice`] structs.
pub async fn adb_devices(adb_path: &Path) -> anyhow::Result<Vec<AdbDevice>> {
    debug!("Getting adb devices from {adb_path:?}");
    if adb_path == Path::new("mock") {
        return Ok(mock_adb_devices());
    }
    let cmd = Command::new(adb_path)
        .arg("devices")
        .arg("-l")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let output = cmd.wait_with_output().await?;
    debug!(
        "Found adb devices {:?}",
        std::str::from_utf8(&output.stdout)
    );
    let re = regex::Regex::new(r"([a-zA-Z0-9]+)\s+device.*model:([^ ]+).*")?;
    Ok(output
        .stdout
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let cap = re.captures_iter(&line).next()?;
            Some(AdbDevice {
                serial: cap[1].to_owned(),
                display_name: cap[2].to_owned(),
            })
        })
        .collect())
}

fn mock_adb_devices() -> Vec<AdbDevice> {
    vec![
        AdbDevice {
            serial: String::from("TEST_SERIAL_1"),
            display_name: String::from("Test device 1"),
        },
        AdbDevice {
            serial: String::from("TEST_SERIAL_2"),
            display_name: String::from("Test device 2"),
        },
    ]
}

/// The Btsnoop log mode, as reflected in "Settings > System > Developer options >
/// Enable Bluetooth HCI snoop log".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BtsnoopLogMode {
    Disabled,
    Filtered,
    Full,
}

/// Functions for controlling the btsnoop log settings, as controlled in "Settings >
/// System > Developer options > Enable Bluetooth HCI snoop log".
pub enum BtsnoopLogSettings {}

impl BtsnoopLogSettings {
    pub async fn set_mode(shell: &RootShell, mode: BtsnoopLogMode) -> std::io::Result<()> {
        let mode_str = match mode {
            BtsnoopLogMode::Disabled => "disabled",
            BtsnoopLogMode::Filtered => "filtered",
            BtsnoopLogMode::Full => "full",
        };
        shell
            .exec_out(&format!(
                concat!(
                    "setprop persist.bluetooth.btsnooplogmode {mode}",
                    " && svc bluetooth disable",
                    " && sleep 2",
                    " && svc bluetooth enable"
                ),
                mode = mode_str
            ))
            .spawn()?
            .wait()
            .await?;
        Ok(())
    }

    /// Gets the value of btsnoop log mode setting.
    pub async fn mode(root_shell: &RootShell) -> anyhow::Result<BtsnoopLogMode> {
        let btsnooplogmode_proc = root_shell
            .exec_out("getprop persist.bluetooth.btsnooplogmode")
            .stdout(Stdio::piped())
            .spawn()?;
        let output = btsnooplogmode_proc.wait_with_output().await?;
        match trim_end(&output.stdout) {
            b"full" => Ok(BtsnoopLogMode::Full),
            b"filtered" => Ok(BtsnoopLogMode::Filtered),
            b"disabled" => Ok(BtsnoopLogMode::Disabled),
            _ => Ok(BtsnoopLogMode::Disabled),
        }
    }
}

pub fn find_adb(adb_path: Option<String>) -> anyhow::Result<PathBuf> {
    match adb_path {
        Some(path) => Ok(path.into()),
        None => match which::which("adb") {
            Ok(result) => Ok(result),
            Err(_) => Ok(if cfg!(target_os = "windows") {
                let mut adb_default_path = dirs_next::data_local_dir()
                    .ok_or_else(|| anyhow::format_err!("Cannot find data local directory"))?;
                adb_default_path.push(r"Android\sdk\platform-tools\adb");
                adb_default_path
            } else {
                let mut adb_default_path = dirs_next::home_dir()
                    .ok_or_else(|| anyhow::format_err!("Cannot find home directory"))?;
                adb_default_path.push("Library/Android/Sdk/platform-tools/adb");
                adb_default_path
            }),
        },
    }
}

/// Tests our assumptions about the behavior of adb. These tests require you to
/// be connected to a device over adb in order to run. To ignore these tests, use
///
/// ```sh
/// cargo test -- --skip skip_in_ci
/// ```
#[cfg(test)]
mod skip_in_ci_adb_assumption_tests {
    use assert_cmd::Command;
    use predicates::prelude::predicate;

    #[cfg(windows)]
    const LINE_ENDING: &str = "\r\n";
    #[cfg(not(windows))]
    const LINE_ENDING: &str = "\n";

    #[test]
    fn adb_exec_out_with_su() {
        // We don't really care about the results of this test, but this
        // illustrates why we needed `| cat` in our implementation.
        let mut cmd = Command::new(super::find_adb(None).unwrap());
        cmd.arg("exec-out")
            .arg(r#"su -c "echo -n 'hello\nworld'""#);
        cmd.assert()
            .success()
            .stdout(predicate::str::diff("hello\r\nworld")
        );
    }

    #[test]
    fn adb_exec_out_with_su_and_cat() {
        let mut cmd = Command::new(super::find_adb(None).unwrap());
        cmd.arg("exec-out")
            .arg(r#"su -c "echo -n 'hello\nworld'" | cat"#);
        cmd.assert()
            .success()
            .stdout(predicate::str::diff("hello\nworld")
        );
    }

    #[test]
    fn adb_exec_out() {
        let mut cmd = Command::new(super::find_adb(None).unwrap());
        cmd.arg("exec-out")
            .arg("echo -n 'hello\nworld'");
        cmd.assert()
            .success()
            .stdout(predicate::str::diff("hello\nworld")
        );
    }

    #[test]
    fn adb_shell() {
        let mut cmd = Command::new(super::find_adb(None).unwrap());
        cmd.arg("shell")
            .arg("echo -n 'hello\nworld'");
        cmd.assert()
            .success()
            .stdout(predicate::str::diff(format!("hello{LINE_ENDING}world"))
        );
    }

}
