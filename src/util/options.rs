
use structopt::StructOpt;
use humantime::{Duration as HumanDuration};

use embedded_spi::hal::{DeviceConfig, LogConfig};

use radio_sx128x::prelude::*;
use radio_sx128x::device::{common, flrc, lora};

#[derive(StructOpt)]
#[structopt(name = "Sx128x-util")]
/// A Command Line Interface (CLI) for interacting with a local Sx128x radio device
pub struct Options {

    #[structopt(subcommand)]
    /// Request for remote-hal server
    pub command: Command,

    #[structopt(flatten)]
    pub spi_config: DeviceConfig,

    /// Use onboard LDO instead of DCDC
    #[structopt(long)]
    pub use_ldo: bool,

    /// Disable CRC appending / checking
    #[structopt(long)]
    pub no_crc: bool,

    #[structopt(flatten)]
    pub log: LogConfig,
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

impl Options {
    pub fn rf_config(&self) -> Config {
        let mut config = Config::default();

        if self.use_ldo {
            config.regulator_mode = RegulatorMode::Ldo;
        }
    
        // Generate configurations
        match &self.command {
            Command::LoRa(lora_config) => {
                // Set to lora mode
                let mut modem = LoRaConfig::default();
                if self.no_crc {
                    modem.crc_mode = lora::LoRaCrc::Disabled;
                }
    
                config.modem = Modem::LoRa(modem);
    
                let mut channel = LoRaChannel::default();
                channel.freq = (lora_config.frequency * 1e9) as u32;
    
                config.channel = Channel::LoRa(channel);
            },
            Command::Flrc(flrc_config) => {
                // Set to Gfsk mode
                let mut modem = FlrcConfig::default();
                if self.no_crc {
                    modem.crc_mode = common::GfskFlrcCrcModes::RADIO_CRC_OFF;
                }
    
                if flrc_config.no_syncword {
                    modem.sync_word_match = common::SyncWordRxMatch::RADIO_RX_MATCH_SYNCWORD_OFF;
                }
    
                config.modem = Modem::Flrc(modem);
    
                let mut channel = FlrcChannel::default();
                channel.freq = (flrc_config.frequency * 1e9) as u32;
                channel.br_bw = flrc_config.bitrate_bandwidth;
    
                config.channel = Channel::Flrc(channel);
            }
            Command::Gfsk(gfsk_config) => {
                // Set to Gfsk mode
                let mut modem = GfskConfig::default();
                if self.no_crc {
                    modem.crc_mode = common::GfskFlrcCrcModes::RADIO_CRC_OFF;
                }
    
                config.modem = Modem::Gfsk(modem);
    
                let mut channel = GfskChannel::default();
                channel.freq = (gfsk_config.frequency * 1e9) as u32;
    
                config.channel = Channel::Gfsk(channel);
            },
            _ => (),
        }

        config
    }
}


/// LoRa mode command wrapper
#[derive(StructOpt, PartialEq, Debug)]
pub struct LoRaCommand {
    /// Operating frequency in GHz
    /// This must be in a range of 2.40 to 2.50 GHz
    #[structopt(long = "freq-ghz", default_value="2.44")]
    pub frequency: f32,


    #[structopt(subcommand)]
    /// Operation to execute
    pub operation: Operation,
}

/// GFSK mode command wrapper
#[derive(StructOpt, PartialEq, Debug)]
pub struct GfskCommand {
    /// Operating frequency in GHz
    /// This must be in a range of 2.40 to 2.50 GHz
    #[structopt(long = "freq-ghz", default_value="2.44")]
    pub frequency: f32,


    #[structopt(subcommand)]
    /// Operation to execute
    pub operation: Operation,
}

/// FLRC mode command wrapper
#[derive(StructOpt, PartialEq, Debug)]
pub struct FlrcCommand {
    /// Operating frequency in GHz
    /// This must be in a range of 2.40 to 2.50 GHz
    #[structopt(long = "freq-ghz", default_value="2.44")]
    pub frequency: f32,

    /// FLRC bitrate-bandwidth in kbps
    /// (options: 2600_2400, 2080_2400, 1300_1200, 1040_1200, 650_600, 520_600, 325_300, 260_300)
    #[structopt(long = "br-bw", default_value="260_300")]
    pub bitrate_bandwidth: flrc::FlrcBitrate,

    /// FLRC coding rate
    /// (options: 3/4, 1/2, 1/0)
    #[structopt(long = "cr", default_value="3/4")]
    pub code_rate: flrc::FlrcCodingRate,

    /// Disable Sync word matching
    #[structopt(long)]
    pub no_syncword: bool,

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
    #[structopt(long = "poll-interval", default_value="1ms")]
    pub poll_interval: HumanDuration,
}

#[derive(Clone, StructOpt, PartialEq, Debug)]
pub struct Receive {
    /// Run continuously
    #[structopt(long = "continuous")]
    pub continuous: bool,

    /// Specify period for polling for device status
    #[structopt(long = "poll-interval", default_value="1ms")]
    pub poll_interval: HumanDuration,

    /// PCAP file for captured packet output
    #[structopt(long)]
    pub pcap_file: Option<String>,
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