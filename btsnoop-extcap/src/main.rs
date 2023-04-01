#![deny(unused_must_use)]

use adb::{AdbRootError, BtsnoopLogMode, BtsnoopLogSettings};
use btsnoop::{FileHeader, PacketHeader};
use btsnoop_ext::Direction;
use clap::Parser;
use lazy_static::lazy_static;
use log::{debug, info, warn, LevelFilter, trace};
use log4rs::{append::file::FileAppender, encode::pattern::PatternEncoder, Config, config::{Appender, Root}};
use nom_derive::Parse as _;
use pcap_file::{
    pcap::{PcapHeader, PcapPacket, PcapWriter},
    DataLink,
};
use r_extcap::{
    cargo_metadata,
    controls::asynchronous::{
        util::AsyncReadExt as _, ExtcapControlSender, ExtcapControlSenderTrait,
    },
    controls::{ButtonControl, ControlCommand, ControlPacket, EnableableControl},
    interface::{Dlt, Interface, Metadata},
    ExtcapArgs, ExtcapStep, PrintSentence,
};
use std::{
    borrow::Cow,
    io::{stdout, Write},
    process::Stdio,
    time::{Duration, Instant},
    path::Path,
};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt},
    sync::Mutex,
};

mod adb;
mod btsnoop_ext;

/// An extcap plugin for Wireshark or tshark that captures the btsnoop HCI logs
/// from an Android device connected over adb.
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct BtsnoopArgs {
    #[command(flatten)]
    extcap: ExtcapArgs,

    /// Specify the path to the btsnoop log file on the device to stream from.
    /// For a special value with the format `local:<path>`, the log file will be
    /// read locally on the host device instead.
    #[arg(long)]
    pub btsnoop_log_file_path: Option<String>,

    /// Specify the path to the ADB executable, or "mock" for a special mock
    /// implementation used for testing.
    #[arg(long)]
    pub adb_path: Option<String>,

    /// Delay in number of seconds before showing packets. Since btsnoop logs
    /// are stored on a file on the Android device, this allows skipping old
    /// packets and only show new ones in the UI.
    #[arg(long, value_parser = |arg: &str| arg.parse().map(std::time::Duration::from_secs), default_value = "1")]
    pub display_delay: Duration,
}

/// Reads from the input, adds the corresponding PCAP headers, and writes to the
/// output data. The `display_delay` can also be set such that packets read
/// during an initial time period will not be displayed.
async fn write_pcap_packets<W: Write, R: AsyncRead + Unpin + Send>(
    mut input_reader: R,
    output_writer: W,
    display_delay: Duration,
) -> anyhow::Result<()> {
    let pcap_header = PcapHeader {
        datalink: DataLink::BLUETOOTH_HCI_H4_WITH_PHDR,
        endianness: pcap_file::Endianness::Big,
        ..Default::default()
    };
    trace!("Reading header...");
    let mut header_buf = [0_u8; FileHeader::LENGTH];
    input_reader.read_exact(&mut header_buf[..]).await?;
    trace!("Parsing header...");
    FileHeader::parse(&header_buf).unwrap();
    let mut pcap_writer = PcapWriter::with_header(output_writer, pcap_header).unwrap();
    let start_time = Instant::now();
    trace!("Start looping...");
    while let Some(packet_header_buf) = input_reader
        .try_read_exact::<{ PacketHeader::LENGTH }>()
        .await?
    {
        trace!("read header: {packet_header_buf:?}");
        let (_rem, packet_header) = PacketHeader::parse(&packet_header_buf).unwrap();
        trace!("Packet length: {packet_header:?}");
        let mut packet_buf: Vec<u8> = vec![0_u8; packet_header.included_length as usize];
        input_reader.read_exact(&mut packet_buf).await?;
        trace!("Read packet content: {packet_buf:?}");
        trace!("Elapsed={:?} delay={:?}", start_time.elapsed(), display_delay);
        if start_time.elapsed() > display_delay {
            let timestamp = packet_header.timestamp();
            let direction =
                Direction::parse_from_payload(&packet_buf).unwrap_or(Direction::Unknown);
            trace!("Writing packet...");
            pcap_writer.write_packet(&PcapPacket {
                timestamp,
                data: Cow::from(&[&direction.to_hci_pseudo_header(), &packet_buf[..]].concat()),
                orig_len: packet_header.original_length + 4,
            })?;
        }
        trace!("Flushing...");
        stdout().flush()?;
    }
    Ok(())
}

async fn handle_control_packet(
    adb_path: &Path,
    serial: &str,
    control_packet: ControlPacket<'_>,
    extcap_control: &mut Option<ExtcapControlSender>,
) -> anyhow::Result<()> {
    let shell = adb::root(adb_path, serial).await?;
    if control_packet.command == ControlCommand::Set {
        if control_packet.control_number == BT_LOGGING_ON_BUTTON.control_number {
            // Turn on
            BtsnoopLogSettings::set_mode(&shell, BtsnoopLogMode::Full).await?;
            extcap_control
                .send(BT_LOGGING_ON_BUTTON.set_enabled(false))
                .await?;
            extcap_control
                .send(BT_LOGGING_OFF_BUTTON.set_enabled(true))
                .await?;
        } else if control_packet.control_number == BT_LOGGING_OFF_BUTTON.control_number {
            // Turn off
            BtsnoopLogSettings::set_mode(&shell, BtsnoopLogMode::Disabled).await?;
            extcap_control
                .send(BT_LOGGING_OFF_BUTTON.set_enabled(false))
                .await?;
            extcap_control
                .send(BT_LOGGING_ON_BUTTON.set_enabled(true))
                .await?;
        } else {
            panic!("Unknown control number {}", control_packet.control_number);
        }
    }
    Ok(())
}

async fn print_packets(
    adb_path: &Path,
    serial: &str,
    extcap_control: &Mutex<Option<ExtcapControlSender>>,
    output_fifo: &mut std::fs::File,
    btsnoop_log_file_path: &Option<String>,
    display_delay: Duration,
) -> anyhow::Result<()> {
    let btsnoop_log_file_path = btsnoop_log_file_path
        .as_deref()
        .unwrap_or("/data/misc/bluetooth/logs/btsnoop_hci.log");
    let write_result = if let Some(test_file) = btsnoop_log_file_path.strip_prefix("local:") {
        write_pcap_packets(File::open(test_file).await?, output_fifo, display_delay).await
    } else {
        let root_shell = match adb::root(adb_path, serial).await {
            Err(e @ AdbRootError::RootDeclined) => {
                extcap_control.info_message(
                    "Unable to get root access. Make sure your device is rooted or on a userdebug/eng build"
                ).await?;
                tokio::time::sleep(Duration::from_secs(1)).await;
                Err(e)?
            }
            Err(e) => Err(e)?,
            Ok(shell) => shell,
        };
        if let Ok(BtsnoopLogMode::Full) = BtsnoopLogSettings::mode(&root_shell).await {
            extcap_control
                .send(BT_LOGGING_ON_BUTTON.set_enabled(false))
                .await?;
            extcap_control
                .send(BT_LOGGING_OFF_BUTTON.set_enabled(true))
                .await?;
        } else {
            extcap_control
                .send(BT_LOGGING_OFF_BUTTON.set_enabled(false))
                .await?;
            extcap_control
                .send(BT_LOGGING_ON_BUTTON.set_enabled(true))
                .await?;
            extcap_control.status_message("BTsnoop logging is turned off. Use View > Interface Toolbars to show the buttons to turn it on").await?;
        }
        let cmd_string = format!(r"until [ -f '{btsnoop_log_file_path}' ]; do sleep 1; done; tail -f -c +0 '{btsnoop_log_file_path}'");
        let mut cmd = root_shell
            .exec_out(cmd_string.as_str())
            .stdout(Stdio::piped())
            .spawn()?;
        info!("Running {cmd_string}");
        let stdout = cmd.stdout.as_mut().unwrap();
        write_pcap_packets(stdout, output_fifo, display_delay).await
    };
    extcap_control
        .status_message("BT capture connection closed")
        .await?;
    // Wireshark overwrites the status bar when we exit, so wait a few seconds
    // so the user at least has a chance to read the message and know why it's
    // flashing.
    tokio::time::sleep(Duration::from_secs(3)).await;
    write_result.map(|_| ())
}

lazy_static! {
    static ref BT_LOGGING_ON_BUTTON: ButtonControl = ButtonControl::builder()
        .control_number(0)
        .display("Turn on BT logging")
        .build();
    static ref BT_LOGGING_OFF_BUTTON: ButtonControl = ButtonControl::builder()
        .control_number(1)
        .display("Turn off BT logging")
        .build();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut log_path = std::env::temp_dir();
    log_path.push("btsnoop.log");
    let logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} - {m}{n}")))
        .build(log_path)?;

    let config = Config::builder()
        .appender(Appender::builder().build("btsnoop", Box::new(logs)))
        .build(Root::builder().appender("btsnoop").build(LevelFilter::Trace))?;
    log4rs::init_config(config)?;
    let args = BtsnoopArgs::parse();
    debug!("Running with args: {args:#?}");
    let dlt = Dlt::builder()
        .data_link_type(DataLink::BLUETOOTH_HCI_H4_WITH_PHDR)
        .name("BluetoothH4".into())
        .display("Bluetooth HCI UART transport layer plius pseudo-header".into())
        .build();
    match args.extcap.run()? {
        ExtcapStep::Interfaces(interfaces_step) => {
            let interfaces: Vec<Interface> = adb::adb_devices(&adb::find_adb(args.adb_path)?)
                .await?
                .iter()
                .map(|d| {
                    Interface::builder()
                        .value(format!("btsnoop-{}", d.serial).into())
                        .display(format!("BTsnoop {} {}", d.display_name, d.serial).into())
                        .dlt(dlt.clone())
                        .build()
                })
                .collect();
            interfaces_step.list_interfaces(
                &Metadata {
                    display_description: "Android btsnoop".into(),
                    ..cargo_metadata!()
                },
                &interfaces.iter().collect::<Vec<&Interface>>(),
                &[&*BT_LOGGING_ON_BUTTON, &*BT_LOGGING_OFF_BUTTON],
            );
        }
        ExtcapStep::Dlts(_dlts_step) => {
            dlt.print_sentence();
        }
        ExtcapStep::Config(_) => {}
        ExtcapStep::ReloadConfig(_) => {}
        ExtcapStep::Capture(mut capture_step) => {
            let interface = capture_step.interface;
            assert!(
                interface.starts_with("btsnoop-"),
                "Interface must start with \"btsnoop-\""
            );
            let serial = interface.split('-').nth(1).unwrap();
            let extcap_reader = capture_step.new_control_reader_async().await;
            let extcap_sender: Mutex<Option<ExtcapControlSender>> =
                Mutex::new(capture_step.new_control_sender_async().await);
            let adb_path = adb::find_adb(args.adb_path)?;
            let result = tokio::try_join!(
                async {
                    if let Some(mut reader) = extcap_reader {
                        while let Ok(packet) = reader.read_control_packet().await {
                            handle_control_packet(
                                &adb_path,
                                serial,
                                packet,
                                &mut *extcap_sender.lock().await,
                            )
                            .await?;
                        }
                    }
                    debug!("Control packet handling ending");
                    Ok::<(), anyhow::Error>(())
                },
                print_packets(
                    &adb_path,
                    serial,
                    &extcap_sender,
                    &mut capture_step.fifo,
                    &args.btsnoop_log_file_path,
                    args.display_delay
                ),
            );
            if let Err(e) = result {
                warn!("Error capturing packets: {e}");
            }
            debug!("Capture ending");
        }
    }
    Ok(())
}
