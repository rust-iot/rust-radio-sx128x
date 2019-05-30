
use super::common::*;

/// Configuration for BLE mode
#[derive(Clone, PartialEq, Debug)]
pub struct BleConfig {
    pub bitrate_bandwidth: GfskBleBitrate,
    pub modulation_index: GfskBleModIndex,
    pub modulation_shaping: ModShaping,
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BleConnectionStates {
    BLE_PAYLOAD_LENGTH_MAX_31_BYTES         = 0x00,
    BLE_PAYLOAD_LENGTH_MAX_37_BYTES         = 0x20,
    BLE_TX_TEST_MODE                        = 0x40,
    BLE_PAYLOAD_LENGTH_MAX_255_BYTES        = 0x80,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BleCrcFields {
    BLE_CRC_OFF                             = 0x00,
    BLE_CRC_3B                              = 0x10,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BlePacketTypes {
    /// Pseudo Random Binary Sequence based on 9th degree polynomial
    BLE_PRBS_9                              = 0x00,
    /// Pseudo Random Binary Sequence based on 15th degree polynomial
    BLE_PRBS_15                             = 0x0C,
    /// Repeated '11110000' sequence
    BLE_EYELONG_1_0                         = 0x04,
    /// Repeated '00001111' sequence
    BLE_EYELONG_0_1                         = 0x18,
    /// Repeated '10101010' sequence
    BLE_EYESHORT_1_0                        = 0x08,
    /// Repeated '01010101' sequence
    BLE_EYESHORT_0_1                        = 0x1C,
    /// Repeated '11111111' sequence
    BLE_ALL_1                               = 0x10,
    /// Repeated '00000000' sequence
    BLE_ALL_0                               = 0x14,
}