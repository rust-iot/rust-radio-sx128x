/// Modulation shaping parameter for GFSK, FLRC and BLE modes
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "util", derive(structopt::StructOpt))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ModShaping {
    /// No filtering
    Off = 0x00,
    Bt1_0 = 0x10,
    Bt0_5 = 0x20,
}

/// Preamble lengths for GFSK, FLRC modes
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PreambleLength {
    /// Preamble length: 04 bits
    PreambleLength04 = 0x00,
    /// Preamble length: 08 bits
    PreambleLength08 = 0x10,
    /// Preamble length: 12 bits
    PreambleLength12 = 0x20,
    /// Preamble length: 16 bits
    PreambleLength16 = 0x30,
    /// Preamble length: 20 bits
    PreambleLength20 = 0x40,
    /// Preamble length: 24 bits
    PreambleLength24 = 0x50,
    /// Preamble length: 28 bits
    PreambleLength28 = 0x60,
    /// Preamble length: 32 bits
    PreambleLength32 = 0x70,
}

/// Bitrate-Bandwidth for GFSK and BLE modes
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "util", derive(structopt::StructOpt))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GfskBleBitrateBandwidth {
    /// Raw baudrate: 2000 kbps Bandwidth: 2.4 MHz
    BR_2_000_BW_2_4 = 0x04,
    /// Raw baudrate: 1600 kbps Bandwidth: 2.4 MHz
    BR_1_600_BW_2_4 = 0x28,
    /// Raw baudrate: 1000 kbps Bandwidth: 2.4 MHz
    BR_1_000_BW_2_4 = 0x4C,
    /// Raw baudrate: 1000 kbps Bandwidth: 1.2 MHz
    BR_1_000_BW_1_2 = 0x45,
    /// Raw baudrate: 800 kbps Bandwidth: 2.4 MHz
    BR_0_800_BW_2_4 = 0x70,
    /// Raw baudrate: 800 kbps Bandwidth: 1.2 MHz
    BR_0_800_BW_1_2 = 0x69,
    /// Raw baudrate: 500 kbps Bandwidth: 1.2 MHz
    BR_0_500_BW_1_2 = 0x8D,
    /// Raw baudrate: 500 kbps Bandwidth: 0.6 MHz
    BR_0_500_BW_0_6 = 0x86,
    /// Raw baudrate: 400 kbps Bandwidth: 1.2 MHz
    BR_0_400_BW_1_2 = 0xB1,
    /// Raw baudrate: 400 kbps Bandwidth: 0.6 MHz
    BR_0_400_BW_0_6 = 0xAA,
    /// Raw baudrate: 250 kbps Bandwidth: 0.6 MHz
    BR_0_250_BW_0_6 = 0xCE,
    /// Raw baudrate: 250 kbps Bandwidth: 0.3 MHz
    BR_0_250_BW_0_3 = 0xC7,
    /// Raw baudrate: 125 kbps Bandwidth: 0.3 MHz
    BR_0_125_BW_0_3 = 0xEF,
}

/// Modulation Index for GFSK and BLE modes
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GfskBleModIndex {
    MOD_IND_0_35 = 0,
    MOD_IND_0_50 = 1,
    MOD_IND_0_75 = 2,
    MOD_IND_1_00 = 3,
    MOD_IND_1_25 = 4,
    MOD_IND_1_50 = 5,
    MOD_IND_1_75 = 6,
    MOD_IND_2_00 = 7,
    MOD_IND_2_25 = 8,
    MOD_IND_2_50 = 9,
    MOD_IND_2_75 = 10,
    MOD_IND_3_00 = 11,
    MOD_IND_3_25 = 12,
    MOD_IND_3_50 = 13,
    MOD_IND_3_75 = 14,
    MOD_IND_4_00 = 15,
}

/// Common radio whitening mode
#[derive(Copy, Clone, PartialEq, Debug, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WhiteningModes {
    RADIO_WHITENING_ON = 0x00,
    RADIO_WHITENING_OFF = 0x08,
}

#[derive(Copy, Clone, PartialEq, Debug, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SyncWordRxMatch {
    /// No correlator turned on, i.e. do not search for SyncWord
    RADIO_RX_MATCH_SYNCWORD_OFF = 0x00,
    RADIO_RX_MATCH_SYNCWORD_1 = 0x10,
    RADIO_RX_MATCH_SYNCWORD_2 = 0x20,
    RADIO_RX_MATCH_SYNCWORD_1_2 = 0x30,
    RADIO_RX_MATCH_SYNCWORD_3 = 0x40,
    RADIO_RX_MATCH_SYNCWORD_1_3 = 0x50,
    RADIO_RX_MATCH_SYNCWORD_2_3 = 0x60,
    RADIO_RX_MATCH_SYNCWORD_1_2_3 = 0x70,
}

#[derive(Copy, Clone, PartialEq, Debug, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GfskFlrcPacketLength {
    /// Fixed length, no header included
    Fixed = 0x00,
    /// Variable length, header included in message
    Variable = 0x20,
}

#[derive(Copy, Clone, PartialEq, Debug, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GfskFlrcCrcModes {
    /// CRC disabled
    RADIO_CRC_OFF = 0x00,
    RADIO_CRC_2_BYTES = 0x10,
    RADIO_CRC_3_BYTES = 0x20,
    RADIO_CRC_4_BYTES = 0x30,
}
