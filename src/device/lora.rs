//! LoRa device configuration definitions

/// LoRa mode radio configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LoRaConfig {
    // Preamble length in symbols (defaults to 8)
    pub preamble_length: u8,
    /// LoRa header configuration, defaults to variable packet length with explicit headers
    pub header_type: LoRaHeader,
    /// Payload length configuration (or maximum length for variable mode)
    pub payload_length: u8,
    /// Payload RX CRC configuration (defaults to enabled)
    pub crc_mode: LoRaCrc,
    /// IQ inversion configuration (defaults to disabled)
    pub invert_iq: LoRaIq,
}

impl Default for LoRaConfig {
    fn default() -> Self {
        Self {
            preamble_length: 08,
            header_type: LoRaHeader::Explicit,
            payload_length: 255,
            crc_mode: LoRaCrc::Enabled,
            invert_iq: LoRaIq::Inverted,
        }
    }
}

/// LoRa mode channel configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LoRaChannel {
    /// LoRa frequency in Hz (defaults to 2.4GHz)
    pub freq: u32,
    /// LoRa Spreading Factor (defaults to SF8)
    pub sf: LoRaSpreadingFactor,
    /// LoRa channel bandwidth (defaults to 200kHz)
    pub bw: LoRaBandwidth,
    /// LoRa Coding rate (defaults to 4/5)
    pub cr: LoRaCodingRate,
}

impl Default for LoRaChannel {
    fn default() -> Self {
        Self {
            freq: 2_440_000_000,
            sf: LoRaSpreadingFactor::Sf8,
            bw: LoRaBandwidth::Bw200kHz,
            cr: LoRaCodingRate::Cr4_5,
        }
    }
}

/// Spreading factor for LoRa mode
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaSpreadingFactor {
    Sf5 = 0x50,
    Sf6 = 0x60,
    Sf7 = 0x70,
    Sf8 = 0x80,
    Sf9 = 0x90,
    Sf10 = 0xA0,
    Sf11 = 0xB0,
    Sf12 = 0xC0,
}

/// Bandwidth for LoRa mode
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaBandwidth {
    /// 200 kHz bandwidth mode (actually 203.125 kHz)
    Bw200kHz = 0x34,
    /// 400 kHz bandwidth mode (actually 406.250 kHz)
    Bw400kHz = 0x26,
    /// 800 kHz bandwidth mode (actually 812.500 kHz)
    Bw800kHz = 0x18,
    /// 1600 kHz bandwidth mode (actually 1625.000 kHz)
    Bw1600kHz = 0x0A,
}

impl LoRaBandwidth {
    /// Fetch the bandwidth in Hz for a given bandwidth configuration
    pub fn get_bw_hz(&self) -> u32 {
        match self {
            LoRaBandwidth::Bw200kHz => 203125,
            LoRaBandwidth::Bw400kHz => 406250,
            LoRaBandwidth::Bw800kHz => 812500,
            LoRaBandwidth::Bw1600kHz => 1625000,
        }
    }
}

/// Coding rates for LoRa mode
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaCodingRate {
    /// LoRa coding rate 4/5
    Cr4_5 = 0x01,
    /// LoRa coding rate 4/6
    Cr4_6 = 0x02,
    /// LoRa coding rate 4/7
    Cr4_7 = 0x03,
    /// LoRa coding rate 4/8
    Cr4_8 = 0x04,

    CrLI_4_5 = 0x05,
    CrLI_4_6 = 0x06,
    CrLI_4_7 = 0x07,
}

/// CRC mode for LoRa packet types
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaCrc {
    Enabled = 0x20,
    Disabled = 0x00,
}

/// IQ mode for LoRa packet types
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaIq {
    Normal = 0x40,
    Inverted = 0x00,
}

/// Header configuration for LoRa packet types
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaHeader {
    /// Variable length packets, length header included in packet
    Explicit = 0x00,
    /// Constant length packets, no length header included in packet
    Implicit = 0x80,
}
