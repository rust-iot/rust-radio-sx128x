//! Gfsk mode device configuration definitions

use super::common::*;

/// GFSK operating mode configuration
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GfskConfig {
    /// Operating frequency
    pub freq: u32,
    /// Bitrate bandwidth
    pub br_bw: GfskBleBitrateBandwidth,
    /// Modulation index
    pub mi: GfskBleModIndex,
    /// Modulation shaping
    pub ms: ModShaping,
}

impl Default for GfskConfig {
    fn default() -> Self {
        Self {
            freq: 2.4e9 as u32,
            br_bw: GfskBleBitrateBandwidth::GFSK_BLE_BR_0_250_BW_0_3,
            mi: GfskBleModIndex::GFSK_BLE_MOD_IND_0_35,
            ms: ModShaping::BtOFF,
        }   
    }
}

/// GFSK packet configuration 
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GfskPacketConfig {
    /// Preamble length
    pub preamble_length: PreambleLength,
    /// Sync word length
    pub sync_word_length: GfskSyncWordLength,
    /// Sync words to match
    pub sync_word_match: SyncWordRxMatch,
    /// Header type (implicit or explicit)
    pub header_type: GfskFlrcPacketLength,
    /// Payload length
    pub payload_length: u8,
    /// CRC mode
    pub crc_type: GfskFlrcCrcModes,
    /// Packet whitening
    pub whitening: WhiteningModes
}

/// GFSK sync word length configuration
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum GfskSyncWordLength {
    /// Sync word length: 1 byte
    GFSK_SYNCWORD_LENGTH_1_BYTE              = 0x00,
    /// Sync word length: 2 bytes
    GFSK_SYNCWORD_LENGTH_2_BYTE              = 0x02,
    /// Sync word length: 3 bytes
    GFSK_SYNCWORD_LENGTH_3_BYTE              = 0x04,
    /// Sync word length: 4 bytes
    GFSK_SYNCWORD_LENGTH_4_BYTE              = 0x06,
    /// Sync word length: 5 bytes
    GFSK_SYNCWORD_LENGTH_5_BYTE              = 0x08,
}
