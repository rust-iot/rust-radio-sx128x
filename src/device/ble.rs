
use super::common::*;

/// Configuration for BLE mode
#[derive(Clone, PartialEq, Debug)]
pub struct BleConfig {
    pub bitrate_bandwidth: BleBitrate,
    pub modulation_index: BleModIndex,
    pub modulation_shaping: BleModShaping,
}

#[derive(Clone, PartialEq, Debug)]
pub struct BlePacketConfig {
    /// BLE connection state
    pub connection_state: BleConnectionStates,
    /// Size of the CRC block in the BLE packet
    pub crc_field: BleCrcFields,
    /// BLE packet types
    pub packet_type: BlePacketTypes,
    /// Whitening on PDU and CRC blocks of BLE packet
    pub whitening: WhiteningModes,
}

#[derive(Clone, PartialEq, Debug)]
pub enum BleBitrate {
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
pub enum BleModIndex {
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
pub enum BleModShaping {

}

#[derive(Clone, PartialEq, Debug)]
pub enum BleConnectionStates {

}

#[derive(Clone, PartialEq, Debug)]
pub enum BleCrcFields {

}

#[derive(Clone, PartialEq, Debug)]
pub enum BlePacketTypes {

}