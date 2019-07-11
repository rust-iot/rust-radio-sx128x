//! LoRa device configuration definitions

/// LoRa mode channel configuration
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LoRaConfig {
    /// Operating frequency in Hz
    pub freq: u32,
    /// LoRa Spreading Factor
    pub sf: LoRaSpreadingFactor,
    /// LoRa channel bandwidth
    pub bw: LoRaBandwidth,
    /// LoRa Coding rate
    pub cr: LoRaCodingRate,
}

impl Default for LoRaConfig {
    fn default() -> Self {
        Self {
            freq: 2.4e9 as u32,
            sf: LoRaSpreadingFactor::Sf8,
            bw: LoRaBandwidth::Bw200kHz,
            cr: LoRaCodingRate::Cr4_5,
        }
    }
}

/// LoRa mode packet configuration
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LoRaPacketConfig {
    pub preamble_length: u8,
    pub header_type: LoRaHeader,
    pub payload_length: u8,
    pub crc_mode: LoRaCrc,
    pub invert_iq: LoRaIq,
}

impl Default for LoRaPacketConfig {
    fn default() -> Self {
        Self {
            preamble_length: 08,
            header_type: LoRaHeader::Explicit,
            payload_length: 0x40,
            crc_mode: LoRaCrc::Enabled,
            invert_iq: LoRaIq::Inverted,
        }
    }
}

/// Spreading factor for LoRa mode
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum LoRaSpreadingFactor {
    Sf5   = 0x50,
    Sf6   = 0x60,
    Sf7   = 0x70,
    Sf8   = 0x80,
    Sf9   = 0x90,
    Sf10  = 0xA0,
    Sf11  = 0xB0,
    Sf12  = 0xC0,
}

/// Bandwidth for LoRa mode
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum LoRaBandwidth {
    /// 200 kHz bandwidth mode (actually 203.125 kHz)
    Bw200kHz  = 0x34,
    /// 400 kHz bandwidth mode (actually 406.250 kHz)
    Bw400kHz  = 0x26,
    /// 800 kHz bandwidth mode (actually 812.500 kHz)
    Bw800kHz  = 0x18,
    /// 1600 kHz bandwidth mode (actually 1625.000 kHz)
    Bw1600kHz  = 0x0A,
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
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum LoRaCodingRate {
    /// LoRa coding rate 4/5
    Cr4_5    = 0x01,
    /// LoRa coding rate 4/6
    Cr4_6    = 0x02,
    /// LoRa coding rate 4/7
    Cr4_7    = 0x03,
    /// LoRa coding rate 4/8
    Cr4_8    = 0x04,

    CrLI_4_5 = 0x05,
    CrLI_4_6 = 0x06,
    CrLI_4_7 = 0x07,
}

/// CRC mode for LoRa packet types
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum LoRaCrc {
    Enabled = 0x20,
    Disabled = 0x00,
}

/// IQ mode for LoRa packet types
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum LoRaIq {
    Normal = 0x40,
    Inverted = 0x00,
}

/// Header configuration for LoRa packet types
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum LoRaHeader {
    /// Variable length packets, length header included in packet
    Explicit = 0x00,
    /// Constant length packets, no length header included in packet
    Implicit = 0x80,
}

