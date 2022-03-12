#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use bitflags::bitflags;
use log::error;

pub mod ble;
use ble::{BleChannel, BleConfig};
pub mod flrc;
use flrc::{FlrcChannel, FlrcConfig};
pub mod gfsk;
use gfsk::{GfskChannel, GfskConfig};
pub mod lora;
use lora::{LoRaChannel, LoRaConfig};

pub mod common;

pub const BUSY_TIMEOUT_MS: u32 = 500;

/// Sx128x general configuration object
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Regulator mode configuration
    pub regulator_mode: RegulatorMode,

    /// Power amplifier configuration
    pub pa_config: PaConfig,

    /// Internal packet type field to track configurations
    pub(crate) packet_type: PacketType,

    /// RF Modem configuration
    ///
    /// (note this must match the modulation configuration)
    pub modem: Modem,

    /// RF Modulation / Channel configuration
    ///
    /// (note this must match the packet configuration)
    pub channel: Channel,

    /// RF timeout configuration
    ///
    /// Note that in high-traffic conditions Timeout::Single must be used
    /// to avoid the radio becoming unresponsive, see the chip errata for
    /// further details
    pub rf_timeout: Timeout,

    /// Crystal oscillator frequency
    pub xtal_freq: u32,

    /// Timeout for blocking / polling internal methods
    pub timeout_ms: u32,

    /// Skip firmware version validation
    pub skip_version_check: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            regulator_mode: RegulatorMode::Ldo,
            pa_config: PaConfig {
                power: 10,
                ramp_time: RampTime::Ramp20Us,
            },
            packet_type: PacketType::None,
            modem: Modem::LoRa(LoRaConfig::default()),
            channel: Channel::LoRa(LoRaChannel::default()),
            //timeout: Timeout::Configurable{ step: TickSize::TickSize1000us, count: 1000 },
            rf_timeout: Timeout::Single,
            xtal_freq: 52000000,
            timeout_ms: 100,
            skip_version_check: false,
        }
    }
}

impl Config {
    /// Create a default FLRC configuration
    pub fn flrc() -> Self {
        Config {
            packet_type: PacketType::Flrc,
            modem: Modem::Flrc(FlrcConfig::default()),
            channel: Channel::Flrc(FlrcChannel::default()),
            ..Default::default()
        }
    }

    /// Create a default GFSK configuration
    pub fn gfsk() -> Self {
        Config {
            packet_type: PacketType::Gfsk,
            modem: Modem::Gfsk(GfskConfig::default()),
            channel: Channel::Gfsk(GfskChannel::default()),
            ..Default::default()
        }
    }

    /// Create a default LoRa configuration
    pub fn lora() -> Self {
        Config {
            packet_type: PacketType::LoRa,
            modem: Modem::LoRa(LoRaConfig::default()),
            channel: Channel::LoRa(LoRaChannel::default()),
            ..Default::default()
        }
    }
}

impl Config {
    /// Calculate frequency step for a given crystal frequency
    pub fn freq_step(&self) -> f32 {
        self.xtal_freq as f32 / (2u32 << 17) as f32
    }

    /// Convert a provided frequency into configuration steps
    pub fn freq_to_steps(&self, f: f32) -> f32 {
        f / self.freq_step() as f32
    }
}

/// Radio modem configuration contains fields for each modem mode
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Modem {
    Gfsk(GfskConfig),
    LoRa(LoRaConfig),
    Flrc(FlrcConfig),
    Ble(BleConfig),
    Ranging(LoRaConfig),
    None,
}

impl Modem {
    pub fn set_payload_len(&mut self, len: u8) {
        match self {
            Modem::Gfsk(c) => c.payload_length = len,
            Modem::LoRa(c) => c.payload_length = len,
            Modem::Flrc(c) => c.payload_length = len,
            _ => (),
        }
    }
}

impl From<&Modem> for PacketType {
    fn from(m: &Modem) -> Self {
        match m {
            Modem::Gfsk(_) => PacketType::Gfsk,
            Modem::LoRa(_) => PacketType::LoRa,
            Modem::Ranging(_) => PacketType::LoRa,
            Modem::Flrc(_) => PacketType::Flrc,
            Modem::Ble(_) => PacketType::Ble,
            Modem::None => PacketType::None,
        }
    }
}

/// Radio channel configuration contains channel options for each mode
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Channel {
    Gfsk(GfskChannel),
    LoRa(LoRaChannel),
    Flrc(FlrcChannel),
    Ble(BleChannel),
    Ranging(LoRaChannel),
}

impl Default for Channel {
    fn default() -> Self {
        Channel::LoRa(LoRaChannel::default())
    }
}

impl Channel {
    /// Fetch frequency for a given modulation configuration
    pub fn frequency(&self) -> u32 {
        use Channel::*;

        match self {
            Gfsk(c) => c.freq,
            LoRa(c) => c.freq,
            Flrc(c) => c.freq,
            Ble(c) => c.freq,
            Ranging(c) => c.freq,
        }
    }
}

impl From<&Channel> for PacketType {
    fn from(m: &Channel) -> Self {
        use Channel::*;

        match m {
            Gfsk(_) => PacketType::Gfsk,
            LoRa(_) => PacketType::LoRa,
            Ranging(_) => PacketType::LoRa,
            Flrc(_) => PacketType::Flrc,
            Ble(_) => PacketType::Ble,
        }
    }
}

/// Radio state
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    Sleep = 0x00,
    StandbyRc = 0x02,
    StandbyXosc = 0x03,
    Fs = 0x04,
    Rx = 0x05,
    Tx = 0x06,
    #[cfg(feature = "patch-unknown-state")]
    /// Unknown state not specified in datasheet but occurs in some conditions..?
    /// See: https://github.com/rust-iot/rust-radio-sx128x/pull/76
    Unknown = 0x07,
}

impl radio::RadioState for State {
    fn idle() -> Self {
        Self::StandbyXosc
    }

    fn sleep() -> Self {
        Self::Sleep
    }
}

impl core::convert::TryFrom<u8> for State {
    type Error = ();

    fn try_from(v: u8) -> Result<State, ()> {
        match v {
            0x00 => Ok(State::Sleep),
            0x02 => Ok(State::StandbyRc),
            0x03 => Ok(State::StandbyXosc),
            0x04 => Ok(State::Fs),
            0x05 => Ok(State::Rx),
            0x06 => Ok(State::Tx),
            #[cfg(feature = "patch-unknown-state")]
            0x07 => Ok(State::Unknown),
            _ => {
                error!("Unrecognised state 0x{:x}", v);
                Err(())
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]

pub enum CommandStatus {
    Reserved = 0x0,
    Success = 0x1,
    DataAvailable = 0x2,
    Timeout = 0x3,
    ProcessingError = 0x4,
    ExecutionFailure = 0x5,
    TxDone = 0x6,
}

impl core::convert::TryFrom<u8> for CommandStatus {
    type Error = ();

    fn try_from(v: u8) -> Result<CommandStatus, ()> {
        match v {
            0x00 => Ok(CommandStatus::Reserved),
            0x01 => Ok(CommandStatus::Success),
            0x02 => Ok(CommandStatus::DataAvailable),
            0x03 => Ok(CommandStatus::Timeout),
            0x04 => Ok(CommandStatus::ProcessingError),
            0x05 => Ok(CommandStatus::ExecutionFailure),
            0x06 => Ok(CommandStatus::TxDone),
            _ => {
                error!("Unrecognised status {:x}", v);
                Err(())
            }
        }
    }
}

/// Power Amplifier configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PaConfig {
    /// Power in dBm
    pub power: i8,
    /// Ramp time for power amplifier
    pub ramp_time: RampTime,
}

/// Receive packet information
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PacketInfo {
    pub rssi: i16,
    pub rssi_sync: Option<i16>,
    pub snr: Option<i16>,

    pub packet_status: PacketStatus,
    pub tx_rx_status: TxRxStatus,
    pub sync_addr_status: u8,
}

impl radio::ReceiveInfo for PacketInfo {
    fn rssi(&self) -> i16 {
        self.rssi
    }
}

impl Default for PacketInfo {
    fn default() -> Self {
        Self {
            rssi: -100,
            rssi_sync: None,
            snr: None,
            packet_status: PacketStatus::empty(),
            tx_rx_status: TxRxStatus::empty(),
            sync_addr_status: 0,
        }
    }
}

/// Regulator operating mode
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RegulatorMode {
    /// Internal LDO
    Ldo = 0x00,
    /// Internal DC/DC converter
    Dcdc = 0x01,
}

/// Power amplifier ramp time
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RampTime {
    /// Ramp over 2us
    Ramp02Us = 0x00,
    /// Ramp over 4us
    Ramp04Us = 0x20,
    /// Ramp over 6us
    Ramp06Us = 0x40,
    /// Ramp over 8us
    Ramp08Us = 0x60,
    /// Ramp over 10us
    Ramp10Us = 0x80,
    /// Ramp over 12us
    Ramp12Us = 0xA0,
    /// Ramp over 16us
    Ramp16Us = 0xC0,
    /// Ramp over 20us
    Ramp20Us = 0xE0,
}

/// Packet type enumeration
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PacketType {
    Gfsk = 0x00,
    LoRa = 0x01,
    Ranging = 0x02,
    Flrc = 0x03,
    Ble = 0x04,
    None = 0x0F,
}

/// Radio commands
#[derive(Clone, PartialEq, Debug, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Commands {
    GetStatus = 0xC0,
    WiteRegister = 0x18,
    ReadRegister = 0x19,
    WriteBuffer = 0x1A,
    ReadBuffer = 0x1B,
    SetSleep = 0x84,
    SetStandby = 0x80,
    SetFs = 0xC1,
    SetTx = 0x83,
    SetRx = 0x82,
    SetRxDutyCycle = 0x94,
    SetCad = 0xC5,
    SetTxContinuousWave = 0xD1,
    SetTxContinuousPreamble = 0xD2,
    SetPacketType = 0x8A,
    GetPacketType = 0x03,
    SetRfFrequency = 0x86,
    SetTxParams = 0x8E,
    SetCadParams = 0x88,
    SetBufferBaseAddress = 0x8F,
    SetModulationParams = 0x8B,
    SetPacketParams = 0x8C,
    GetRxBufferStatus = 0x17,
    GetPacketStatus = 0x1D,
    GetRssiInst = 0x1F,
    SetDioIrqParams = 0x8D,
    GetIrqStatus = 0x15,
    ClearIrqStatus = 0x97,
    Calibrate = 0x89,
    SetRegulatorMode = 0x96,
    SetSaveContext = 0xD5,
    SetAutoTx = 0x98,
    SetAutoFs = 0x9E,
    SetLongPreamble = 0x9B,
    SetUartSpeed = 0x9D,
    SetRangingRole = 0xA3,
}

/// Radio registers
#[derive(Clone, PartialEq, Debug, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Registers {
    LrFirmwareVersionMsb = 0x0153,
    LrCrcSeedBaseAddr = 0x09C8,
    LrCrcPolyBaseAddr = 0x09C6,
    LrWhitSeedBaseAddr = 0x09C5,
    LrRangingIdCheckLength = 0x0931,
    LrDeviceRangingAddr = 0x0916,
    LrRequestRangingAddr = 0x0912,
    LrRangingResultConfig = 0x0924,
    LrRangingResultBaseAddr = 0x0961,
    LrRangingResultsFreeze = 0x097F,
    LrRangingReRxTxDelayCal = 0x092C,
    LrRangingFilterWindowSize = 0x091E,
    LrRangingResultClearReg = 0x0923,
    RangingRssi = 0x0964,
    LrPacketParams = 0x903,
    LrPayloadLength = 0x901,
    LrSyncWordBaseAddress1 = 0x09CE,
    LrSyncWordBaseAddress2 = 0x09D3,
    LrSyncWordBaseAddress3 = 0x09D8,
    LrEstimatedFrequencyErrorMsb = 0x0954,
    GfskBlePreambleLength = 0x09C1,
    LrSyncWordTolerance = 0x09CD,
    LrBleAccessAddress = 0x09CF,
    LnaRegime = 0x0891,
    EnableManuaLGainControl = 0x089F,
    DemodDetection = 0x0895,
    ManualGainValue = 0x089E,
}

pub const MASK_RANGINGMUXSEL: u8 = 0xCF;
pub const MASK_LNA_REGIME: u8 = 0xC0;
pub const MASK_MANUAL_GAIN_CONTROL: u8 = 0x80;
pub const MASK_DEMOD_DETECTION: u8 = 0xFE;
pub const MASK_MANUAL_GAIN_VALUE: u8 = 0xF0;

pub const MASK_LR_ESTIMATED_FREQUENCY_ERROR: u32 = 0x0FFFFF;

pub const AUTO_RX_TX_OFFSET: u16 = 33;

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AutoTx {
    /// Enable AutoTX with the provided timeout in microseconds (uS)
    Enabled(u16),
    /// Disable AutoTx
    Disabled,
}

bitflags! {
    /// Interrupt flags register
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Irq: u16 {
        const TX_DONE                             = 0x0001;
        const RX_DONE                             = 0x0002;
        const SYNCWORD_VALID                      = 0x0004;
        const SYNCWORD_ERROR                      = 0x0008;
        const HEADER_VALID                        = 0x0010;
        const HEADER_ERROR                        = 0x0020;
        const CRC_ERROR                           = 0x0040;
        const RANGING_SLAVE_RESPONSE_DONE         = 0x0080;
        const RANGING_SLAVE_REQUEST_DISCARDED     = 0x0100;
        const RANGING_MASTER_RESULT_VALID         = 0x0200;
        const RANGING_MASTER_RESULT_TIMEOUT       = 0x0400;
        const RANGING_SLAVE_REQUEST_VALID         = 0x0800;
        const CAD_DONE                            = 0x1000;
        const CAD_ACTIVITY_DETECTED               = 0x2000;
        const RX_TX_TIMEOUT                       = 0x4000;
        const PREAMBLE_DETECTED                   = 0x8000;
    }
}

/// DIO IRQ flag mask
pub type DioMask = Irq;

bitflags! {
    /// Packet status register
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct PacketStatus: u8 {
        /// Top flag value unknown due to lack of complete datasheet
        const UNKNOWN               = (1 << 7);
        const SYNC_ERROR            = (1 << 6);
        const LENGTH_ERROR          = (1 << 5);
        const CRC_ERROR             = (1 << 4);
        const ABORT_ERROR           = (1 << 3);
        const HEADER_RECEIVED       = (1 << 2);
        const PACKET_RECEIVED       = (1 << 1);
        const PACKET_CONTROLER_BUSY = (1 << 0);
    }
}

bitflags! {
    /// TxRx status packet status byte
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct TxRxStatus: u8 {
        /// Top flag value unknown due to lack of complete datasheet
        const RX_NO_ACK             = (1 << 5);
        const PACKET_SENT           = (1 << 0);
    }
}

bitflags! {
    /// TxRx status register
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct SyncAddrStatus: u8 {
        const SYNC_ERROR            = (1 << 6);
    }
}

bitflags! {
    /// Radio calibration parameters
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct CalibrationParams: u8 {
        const ADCBulkPEnable    = (1 << 5);
        const ADCBulkNEnable    = (1 << 4);
        const ADCPulseEnable    = (1 << 3);
        const PLLEnable         = (1 << 2);
        const RC13MEnable       = (1 << 1);
        const RC64KEnable       = (1 << 0);
    }
}

/// Ranging mode role
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RangingRole {
    /// Responder listens for ranging requests and responds
    Responder = 0x00,
    /// Initiator sends ranging requests and awaits responses
    Initiator = 0x01,
}

/// TickSize for timeout calculations
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TickSize {
    // 15us tick size
    TickSize0015us = 0x00,
    // 62us tick size
    TickSize0062us = 0x01,
    // 1000us tick size
    TickSize1000us = 0x02,
    // 4000us tick size
    TickSize4000us = 0x03,
}

/// Timeout confguration for autonomous radio operations
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Timeout {
    /// Single tx/rx mode
    Single,
    // Configurable timeout
    Configurable {
        /// Timeout step size
        step: TickSize,
        /// Number of steps to timeout
        count: u16,
    },
    /// Continuous rx/tx mode
    Continuous,
}

impl Timeout {
    /// Fetch the TickSize from a timeout configuration
    pub fn step(&self) -> TickSize {
        match self {
            Timeout::Single => TickSize::TickSize0015us,
            Timeout::Configurable { step, count: _ } => *step,
            Timeout::Continuous => TickSize::TickSize0015us,
        }
    }

    /// Fetch the step count for a timeout configuration
    pub fn count(&self) -> u16 {
        match self {
            Timeout::Single => 0x0000,
            Timeout::Configurable { step: _, count } => *count,
            Timeout::Continuous => 0xFFFF,
        }
    }
}
