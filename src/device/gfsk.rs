//! Gfsk mode device configuration definitions

use super::common::*;

/// GFSK operating mode configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
            freq: 2_440_000_000,
            br_bw: GfskBleBitrateBandwidth::BR_0_250_BW_0_3,
            mi: GfskBleModIndex::MOD_IND_0_35,
            ms: ModShaping::Off,
        }
    }
}

/// GFSK packet configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    pub whitening: WhiteningModes,

    /// Patch to set "Packet Preamble Settings" register 0x09C1 with preamble
    /// length. Without this the preamble is not correctly filter from RX'd packets.
    pub patch_preamble: bool,
}

impl Default for GfskConfig {
    fn default() -> Self {
        Self {
            preamble_length: PreambleLength::PreambleLength32,
            sync_word_length: GfskSyncWordLength::GFSK_SYNCWORD_LENGTH_5_BYTE,
            sync_word_match: SyncWordRxMatch::RADIO_RX_MATCH_SYNCWORD_1,
            header_type: GfskFlrcPacketLength::Variable,
            payload_length: 255,
            crc_mode: GfskFlrcCrcModes::RADIO_CRC_OFF,
            whitening: WhiteningModes::RADIO_WHITENING_OFF,
            patch_preamble: false,
        }
    }
}

/// GFSK sync word length configuration
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GfskSyncWordLength {
    /// Sync word length: 1 byte
    GFSK_SYNCWORD_LENGTH_1_BYTE = 0x00,
    /// Sync word length: 2 bytes
    GFSK_SYNCWORD_LENGTH_2_BYTE = 0x02,
    /// Sync word length: 3 bytes
    GFSK_SYNCWORD_LENGTH_3_BYTE = 0x04,
    /// Sync word length: 4 bytes
    GFSK_SYNCWORD_LENGTH_4_BYTE = 0x06,
    /// Sync word length: 5 bytes
    GFSK_SYNCWORD_LENGTH_5_BYTE = 0x08,
}
