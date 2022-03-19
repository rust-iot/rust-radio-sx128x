//! Flrc mode device configuration definitions

use super::common::*;

/// FLRC configuration structure
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlrcChannel {
    /// Operating frequency
    pub freq: u32,
    /// Bitrate bandwidth
    pub br_bw: FlrcBitrate,
    /// Coding rate
    pub cr: FlrcCodingRate,
    /// Modulation shaping
    pub ms: ModShaping,
}

impl Default for FlrcChannel {
    fn default() -> Self {
        Self {
            freq: 2_440_000_000,
            br_bw: FlrcBitrate::BR_2_080_BW_2_4,
            cr: FlrcCodingRate::Cr3_4,
            ms: ModShaping::Off,
        }
    }
}

/// FLRC packet configuration structure
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlrcConfig {
    pub preamble_length: PreambleLength,
    pub sync_word_length: FlrcSyncWordLength,
    pub sync_word_match: SyncWordRxMatch,
    pub header_type: GfskFlrcPacketLength,
    pub payload_length: u8,
    pub crc_mode: GfskFlrcCrcModes,
    pub whitening: WhiteningModes,

    /// Patch to resolver errata 16.4, increased PER in FLRC packets with syncword
    /// This sets the LrSyncWordTolerance to maximum
    pub patch_syncword: bool,
}

impl Default for FlrcConfig {
    fn default() -> Self {
        Self {
            preamble_length: PreambleLength::PreambleLength16,
            sync_word_length: FlrcSyncWordLength::Length4,
            sync_word_match: SyncWordRxMatch::RADIO_RX_MATCH_SYNCWORD_1,
            header_type: GfskFlrcPacketLength::Variable,
            payload_length: 127,
            crc_mode: GfskFlrcCrcModes::RADIO_CRC_2_BYTES,
            whitening: WhiteningModes::RADIO_WHITENING_OFF,
            patch_syncword: true,
        }
    }
}

/// Bit rate / bandwidth pairs for FLRC mode
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlrcBitrate {
    /// Baud: 2600 kbps Bandwidth: 2.4 MHz
    BR_2_600_BW_2_4 = 0x04,
    /// Baud: 2080 kbps Bandwidth: 2.4 MHz
    BR_2_080_BW_2_4 = 0x28,
    /// Baud: 1300 kbps Bandwidth: 1.2 MHz
    BR_1_300_BW_1_2 = 0x45,
    /// Baud: 1040 kbps Bandwidth: 1.2 MHz
    BR_1_040_BW_1_2 = 0x69,
    /// Baud: 650 kbps Bandwidth: 0.6 MHz
    BR_0_650_BW_0_6 = 0x86,
    /// Baud: 520 kbps Bandwidth: 0.6 MHz
    BR_0_520_BW_0_6 = 0xAA,
    /// Baud: 325 kbps Bandwidth: 0.3 MHz
    BR_0_325_BW_0_3 = 0xC7,
    /// Baud: 260 kbps Bandwidth: 0.3 MHz
    BR_0_260_BW_0_3 = 0xEB,
}

#[cfg(feature = "util")]
const FLRC_BIT_RATE_PARSE_ERR: &str = "Invalid FLRC bitrate bandwidth (supported options: 2600_2400, 2080_2400, 1300_1200, 1040_1200, 650_600, 520_600, 325_300, 260_300)";

#[cfg(feature = "util")]
impl std::str::FromStr for FlrcBitrate {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::FlrcBitrate::*;

        let v = match s {
            "2600_2400" => BR_2_600_BW_2_4,
            "2080_2400" => BR_2_080_BW_2_4,
            "1300_1200" => BR_1_300_BW_1_2,
            "1040_1200" => BR_1_040_BW_1_2,
            "650_600" => BR_0_650_BW_0_6,
            "520_600" => BR_0_520_BW_0_6,
            "325_300" => BR_0_325_BW_0_3,
            "260_300" => BR_0_260_BW_0_3,
            _ => return Err(FLRC_BIT_RATE_PARSE_ERR),
        };

        Ok(v)
    }
}

/// Coding rates for FLRC mode
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlrcCodingRate {
    /// 1/2 coding rate
    Cr1_2 = 0x00,
    /// 3/4 coding rate
    Cr3_4 = 0x02,
    /// 1/0 coding rate (disabled)
    Cr1_0 = 0x04,
}

#[cfg(feature = "util")]
const FLRC_CODE_RATE_PARSE_ERR: &str = "Invalid coding rate (supported options: 1/2, 3/4, 1/0)";

#[cfg(feature = "util")]
impl std::str::FromStr for FlrcCodingRate {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s {
            "1/2" => FlrcCodingRate::Cr1_2,
            "3/4" => FlrcCodingRate::Cr3_4,
            "1/0" => FlrcCodingRate::Cr1_0,
            _ => return Err(FLRC_CODE_RATE_PARSE_ERR),
        };

        Ok(v)
    }
}

/// FLRC sync word length
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlrcSyncWordLength {
    /// No sync word
    None = 0x00,
    /// 4-byte sync word
    Length4 = 0x04,
}
