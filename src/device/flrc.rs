
/// FLRC configuration structure
#[derive(Clone, PartialEq, Debug)]
pub struct Flrc {
    pub bitrate_bandwidth: FlrcBitrate,
    pub coding_rate: FlrcCodingRate,
    pub modulation_shaping: (),
}

/// Bit rate / bandwidht pairs for FLRC mode
#[derive(Clone, PartialEq, Debug)]
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
#[derive(Clone, PartialEq, Debug)]
pub enum FlrcCodingRate {
    Cr1_2 = 0x00,
    Cr3_4 = 0x02,
    Cr1_0 = 0x04,
}

/// FLRC sync word length
#[derive(Clone, PartialEq, Debug)]
pub enum FlrcSyncWordLength {
    None = 0x00,
    /// 4-byte sync word
    Length4 = 0x04,
}
