//! Utilities for functionalities related to Android Debug Bridge.

use std::{io::BufRead, process::Stdio};

use log::debug;
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum AdbRootError {
    #[error("Root was declined. Check that you are on a userdebug or eng build.")]
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
    adb_path: String,
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
    pub fn shell(&self, command: &str) -> Command {
        let mut cmd = Command::new(&self.adb_path);
        if !self.needs_su {
            cmd.args(["-s", &self.serial, "shell", command]);
        } else {
            cmd.args([
                "-s",
                &self.serial,
                "shell",
                &format!("su -c {}", shlex::quote(command)),
            ]);
        }
        cmd
    }
}

/// Run adb root on the given device.
pub async fn root(adb_path: &str, serial: &str) -> Result<RootShell, AdbRootError> {
    Command::new("adb")
        .args(["-s", serial, "root"])
        .stdout(Stdio::null())
        .spawn()?
        .wait()
        .await?;
    let shell_uid = shell(serial, "id -u")
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()
        .await?
        .stdout;
    debug!("Shell UID={shell_uid:?}");
    if trim_end(&shell_uid) != b"0" {
        let shell_uid = shell(serial, "su -c id -u")
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?
            .stdout;
        if trim_end(&shell_uid) != b"0" {
            // If only `adb root` will return a different exit code...
            Err(AdbRootError::RootDeclined)?;
        } else {
            return Ok(RootShell {
                adb_path: adb_path.to_string(),
                serial: serial.to_string(),
                needs_su: true,
            });
        }
    }
    Ok(RootShell {
        adb_path: adb_path.to_string(),
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
pub fn shell(serial: &str, command: &str) -> Command {
    let mut cmd = Command::new("adb");
    cmd.args(["-s", serial, "shell", command]);
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
pub async fn adb_devices(adb_path: Option<String>) -> anyhow::Result<Vec<AdbDevice>> {
    debug!("Getting adb devices from {adb_path:?}");
    let adb_path = adb_path.as_deref().unwrap_or("adb");
    if adb_path == "mock" {
        return Ok(mock_adb_devices());
    }
    let cmd = Command::new("adb")
        .arg("devices")
        .arg("-l")
        .stdout(Stdio::piped())
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
            .shell(&format!(
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
            .shell("getprop persist.bluetooth.btsnooplogmode")
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
