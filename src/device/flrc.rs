//! Flrc mode device configuration definitions

use super::common::*;

/// FLRC configuration structure
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FlrcConfig {
    /// Operating frequency
    pub freq: u32,
    /// Bitrate bandwidth
    pub br_bw: FlrcBitrate,
    /// Coding rate
    pub cr: FlrcCodingRate,
    /// Modulation shaping
    pub ms: ModShaping,
}

impl Default for FlrcConfig {
    fn default() -> Self {
        Self {
            freq: 2.4e9 as u32,
            br_bw: FlrcBitrate::BR_2_080_BW_2_4,
            cr: FlrcCodingRate::Cr3_4,
            ms: ModShaping::BtOFF,
        }   
    }
}


/// FLRC packet configuration structure
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FlrcPacketConfig {
    pub preamble_length: PreambleLength,
    pub sync_word_length: FlrcSyncWordLength,
    pub sync_word_match: SyncWordRxMatch,
    pub header_type: GfskFlrcPacketLength,
    pub payload_length: u8,
    pub crc_type: GfskFlrcCrcModes,
    pub whitening: WhiteningModes
}

/// Bit rate / bandwidth pairs for FLRC mode
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FlrcBitrate {
    BR_2_600_BW_2_4                    = 0x04,
    BR_2_080_BW_2_4                    = 0x28,
    BR_1_300_BW_1_2                    = 0x45,
    BR_1_040_BW_1_2                    = 0x69,
    BR_0_650_BW_0_6                    = 0x86,
    BR_0_520_BW_0_6                    = 0xAA,
    BR_0_325_BW_0_3                    = 0xC7,
    BR_0_260_BW_0_3                    = 0xEB,
}

/// Coding rates for FLRC mode
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FlrcCodingRate {
    Cr1_2 = 0x00,
    Cr3_4 = 0x02,
    Cr1_0 = 0x04,
}

/// FLRC sync word length
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FlrcSyncWordLength {
    /// No sync word
    None = 0x00,
    /// 4-byte sync word
    Length4 = 0x04,
}
