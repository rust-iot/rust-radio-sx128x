

/// Modulation shaping parameter for GFSK, FLRC and BLE modes
#[derive(Clone, PartialEq, Debug)]
pub enum ModShaping {
    /// No filtering
    BtOFF                = 0x00,
    Bt1_0                = 0x10,
    Bt0_5                = 0x20,
}

/// Preamble lengths for GFSK, FLRC modes
#[derive(Clone, PartialEq, Debug)]
pub enum PreambleLength {
    /// Preamble length: 04 bits
    PreambleLength04                 = 0x00,
    /// Preamble length: 08 bits
    PreambleLength08                 = 0x10,
    /// Preamble length: 12 bits
    PreambleLength12                 = 0x20,
    /// Preamble length: 16 bits
    PreambleLength16                 = 0x30,
    /// Preamble length: 20 bits
    PreambleLength20                 = 0x40,
    /// Preamble length: 24 bits
    PreambleLength24                 = 0x50,
    /// Preamble length: 28 bits
    PreambleLength28                 = 0x60,
    /// Preamble length: 32 bits
    PreambleLength32                 = 0x70,
}
