//! Gfsk mode device configuration definitions

use super::common::*;

/// GFSK operating mode configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))] 
pub struct GfskChannel {
    /// Operating frequency
    pub freq: u32,
    /// Bitrate bandwidth
    pub br_bw: GfskBleBitrateBandwidth,
    /// Modulation index
    pub mi: GfskBleModIndex,
    /// Modulation shaping
    pub ms: ModShaping,
}

impl Default for GfskChannel {
    fn default() -> Self {
        Self {
            freq: 2.4e9 as u32,
            br_bw: GfskBleBitrateBandwidth::BR_0_250_BW_0_3,
            mi: GfskBleModIndex::MOD_IND_0_35,
            ms: ModShaping::Off,
        }   
    }
}

/// GFSK packet configuration 
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))] 
pub struct GfskConfig {
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
    pub crc_mode: GfskFlrcCrcModes,
    /// Packet whitening
    pub whitening: WhiteningModes
}

impl Default for GfskConfig {
    fn default() -> Self {
        Self {
            preamble_length: PreambleLength::PreambleLength16,
            sync_word_length: GfskSyncWordLength::GFSK_SYNCWORD_LENGTH_2_BYTE,
            sync_word_match: SyncWordRxMatch::RADIO_RX_MATCH_SYNCWORD_2,
            header_type: GfskFlrcPacketLength::Variable,
            payload_length: 0x40,
            crc_mode: GfskFlrcCrcModes::RADIO_CRC_2_BYTES,
            whitening: WhiteningModes::RADIO_WHITENING_OFF,
        }   
    }
}

/// GFSK sync word length configuration
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))] 
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
