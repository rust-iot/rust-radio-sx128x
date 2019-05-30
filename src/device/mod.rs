#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod ble;
use ble::{BleConfig, BlePacketConfig};
pub mod flrc;
use flrc::{FlrcConfig, FlrcPacketConfig};
pub mod gfsk;
use gfsk::{GfskConfig, GfskPacketConfig};
pub mod lora;
use lora::{LoRaConfig, LoRaPacketConfig};

pub mod common;



/// Sx128x configuration object
#[derive(Clone, PartialEq, Debug)]
pub struct Config {
    pub frequency: u32,
    pub regulator_mode: RegulatorMode,
    pub pa_config: PaConfig,
    pub(crate) packet_type: PacketType,
    pub modulation_config: ModulationMode,
    pub packet_config: PacketMode,
    pub timeout: Timeout,
}

impl Default for Config {
    fn default() -> Self {
        Config{
            frequency: 2.4e9 as u32,
            regulator_mode: RegulatorMode::Dcdc,
            pa_config: PaConfig{ power: 10, ramp_time: RampTime::Ramp20Us },
            packet_type: PacketType::None,
            modulation_config: ModulationMode::LoRa(LoRaConfig::default()),
            packet_config: PacketMode::LoRa(LoRaPacketConfig::default()),
            //timeout: Timeout::Configurable{ step: TickSize::TickSize1000us, count: 1000 },
            timeout: Timeout::Single,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct PaConfig {
    pub power: i8,
    pub ramp_time: RampTime,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ModulationMode {
    Gfsk(GfskConfig),
    LoRa(LoRaConfig),
    Flrc(FlrcConfig),
    Ble(BleConfig),
    Ranging(LoRaConfig),
}

impl From<&ModulationMode> for PacketType {
    fn from(m: &ModulationMode) -> Self {
         match m {
            ModulationMode::Gfsk(_) => PacketType::Gfsk,
            ModulationMode::LoRa(_) => PacketType::LoRa,
            ModulationMode::Ranging(_) => PacketType::LoRa,
            ModulationMode::Flrc(_) => PacketType::Flrc,
            ModulationMode::Ble(_) => PacketType::Ble,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum PacketMode {
    Gfsk(GfskPacketConfig),
    LoRa(LoRaPacketConfig),
    Flrc(FlrcPacketConfig),
    Ble(BlePacketConfig),
    Ranging(LoRaPacketConfig),
    None,
}

impl PacketMode {
    pub fn set_payload_len(&mut self, len: u8) {
        match self {
            PacketMode::Gfsk(c) => c.payload_length = len,
            PacketMode::LoRa(c) => c.payload_length = len,
            PacketMode::Flrc(c) => c.payload_length = len,
            _ => (),
        }
    }
}

impl From<&PacketMode> for PacketType {
    fn from(m: &PacketMode) -> Self {
         match m {
            PacketMode::Gfsk(_) => PacketType::Gfsk,
            PacketMode::LoRa(_) => PacketType::LoRa,
            PacketMode::Ranging(_) => PacketType::LoRa,
            PacketMode::Flrc(_) => PacketType::Flrc,
            PacketMode::Ble(_) => PacketType::Ble,
            PacketMode::None => PacketType::None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Channel {

}

#[derive(Clone, PartialEq, Debug)]
pub enum State {
    Idle = 0x00,
    Rx,
    Tx,
    Cad,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Mode {
    Sleep       = 0x00,
    StandbyRc   = 0x01,
    StandbyXosc = 0x02,
    Fs          = 0x03,
    Tx          = 0x04,
    Rx          = 0x05,
    Cad         = 0x06,
}

impl core::convert::TryFrom<u8> for Mode {
    type Error = ();

    fn try_from(v: u8) -> Result<Mode, ()> {
        match v {
            0x00 => Ok(Mode::Sleep),
            0x01 => Ok(Mode::StandbyRc),
            0x02 => Ok(Mode::StandbyXosc),
            0x03 => Ok(Mode::Fs),
            0x04 => Ok(Mode::Tx),
            0x05 => Ok(Mode::Rx),
            0x06 => Ok(Mode::Cad),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RegulatorMode {
    Ldo  = 0x00,
    Dcdc = 0x01,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RampTime {
    Ramp02Us = 0x00,
    Ramp04Us = 0x20,
    Ramp06Us = 0x40,
    Ramp08Us = 0x60,
    Ramp10Us = 0x80,
    Ramp12Us = 0xA0,
    Ramp16Us = 0xC0,
    Ramp20Us = 0xE0,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PacketType {
    Gfsk     = 0x00,
    LoRa     = 0x01,
    Ranging  = 0x02,
    Flrc     = 0x03,
    Ble      = 0x04,
    None     = 0x0F,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Commands {
    GetStatus                = 0xC0,
    WiteRegister             = 0x18,
    ReadRegister             = 0x19,
    WriteBuffer              = 0x1A,
    ReadBuffer               = 0x1B,
    SetSleep                 = 0x84,
    SetStandby               = 0x80,
    SetFs                    = 0xC1,
    SetTx                    = 0x83,
    SetRx                    = 0x82,
    SetRxDutyCycle           = 0x94,
    SetCad                   = 0xC5,
    SetTxContinuousWave      = 0xD1,
    SetTxContinuousPreamble  = 0xD2,
    SetPacketType            = 0x8A,
    GetPacketType            = 0x03,
    SetRfFrequency           = 0x86,
    SetTxParams              = 0x8E,
    SetCadParams             = 0x88,
    SetBufferBaseAddress     = 0x8F,
    SetModulationParams      = 0x8B,
    SetPacketParams          = 0x8C,
    GetRxBufferStatus        = 0x17,
    GetPacketStatus          = 0x1D,
    GetRssiInst              = 0x1F,
    SetDioIrqParams          = 0x8D,
    GetIrqStatus             = 0x15,
    ClearIrqStatus           = 0x97,
    Calibrate                = 0x89,
    SetRegulatorMode         = 0x96,
    SetSaveContext           = 0xD5,
    SetAutoTx                = 0x98,
    SetAutoFs                = 0x9E,
    SetLongPreamble          = 0x9B,
    SetUartSpeed             = 0x9D,
    SetRangingRole           = 0xA3,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Registers {
    LrFirmwareVersionMsb               = 0x0153,
    LrCrcSeedBaseAddr                  = 0x09C8,
    LrCrcPolyBaseAddr                  = 0x09C6,
    LrWhitSeedBaseAddr                 = 0x09C5,
    LrRangingIdCheckLength             = 0x0931,
    LrDeviceRangingAddr                = 0x0916,
    LrRequestRangingAddr               = 0x0912,
    LrRangingResultConfig              = 0x0924,
    LrRangingResultBaseAddr            = 0x0961,
    LrRangingResultsFreeze             = 0x097F,
    LrRangingReRxTxDelayCal            = 0x092C,
    LrRangingFilterWindowSize          = 0x091E,
    LrRangingResultClearReg            = 0x0923,
    RangingRssi                        = 0x0964,
    LrPacketParams                     = 0x903,
    LrPayloadLength                    = 0x901,
    LrSyncWordBaseAddress1             = 0x09CE,
    LrSyncWordBaseAddress2             = 0x09D3,
    LrSyncWordBaseAddress3             = 0x09D8,
    LrEstimatedFrequencyErrorMsb       = 0x0954,
    LrSyncWordTolerance                = 0x09CD,
    LrBleAccessAddress                 = 0x09CF,
    LnaRegime                          = 0x0891,
    EnableManuaLGainControl            = 0x089F,
    DemodDetection                     = 0x0895,
    ManualGainValue                    = 0x089E,
}

pub const MASK_RANGINGMUXSEL: u8       = 0xCF;
pub const MASK_LNA_REGIME: u8          = 0xC0;
pub const MASK_MANUAL_GAIN_CONTROL: u8 = 0x80;
pub const MASK_DEMOD_DETECTION: u8     = 0xFE;
pub const MASK_MANUAL_GAIN_VALUE: u8   = 0xF0;

pub const MASK_LR_ESTIMATED_FREQUENCY_ERROR: u32 = 0x0FFFFF;

pub const AUTO_RX_TX_OFFSET: u16 = 33;

#[derive(Clone, PartialEq, Debug)]
pub enum AutoTx {
    /// Enable AutoTX with the provided timeout in microseconds (uS)
    Enabled(u16),
    /// Disable AutoTx
    Disabled,
}

bitflags! {
    /// Interrupt flags register 
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


bitflags! {
    /// Packet status register
    pub struct PacketStatus: u8 {
        const SYNC_ERROR            = 1 >> 6;
        const LENGTH_ERROR          = 1 >> 5;
        const CRC_ERROR             = 1 >> 4;
        const ABORT_ERROR           = 1 >> 3;
        const HEADER_RECEIVED       = 1 >> 2;
        const PACKET_RECEIVED       = 1 >> 1;
        const PACKET_CONTROLER_BUSY = 1 >> 0;
    }
}


bitflags! {
    /// TxRx status register
    pub struct TxRxStatus: u8 {
        const SYNC_ERROR            = 1 >> 6;
        const LENGTH_ERROR          = 1 >> 5;
        const CRC_ERROR             = 1 >> 4;
        const ABORT_ERROR           = 1 >> 3;
        const HEADER_RECEIVED       = 1 >> 2;
        const PACKET_RECEIVED       = 1 >> 1;
        const PACKET_CONTROLER_BUSY = 1 >> 0;
    }
}

bitflags! {
    /// TxRx status register
    pub struct SyncAddrStatus: u8 {
        const SYNC_ERROR            = 1 >> 6;
    }
}


bitflags! {
    /// Radio calibration parameters
    pub struct CalibrationParams: u8 {
        const ADCBulkPEnable    = (1 >> 5);
        const ADCBulkNEnable    = (1 >> 4);
        const ADCPulseEnable    = (1 >> 3);
        const PLLEnable         = (1 >> 2);
        const RC13MEnable       = (1 >> 1);
        const RC64KEnable       = (1 >> 0);
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RangingRole {
    Responder = 0x00,
    Initiator = 0x01,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TickSize {
    TickSize0015us   = 0x00,
    TickSize0062us   = 0x01,
    TickSize1000us   = 0x02,
    TickSize4000us   = 0x03,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Timeout {
    Single,
    Configurable {
        step: TickSize,
        count: u16,
    },
    Continuous,
}

impl Timeout {
    pub fn step(&self) -> TickSize  {
        match self {
            Timeout::Single          => TickSize::TickSize0015us,
            Timeout::Configurable{step, count: _} => *step,
            Timeout::Continuous      => TickSize::TickSize0015us,
        }
    }

    pub fn count(&self) -> u16 {
        match self {
            Timeout::Single          => 0x0000,
            Timeout::Configurable{step, count} => *count,
            Timeout::Continuous      => 0xFFFF,
        }
    }
}
