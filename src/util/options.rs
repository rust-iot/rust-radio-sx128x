use std::str::FromStr;

use clap::Parser;
use driver_pal::hal::DeviceConfig;
use tracing_subscriber::filter::LevelFilter;

use radio::helpers::Operation;

use radio_sx128x::{
    device::{
        common::{self, GfskFlrcCrcModes::*, PreambleLength::*},
        flrc, lora,
    },
    prelude::*,
};

#[derive(Parser)]
#[clap(name = "Sx128x-util")]
/// A Command Line Interface (CLI) for interacting with a local Sx128x radio device
pub struct Options {
    #[clap(subcommand)]
    /// Request for remote-hal server
    pub command: Command,

    #[clap(flatten)]
    pub spi_config: DeviceConfig,

    /// Use onboard DCDC instead of LDO
    #[clap(long, env = "USE_DCDC")]
    pub use_dcdc: bool,

    /// Set CRC length (0, 2, 3 bytes)
    #[clap(long, default_value = "2", env = "CRC_MODE")]
    pub crc_mode: u8,

    /// Set preamble length
    #[clap(long, default_value = "16", env = "PREAMBLE_LEN")]
    pub preamble_len: u8,

    #[clap(long, default_value = "info")]
    /// Configure radio log level
    pub log_level: LevelFilter,

    /// Set sync word in hex (base 16), from LSB to MSB without spaces
    #[clap(long, value_parser=HexData::from_str)]
    pub syncword: Option<HexData>,
}

#[derive(Parser, PartialEq, Debug)]
pub enum Command {
    #[clap(name = "chip-version")]
    /// Fetch the device silicon/firmware version
    FirmwareVersion,

    #[clap(name = "lora")]
    /// LoRa mode configuration and operations
    LoRa(LoRaCommand),

    #[clap(name = "gfsk")]
    /// GFSK mode configuration and operations
    Gfsk(GfskCommand),

    #[clap(name = "flrc")]
    /// FLRC mode configuration and operations
    Flrc(FlrcCommand),
}

#[derive(Clone, Debug, PartialEq)]
pub struct HexData(pub Vec<u8>);

impl std::str::FromStr for HexData {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex::decode(s).map(HexData)
    }
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

        config.regulator_mode = match self.use_dcdc {
            true => RegulatorMode::Dcdc,
            false => RegulatorMode::Ldo,
        };

        // Generate configurations
        match &self.command {
            Command::LoRa(lora_config) => {
                // Set to lora mode
                let mut modem = LoRaConfig::default();
                if self.crc_mode == 0 {
                    modem.crc_mode = lora::LoRaCrc::Disabled;
                } else {
                    modem.crc_mode = lora::LoRaCrc::Enabled;
                }

                config.modem = Modem::LoRa(modem);

                let mut channel = LoRaChannel::default();
                channel.freq = (lora_config.frequency * 1e9) as u32;

                config.channel = Channel::LoRa(channel);
            }
            Command::Flrc(flrc_config) => {
                // Set to Gfsk mode
                let mut modem = FlrcConfig::default();

                modem.crc_mode = match self.crc_mode {
                    0 => RADIO_CRC_OFF,
                    2 => RADIO_CRC_2_BYTES,
                    3 => RADIO_CRC_3_BYTES,
                    4 => RADIO_CRC_4_BYTES,
                    _ => unimplemented!(),
                };

                modem.preamble_length = match self.preamble_len {
                    4 => PreambleLength04,
                    08 => PreambleLength08,
                    12 => PreambleLength12,
                    16 => PreambleLength16,
                    20 => PreambleLength20,
                    24 => PreambleLength24,
                    28 => PreambleLength28,
                    32 => PreambleLength32,
                    _ => unimplemented!(),
                };

                if flrc_config.no_syncword {
                    modem.sync_word_match = common::SyncWordRxMatch::RADIO_RX_MATCH_SYNCWORD_OFF;
                }

                config.modem = Modem::Flrc(modem);

                let mut channel = FlrcChannel::default();
                channel.freq = (flrc_config.frequency * 1e9) as u32;
                channel.br_bw = flrc_config.bitrate_bandwidth;
                channel.cr = flrc_config.code_rate;

                config.channel = Channel::Flrc(channel);
            }
            Command::Gfsk(gfsk_config) => {
                // Set to Gfsk mode
                let mut modem = GfskConfig::default();

                match self.crc_mode {
                    0 => modem.crc_mode = common::GfskFlrcCrcModes::RADIO_CRC_OFF,
                    2 => modem.crc_mode = common::GfskFlrcCrcModes::RADIO_CRC_2_BYTES,
                    3 => modem.crc_mode = common::GfskFlrcCrcModes::RADIO_CRC_3_BYTES,
                    4 => modem.crc_mode = common::GfskFlrcCrcModes::RADIO_CRC_4_BYTES,
                    _ => unimplemented!(),
                }

                config.modem = Modem::Gfsk(modem);

                let mut channel = GfskChannel::default();
                channel.freq = (gfsk_config.frequency * 1e9) as u32;

                config.channel = Channel::Gfsk(channel);
            }
            _ => (),
        }

        config
    }
}

/// LoRa mode command wrapper
#[derive(Parser, PartialEq, Debug)]
pub struct LoRaCommand {
    /// Operating frequency in GHz
    /// This must be in a range of 2.40 to 2.50 GHz
    #[clap(long = "freq-ghz", default_value = "2.44")]
    pub frequency: f32,

    #[clap(subcommand)]
    /// Operation to execute
    pub operation: Operation,
}

/// GFSK mode command wrapper
#[derive(Parser, PartialEq, Debug)]
pub struct GfskCommand {
    /// Operating frequency in GHz
    /// This must be in a range of 2.40 to 2.50 GHz
    #[clap(long = "freq-ghz", default_value = "2.44")]
    pub frequency: f32,

    #[clap(subcommand)]
    /// Operation to execute
    pub operation: Operation,
}

/// FLRC mode command wrapper
#[derive(Parser, PartialEq, Debug)]
pub struct FlrcCommand {
    /// Operating frequency in GHz
    /// This must be in a range of 2.40 to 2.50 GHz
    #[clap(long = "freq-ghz", default_value = "2.44", env = "FLRC_FREQ_GHZ")]
    pub frequency: f32,

    /// FLRC bitrate-bandwidth in kbps
    /// (options: 2600_2400, 2080_2400, 1300_1200, 1040_1200, 650_600, 520_600, 325_300, 260_300)
    #[clap(long = "br-bw", default_value = "260_300", env = "FLRC_BR_BW")]
    pub bitrate_bandwidth: flrc::FlrcBitrate,

    /// FLRC coding rate
    /// (options: 3/4, 1/2, 1/0)
    #[clap(long = "cr", default_value = "3/4", env = "FLRC_CR")]
    pub code_rate: flrc::FlrcCodingRate,

    /// Disable Sync word matching
    #[clap(long)]
    pub no_syncword: bool,

    #[clap(subcommand)]
    /// Operation to execute
    pub operation: Operation,
}
