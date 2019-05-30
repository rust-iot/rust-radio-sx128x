//! LoRa device configuration definitions

/// LoRa mode configuration
#[derive(Clone, PartialEq, Debug)]
pub struct LoRaConfig {
    pub spreading_factor: LoRaSpreadingFactor,
    pub bandwidth: LoRaBandwidth,
    pub coding_rate: LoRaCodingRate,
}

impl Default for LoRaConfig {
    fn default() -> Self {
        Self {
            spreading_factor: LoRaSpreadingFactor::Sf8,
            bandwidth: LoRaBandwidth::Bw0200,
            coding_rate: LoRaCodingRate::Cr4_5,
        }
    }
}

/// LoRa packet configuration
#[derive(Clone, PartialEq, Debug)]
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
#[derive(Copy, Clone, PartialEq, Debug)]
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
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LoRaBandwidth {
    /// 200 kHz bandwidth mode (actually 203.125 kHz)
    Bw0200  = 0x34,
    /// 400 kHz bandwidth mode (actually 406.250 kHz)
    Bw0400  = 0x26,
    /// 800 kHz bandwidth mode (actually 812.500 kHz)
    Bw0800  = 0x18,
    /// 1600 kHz bandwidth mode (actually 1625.000 kHz)
    Bw1600  = 0x0A,
}

impl LoRaBandwidth {
    /// Fetch the bandwidth in Hz for a given bandwidth configuration
    pub fn get_bw_hz(&self) -> u32 {
        match self {
            LoRaBandwidth::Bw0200 => 203125,
            LoRaBandwidth::Bw0400 => 406250,
            LoRaBandwidth::Bw0800 => 812500,
            LoRaBandwidth::Bw1600 => 1625000,
        }
    }
}

/// Coding rates for LoRa mode
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LoRaCodingRate {
    Cr4_5    = 0x01,
    Cr4_6    = 0x02,
    Cr4_7    = 0x03,
    Cr4_8    = 0x04,
    CrLI_4_5 = 0x05,
    CrLI_4_6 = 0x06,
    CrLI_4_7 = 0x07,
}

/// CRC mode for LoRa packet types
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LoRaCrc {
    Enabled = 0x20,
    Disabled = 0x00,
}

/// IQ mode for LoRa packet types
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LoRaIq {
    Normal = 0x40,
    Inverted = 0x00,
}

/// Header configuration for LoRa packet types
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LoRaHeader {
    // Variable length packets, header included
    Explicit = 0x00,
    /// Constant length packets, no header included
    Implicit = 0x80,
}

