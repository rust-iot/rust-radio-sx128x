#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod ble;
use ble::BleConfig;
pub mod flrc;
use flrc::FlrcConfig;
pub mod gfsk;
use gfsk::GfskConfig;
pub mod lora;
use lora::LoRaConfig;

pub mod common;



/// Sx128x configuration object
#[derive(Clone, PartialEq, Debug)]
pub struct Config {
    pub regulator_mode: RegulatorMode,
    pub ramp_time: RampTime,
}

impl Default for Config {
    fn default() -> Self {
        Config{
            regulator_mode: RegulatorMode::Ldo,
            ramp_time: RampTime::Ramp04Us,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ModulationConfig {
    Gfsk(GfskConfig),
    LoRa(LoRaConfig),
    Flrc(FlrcConfig),
    Ble(BleConfig),
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
    Sleep = 0x00,
    StandbyRc,
    StandbyXosc,
    Fs,
    Tx,
    Rx,
    Cad,
}

#[derive(Clone, PartialEq, Debug)]
pub enum RegulatorMode {
    Ldo  = 0x00,
    Dcdc = 0x01,
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
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
    WiteRegister            = 0x18,
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
    LrFirmwareVersionMsb             = 0x0153,
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
    LrEstimatedFrequencyErrorMask      = 0x0FFFFF,
    LrSyncWordTolerance                = 0x09CD,
    LrBleAccessAddress                 = 0x09CF,
    LnaRegime                          = 0x0891,
    EnableManuaLGainControl            = 0x089F,
    DemodDetection                     = 0x0895,
    ManualGainValue                    = 0x089E,
}

pub const MASK_RANGINGMUXSEL: u8        = 0xCF;
pub const MASK_LNA_REGIME: u8           = 0xC0;
pub const MASK_MANUAL_GAIN_CONTROL: u8  = 0x80;
pub const MASK_DEMOD_DETECTION: u8      = 0xFE;
pub const MASK_MANUAL_GAIN_VALUE: u8    = 0xF0;


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
