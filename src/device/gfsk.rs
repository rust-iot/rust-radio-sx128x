
use super::common::*;

#[derive(Clone, PartialEq, Debug)]
pub struct GfskConfig {
    pub bitrate_bandwidth: GfskBleBitrate,
    pub modulation_index: GfskBleModIndex,
    pub modulation_shaping: ModShaping,
}

impl Default for GfskConfig {
    fn default() -> Self {
        Self {
            bitrate_bandwidth: GfskBleBitrate::GFSK_BLE_BR_0_250_BW_0_3,
            modulation_index: GfskBleModIndex::GFSK_BLE_MOD_IND_0_35,
            modulation_shaping: ModShaping::BtOFF,
        }   
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct GfskPacketConfig {
    pub preamble_length: PreambleLength,
    pub sync_word_length: GfskSyncWordLength,
    pub sync_word_match: SyncWordRxMatch,
    pub header_type: GfskFlrcPacketLength,
    pub payload_length: u8,
    pub crc_type: GfskFlrcCrcModes,
    pub whitening: WhiteningModes
}

#[derive(Clone, PartialEq, Debug)]
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
