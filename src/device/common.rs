

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

#[derive(Clone, PartialEq, Debug)]
pub enum GfskBleBitrate {
    GFSK_BLE_BR_2_000_BW_2_4                 = 0x04,
    GFSK_BLE_BR_1_600_BW_2_4                 = 0x28,
    GFSK_BLE_BR_1_000_BW_2_4                 = 0x4C,
    GFSK_BLE_BR_1_000_BW_1_2                 = 0x45,
    GFSK_BLE_BR_0_800_BW_2_4                 = 0x70,
    GFSK_BLE_BR_0_800_BW_1_2                 = 0x69,
    GFSK_BLE_BR_0_500_BW_1_2                 = 0x8D,
    GFSK_BLE_BR_0_500_BW_0_6                 = 0x86,
    GFSK_BLE_BR_0_400_BW_1_2                 = 0xB1,
    GFSK_BLE_BR_0_400_BW_0_6                 = 0xAA,
    GFSK_BLE_BR_0_250_BW_0_6                 = 0xCE,
    GFSK_BLE_BR_0_250_BW_0_3                 = 0xC7,
    GFSK_BLE_BR_0_125_BW_0_3                 = 0xEF,
}

#[derive(Clone, PartialEq, Debug)]
pub enum GfskBleModIndex {
    GFSK_BLE_MOD_IND_0_35                    =  0,
    GFSK_BLE_MOD_IND_0_50                    =  1,
    GFSK_BLE_MOD_IND_0_75                    =  2,
    GFSK_BLE_MOD_IND_1_00                    =  3,
    GFSK_BLE_MOD_IND_1_25                    =  4,
    GFSK_BLE_MOD_IND_1_50                    =  5,
    GFSK_BLE_MOD_IND_1_75                    =  6,
    GFSK_BLE_MOD_IND_2_00                    =  7,
    GFSK_BLE_MOD_IND_2_25                    =  8,
    GFSK_BLE_MOD_IND_2_50                    =  9,
    GFSK_BLE_MOD_IND_2_75                    = 10,
    GFSK_BLE_MOD_IND_3_00                    = 11,
    GFSK_BLE_MOD_IND_3_25                    = 12,
    GFSK_BLE_MOD_IND_3_50                    = 13,
    GFSK_BLE_MOD_IND_3_75                    = 14,
    GFSK_BLE_MOD_IND_4_00                    = 15,
}

#[derive(Clone, PartialEq, Debug)]
pub enum WhiteningModes {

}

#[derive(Clone, PartialEq, Debug)]
pub enum SyncWordRxMatch {

}

#[derive(Clone, PartialEq, Debug)]
pub enum HeaderType {

}

#[derive(Clone, PartialEq, Debug)]
pub enum CrcTypes {

}
