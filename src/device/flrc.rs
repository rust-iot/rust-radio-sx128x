//! Flrc mode device configuration definitions

use super::common::*;

/// FLRC configuration structure
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))] 
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
pub struct FlrcConfig {
    pub preamble_length: PreambleLength,
    pub sync_word_length: FlrcSyncWordLength,
    pub sync_word_match: SyncWordRxMatch,
    pub sync_word_value: Option<[u8; 4]>,
    pub header_type: GfskFlrcPacketLength,
    pub payload_length: u8,
    pub crc_mode: GfskFlrcCrcModes,
    pub whitening: WhiteningModes
}

impl Default for FlrcConfig {
    fn default() -> Self {
        Self{
            preamble_length: PreambleLength::PreambleLength16,
            sync_word_length: FlrcSyncWordLength::Length4,
            sync_word_match: SyncWordRxMatch::RADIO_RX_MATCH_SYNCWORD_1,
            sync_word_value: Some([0x12, 0x13, 0x14, 0x15]),
            header_type: GfskFlrcPacketLength::Variable,
            payload_length: 0x40,
            crc_mode: GfskFlrcCrcModes::RADIO_CRC_2_BYTES,
            whitening: WhiteningModes::RADIO_WHITENING_OFF,
        }
    }
}

/// Bit rate / bandwidth pairs for FLRC mode
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))] 
pub enum FlrcBitrate {
    /// Baud: 2600 kbps Bandwidth: 2.4 MHz
    BR_2_600_BW_2_4                    = 0x04,
    /// Baud: 2080 kbps Bandwidth: 2.4 MHz
    BR_2_080_BW_2_4                    = 0x28,
    /// Baud: 1300 kbps Bandwidth: 1.2 MHz
    BR_1_300_BW_1_2                    = 0x45,
    /// Baud: 1040 kbps Bandwidth: 1.2 MHz
    BR_1_040_BW_1_2                    = 0x69,
    /// Baud: 650 kbps Bandwidth: 0.6 MHz
    BR_0_650_BW_0_6                    = 0x86,
    /// Baud: 520 kbps Bandwidth: 0.6 MHz
    BR_0_520_BW_0_6                    = 0xAA,
    /// Baud: 325 kbps Bandwidth: 0.3 MHz
    BR_0_325_BW_0_3                    = 0xC7,
    /// Baud: 260 kbps Bandwidth: 0.3 MHz
    BR_0_260_BW_0_3                    = 0xEB,
}

impl FlrcBitrate {

    /// Compute an FLRC bitrate/bandwidth from parts
    pub fn from_parts(bitrate_kbps: u32, bandwidth_khz: u32) -> Option<Self> {
        use self::FlrcBitrate::*;

        match (bitrate_kbps, bandwidth_khz) {
            (2_600, 2_400) => Some(BR_2_600_BW_2_4),
            (2_080, 2_400) => Some(BR_2_080_BW_2_4),
            (1_300, 1_200) => Some(BR_1_300_BW_1_2),
            (1_040, 1_200) => Some(BR_1_040_BW_1_2),
            (0_650, 0_600) => Some(BR_0_650_BW_0_6),
            (0_520, 0_600) => Some(BR_0_520_BW_0_6),
            (0_325, 0_300) => Some(BR_0_325_BW_0_3),
            (0_260, 0_300) => Some(BR_0_260_BW_0_3),
            _ => None
        }
    }

    /// Fetch the configured baud rate in kbps
    pub fn baud(&self) -> u32 {
        use self::FlrcBitrate::*;

        match self {
            BR_2_600_BW_2_4 => 2_600,
            BR_2_080_BW_2_4 => 2_080,
            BR_1_300_BW_1_2 => 1_300,
            BR_1_040_BW_1_2 => 1_040,
            BR_0_650_BW_0_6 => 0_650,
            BR_0_520_BW_0_6 => 0_520,
            BR_0_325_BW_0_3 => 0_325,
            BR_0_260_BW_0_3 => 0_260,
        }
    }

    /// Fetch the configured bandwidth in kHz
    pub fn bw(&self) -> u32 {
        use self::FlrcBitrate::*;
        
        match self {
            BR_2_600_BW_2_4 => 2_600,
            BR_2_080_BW_2_4 => 2_080,
            BR_1_300_BW_1_2 => 1_300,
            BR_1_040_BW_1_2 => 1_040,
            BR_0_650_BW_0_6 => 0_650,
            BR_0_520_BW_0_6 => 0_520,
            BR_0_325_BW_0_3 => 0_325,
            BR_0_260_BW_0_3 => 0_260,
        }
    }
}

/// Coding rates for FLRC mode
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))] 
pub enum FlrcCodingRate {
    /// 1/2 coding rate
    Cr1_2 = 0x00,
    /// 3/4 coding rate
    Cr3_4 = 0x02,
    /// 1/0 coding rate (disabled)
    Cr1_0 = 0x04,
}


/// FLRC sync word length
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))] 
pub enum FlrcSyncWordLength {
    /// No sync word
    None = 0x00,
    /// 4-byte sync word
    Length4 = 0x04,
}


