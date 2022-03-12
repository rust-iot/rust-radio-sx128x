//! Ble mode device configuration definitions

use super::common::*;

/// BLE operating mode channel configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BleChannel {
    /// Operating frequency
    pub freq: u32,
    /// Bitrate bandwidth
    pub br_bw: GfskBleBitrateBandwidth,
    /// Modulation index
    pub mi: GfskBleModIndex,
    /// Modulation shaping
    pub ms: ModShaping,
}

/// BLE operating mode packet configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BleConfig {
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BleConnectionStates {
    // TODO
    BLE_PAYLOAD_LENGTH_MAX_31_BYTES = 0x00,
    // TODO
    BLE_PAYLOAD_LENGTH_MAX_37_BYTES = 0x20,
    /// Transmit test mode
    BLE_TX_TEST_MODE = 0x40,
    /// TODO
    BLE_PAYLOAD_LENGTH_MAX_255_BYTES = 0x80,
}

/// BLE CRC field configuration
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BleCrcFields {
    /// CRC disabled
    BLE_CRC_OFF = 0x00,
    /// CRC 3B
    BLE_CRC_3B = 0x10,
}

/// BLE mode packet types
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BlePacketTypes {
    /// Pseudo Random Binary Sequence based on 9th degree polynomial
    BLE_PRBS_9 = 0x00,
    /// Pseudo Random Binary Sequence based on 15th degree polynomial
    BLE_PRBS_15 = 0x0C,
    /// Repeated '11110000' sequence
    BLE_EYELONG_1_0 = 0x04,
    /// Repeated '00001111' sequence
    BLE_EYELONG_0_1 = 0x18,
    /// Repeated '10101010' sequence
    BLE_EYESHORT_1_0 = 0x08,
    /// Repeated '01010101' sequence
    BLE_EYESHORT_0_1 = 0x1C,
    /// Repeated '11111111' sequence
    BLE_ALL_1 = 0x10,
    /// Repeated '00000000' sequence
    BLE_ALL_0 = 0x14,
}
