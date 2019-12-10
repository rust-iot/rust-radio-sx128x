
use structopt::StructOpt;
use simplelog::{LevelFilter};
use humantime::{Duration as HumanDuration};

use radio_sx128x::device::flrc;

#[derive(StructOpt)]
#[structopt(name = "Sx128x-util")]
/// A Command Line Interface (CLI) for interacting with a local Sx128x radio device
pub struct Options {

    #[structopt(subcommand)]
    /// Request for remote-hal server
    pub command: Command,


    #[structopt(long = "spi", default_value = "/dev/spidev0.0", env = "SX128X_SPI")]
    /// SPI device for the radio connection
    pub spi: String,

    /// Chip Select (output) pin
    #[structopt(long = "cs-pin", default_value = "16", env = "SX128X_CS")]
    pub cs: u64,

    /// Reset (output) pin
    #[structopt(long = "rst-pin", default_value = "17", env = "SX128X_RST")]
    pub rst: u64,

    /// Antenna control pin
    #[structopt(long = "ant-pin", default_value = "23", env = "SX128X_ANT")]
    pub ant: u64,

    /// Busy (input) pin
    #[structopt(long = "busy-pin", default_value = "5", env = "SX128X_BUSY")]
    pub busy: u64,

    /// Baud rate setting
    #[structopt(long = "baud", default_value = "1000000", env = "SX128X_BAUD")]
    pub baud: u32,


    #[structopt(long = "log-level", default_value = "info")]
    /// Enable verbose logging
    pub level: LevelFilter,
}

#[derive(StructOpt, PartialEq, Debug)]
pub enum Command {
    #[structopt(name="chip-version")]
    /// Fetch the device silicon/firmware version
    FirmwareVersion,

    #[structopt(name="lora")]
    /// LoRa mode configuration and operations
    LoRa(LoRaCommand),

    #[structopt(name="gfsk")]
    /// GFSK mode configuration and operations
    Gfsk(GfskCommand),

    #[structopt(name="flrc")]
    /// FLRC mode configuration and operations
    Flrc(FlrcCommand),
}

impl Command {
    pub fn operation(&self) -> Option<Operation> {
        match self {
            Command::FirmwareVersion => None,
            Command::LoRa(c) => Some(c.operation.clone()),
            Command::Gfsk(c) => Some(c.operation.clone()),
            Command::Flrc(c) => Some(c.operation.clone()),
        }
    }
}

/// LoRa mode command wrapper
#[derive(StructOpt, PartialEq, Debug)]
pub struct LoRaCommand {
    /// Operating frequency in GHz
    /// This must be in a range of 2.45 to 2.55 GHz
    #[structopt(long = "freq-ghz", default_value="2.45")]
    pub frequency: f32,


    #[structopt(subcommand)]
    /// Operation to execute
    pub operation: Operation,
}

/// GFSK mode command wrapper
#[derive(StructOpt, PartialEq, Debug)]
pub struct GfskCommand {
    /// Operating frequency in GHz
    /// This must be in a range of 2.45 to 2.55 GHz
    #[structopt(long = "freq-ghz", default_value="2.45")]
    pub frequency: f32,


    #[structopt(subcommand)]
    /// Operation to execute
    pub operation: Operation,
}

/// FLRC mode command wrapper
#[derive(StructOpt, PartialEq, Debug)]
pub struct FlrcCommand {
    /// Operating frequency in GHz
    /// This must be in a range of 2.45 to 2.55 GHz
    #[structopt(long = "freq-ghz", default_value="2.45")]
    pub frequency: f32,

    /// FLRC bitrate-bandwidth in kbps
    /// (options: 2600_2400, 2080_2400, 1300_1200, 1040_1200, 650_600, 520_600, 325_300, 260_300)
    #[structopt(long = "br-bw", default_value="260_300")]
    pub bitrate_bandwidth: flrc::FlrcBitrate,

    /// FLRC coding rate
    /// (options: 3/4, 1/2, 1/0)
    #[structopt(long = "cr", default_value="3/4")]
    pub code_rate: flrc::FlrcCodingRate,

    #[structopt(subcommand)]
    /// Operation to execute
    pub operation: Operation,
}

#[derive(Clone, StructOpt, PartialEq, Debug)]
pub enum Operation {
    #[structopt(name="tx")]
    /// Transmit a (string) packet
    Transmit(Transmit),

    #[structopt(name="rx")]
    /// Receive a (string) packet
    Receive(Receive),

    #[structopt(name="rssi")]
    /// Poll for RSSI on the specified channel
    Rssi(Rssi),

    #[structopt(name="repeat")]
    /// Repeat received messages
    Repeat(Repeat),
}

#[derive(Clone, StructOpt, PartialEq, Debug)]
pub struct Transmit {
    /// Data to be transmitted
    #[structopt(long = "data")]
    pub data: String,

    /// Run continuously
    #[structopt(long = "continuous")]
    pub continuous: bool,

    /// Power in dBm (range -18dBm to 13dBm)
    #[structopt(long = "power")]
    pub power: Option<i8>,

    /// Specify period for transmission
    #[structopt(long = "period", default_value="1s")]
    pub period: HumanDuration,

    /// Specify period for polling for device status
    #[structopt(long = "poll-interval", default_value="10ms")]
    pub poll_interval: HumanDuration,
}

#[derive(Clone, StructOpt, PartialEq, Debug)]
pub struct Receive {
    /// Run continuously
    #[structopt(long = "continuous")]
    pub continuous: bool,

    /// Specify period for polling for device status
    #[structopt(long = "poll-interval", default_value="10ms")]
    pub poll_interval: HumanDuration,
}

#[derive(Clone, StructOpt, PartialEq, Debug)]
pub struct Rssi {
    /// Specify period for RSSI polling
    #[structopt(long = "period", default_value="1s")]
    pub period: HumanDuration,

    /// Run continuously
    #[structopt(long = "continuous")]
    pub continuous: bool,
}

#[derive(Clone, StructOpt, PartialEq, Debug)]
pub struct Repeat {
    /// Run continuously
    #[structopt(long = "continuous")]
    pub continuous: bool,
    
    /// Power in dBm (range -18dBm to 13dBm)
    #[structopt(long = "power")]
    pub power: Option<i8>,

    /// Specify period for polling for device status
    #[structopt(long = "poll-interval", default_value="1ms")]
    pub poll_interval: HumanDuration,

    /// Specify delay for response message
    #[structopt(long = "delay", default_value="100ms")]
    pub delay: HumanDuration,

    /// Append RSSI and LQI to repeated message
    #[structopt(long = "append-info")]
    pub append_info: bool,
}