//! Sx128x Radio Driver
// Copyright 2018 Ryan Kurte

#![no_std]
#![feature(associated_type_defaults)]

use core::convert::TryFrom;
use core::fmt::Debug;

extern crate libc;

#[cfg(any(test, feature = "util"))]
#[macro_use]
extern crate std;

use base::Base;

#[cfg(not(feature = "defmt"))]
use log::{debug, error, trace, warn};

#[cfg(feature = "defmt")]
use defmt::{trace, debug, error, warn};

use embedded_hal::delay::blocking::DelayUs;
use embedded_hal::digital::blocking::{InputPin, OutputPin};
use embedded_hal::spi::blocking::{Transactional, Transfer, Write};
use embedded_hal::spi::{Mode as SpiMode, Phase, Polarity};


pub use radio::{Channel as _, Interrupts as _, State as _};

pub mod base;

pub mod device;
use device::*;
pub use device::{Config, State};

pub mod prelude;

/// Sx128x Spi operating mode
pub const SPI_MODE: SpiMode = SpiMode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// Sx128x device object
pub struct Sx128x<Base> {
    config: Config,
    packet_type: PacketType,
    hal: Base,
}

pub const FREQ_MIN: u32 = 2_400_000_000;
pub const FREQ_MAX: u32 = 2_500_000_000;

pub const NUM_RETRIES: usize = 3;

/// Sx128x error type
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<CommsError: Debug + 'static, PinError: Debug + 'static, DelayError: Debug + 'static>
{
    #[cfg_attr(feature = "thiserror", error("communication error: {:?}", 0))]
    /// Communications (SPI or UART) error
    Comms(CommsError),

    #[cfg_attr(feature = "thiserror", error("pin error: {:?}", 0))]
    /// Pin control error
    Pin(PinError),

    #[cfg_attr(feature = "thiserror", error("delay error: {:?}", 0))]
    /// Delay error
    Delay(DelayError),

    #[cfg_attr(feature = "thiserror", error("transaction aborted"))]
    /// Transaction aborted
    Aborted,

    #[cfg_attr(feature = "thiserror", error("transaction timeout"))]
    /// Timeout by device
    Timeout,

    #[cfg_attr(feature = "thiserror", error("busy timeout"))]
    /// Timeout awaiting busy pin de-assert
    BusyTimeout,

    #[cfg_attr(feature = "thiserror", error("invalid message CRC"))]
    /// CRC error on received message
    InvalidCrc,

    #[cfg_attr(feature = "thiserror", error("invalid message length"))]
    /// Invalid message length
    InvalidLength,

    #[cfg_attr(feature = "thiserror", error("invalid sync word"))]
    /// TODO
    InvalidSync,

    #[cfg_attr(feature = "thiserror", error("transaction aborted"))]
    /// TODO
    Abort,

    #[cfg_attr(
        feature = "thiserror",
        error("invalid state (expected {:?} actual {:?})", 0, 1)
    )]
    /// TODO
    InvalidState(State, State),

    #[cfg_attr(
        feature = "thiserror",
        error("invalid device version (received {:?})", 0)
    )]
    /// Radio returned an invalid device firmware version
    InvalidDevice(u16),

    #[cfg_attr(
        feature = "thiserror",
        error("invalid circuit state (received {:?})", 0)
    )]
    /// Radio returned an invalid response
    InvalidCircuitState(u8),

    #[cfg_attr(
        feature = "thiserror",
        error("invalid command state (received {:?})", 0)
    )]
    /// Radio returned an invalid response
    InvalidCommandStatus(u8),

    #[cfg_attr(feature = "thiserror", error("invalid configuration"))]
    /// Invalid configuration option provided
    InvalidConfiguration,

    #[cfg_attr(feature = "thiserror", error("invalid state command"))]
    /// Invalid state command
    InvalidStateCommand,

    #[cfg_attr(
        feature = "thiserror",
        error("invalid frequency or frequency out of range")
    )]
    /// Frequency out of range
    InvalidFrequency,

    #[cfg_attr(feature = "thiserror", error("device communication failed"))]
    /// No SPI communication detected
    NoComms,
}

pub type Sx128xSpi<Spi, CsPin, BusyPin, ReadyPin, SdnPin, DelayPin> = Sx128x<Base<Spi, CsPin, BusyPin, ReadyPin, SdnPin, DelayPin>>;

/// Helper to group SPI functions by error, not needed when e-h@1.0.0-alpha.8 lands
pub trait SpiBase: Transfer<u8, Error = <Self as SpiBase>::Error> + Write<u8, Error = <Self as SpiBase>::Error> + Transactional<u8, Error = <Self as SpiBase>::Error> {
    type Error;
}

impl <T: Transfer<u8, Error = E> + Write<u8, Error = E> + Transactional<u8, Error = E>, E> SpiBase for T {
    type Error = E;
}

impl<Spi, CsPin, BusyPin, ReadyPin, SdnPin, PinError, Delay>
    Sx128x<
        Base<Spi, CsPin, BusyPin, ReadyPin, SdnPin, Delay>,
    >
where
    Spi: SpiBase,
    <Spi as SpiBase>::Error: Debug,

    CsPin: OutputPin<Error = PinError>,
    BusyPin: InputPin<Error = PinError>,
    ReadyPin: InputPin<Error = PinError>,
    SdnPin: OutputPin<Error = PinError>,
    PinError: Debug,

    Delay: DelayUs,
    <Delay as DelayUs>::Error: Debug,
{
    /// Create an Sx128x with the provided `Spi` implementation and pins
    pub fn spi(
        spi: Spi,
        cs: CsPin,
        busy: BusyPin,
        ready: ReadyPin,
        sdn: SdnPin,
        delay: Delay,
        config: &Config,
    ) -> Result<Self, Error<<Spi as SpiBase>::Error, PinError, <Delay as DelayUs>::Error>> {
        // Create SpiWrapper over spi/cs/busy
        let hal = Base{spi, cs, sdn, busy, ready, delay};
        // Create instance with new hal
        Self::new(hal, config)
    }
}

impl<Hal> Sx128x<Hal>
where
    Hal: base::Hal,
    <Hal as base::Hal>::CommsError: Debug + 'static,
    <Hal as base::Hal>::PinError: Debug + 'static,
    <Hal as base::Hal>::DelayError: Debug + 'static,
{
    /// Create a new Sx128x instance over a generic Hal implementation
    pub fn new(hal: Hal, config: &Config) -> Result<Self, Error<<Hal as base::Hal>::CommsError, <Hal as base::Hal>::PinError, <Hal as base::Hal>::DelayError>> {
        let mut sx128x = Self::build(hal);

        debug!("Resetting device");

        // Reset IC
        sx128x.hal.reset()?;

        debug!("Checking firmware version");

        // Check communication with the radio
        let firmware_version = sx128x.firmware_version()?;

        if firmware_version == 0xFFFF || firmware_version == 0x0000 {
            return Err(Error::NoComms);
        } else if firmware_version != 0xA9B5 {
            warn!(
                "Invalid firmware version! expected: 0x{:x} actual: 0x{:x}",
                0xA9B5, firmware_version
            );
        }

        if firmware_version != 0xA9B5 && !config.skip_version_check {
            // Disable version check. Known F/W version is: 0x8E8E
            // return Err(Error::InvalidDevice(firmware_version));
        }

        // TODO: do we need to calibrate things here?
        //sx128x.calibrate(CalibrationParams::default())?;

        debug!("Configuring device");

        // Configure device prior to use
        sx128x.configure(config)?;

        // Ensure state is idle
        sx128x.set_state(State::StandbyRc)?;

        Ok(sx128x)
    }

    pub fn reset(&mut self) -> Result<(), <Hal as base::HalError>::E> {
        debug!("Resetting device");

        self.hal.reset()?;

        Ok(())
    }

    pub(crate) fn build(hal: Hal) -> Self {
        Sx128x {
            config: Config::default(),
            packet_type: PacketType::None,
            hal,
        }
    }

    pub fn configure(
        &mut self,
        config: &Config,
    ) -> Result<(), <Hal as base::HalError>::E> {
        // Switch to standby mode
        self.set_state(State::StandbyRc)?;

        // Check configs match
        match (&config.modem, &config.channel) {
            (Modem::LoRa(_), Channel::LoRa(_)) => (),
            (Modem::Flrc(_), Channel::Flrc(_)) => (),
            (Modem::Gfsk(_), Channel::Gfsk(_)) => (),
            _ => return Err(Error::InvalidConfiguration),
        }

        // Update regulator mode
        self.set_regulator_mode(config.regulator_mode)?;
        self.config.regulator_mode = config.regulator_mode;

        // Update modem and channel configuration
        self.set_channel(&config.channel)?;
        self.config.channel = config.channel.clone();

        self.configure_modem(&config.modem)?;
        self.config.modem = config.modem.clone();

        // Update power amplifier configuration
        self.set_power_ramp(config.pa_config.power, config.pa_config.ramp_time)?;
        self.config.pa_config = config.pa_config.clone();

        Ok(())
    }

    pub fn firmware_version(&mut self) -> Result<u16, <Hal as base::HalError>::E> {
        let mut d = [0u8; 2];

        self.hal
            .read_regs(Registers::LrFirmwareVersionMsb as u16, &mut d)?;

        Ok((d[0] as u16) << 8 | (d[1] as u16))
    }

    pub fn set_frequency(&mut self, f: u32) -> Result<(), <Hal as base::HalError>::E> {
        let c = self.config.freq_to_steps(f as f32) as u32;

        trace!("Setting frequency ({:?} MHz, {} index)", f / 1000 / 1000, c);

        let data: [u8; 3] = [(c >> 16) as u8, (c >> 8) as u8, (c >> 0) as u8];

        self.hal.write_cmd(Commands::SetRfFrequency as u8, &data)
    }

    pub(crate) fn set_power_ramp(
        &mut self,
        power: i8,
        ramp: RampTime,
    ) -> Result<(), <Hal as base::HalError>::E> {
        if power > 13 || power < -18 {
            warn!("TX power out of range (-18 < p < 13)");
        }

        // Limit to -18 to +13 dBm
        let power = core::cmp::max(power, -18);
        let power = core::cmp::min(power, 13);
        let power_reg = (power + 18) as u8;

        trace!(
            "Setting TX power to {} dBm {:?} ramp ({}, {})",
            power,
            ramp,
            power_reg,
            ramp as u8
        );
        self.config.pa_config.power = power;
        self.config.pa_config.ramp_time = ramp;

        self.hal
            .write_cmd(Commands::SetTxParams as u8, &[power_reg, ramp as u8])
    }

    /// Set IRQ mask
    pub fn set_irq_mask(
        &mut self,
        irq: Irq,
    ) -> Result<(), <Hal as base::HalError>::E> {
        trace!("Setting IRQ mask: {:?}", irq);

        let raw = irq.bits();
        self.hal.write_cmd(
            Commands::SetDioIrqParams as u8,
            &[(raw >> 8) as u8, (raw & 0xff) as u8],
        )
    }

    /// Set the IRQ and DIO masks
    pub fn set_irq_dio_mask(
        &mut self,
        irq: Irq,
        dio1: DioMask,
        dio2: DioMask,
        dio3: DioMask,
    ) -> Result<(), <Hal as base::HalError>::E> {
        trace!(
            "Setting IRQ mask: {:?} DIOs: {:?}, {:?}, {:?}",
            irq,
            dio1,
            dio2,
            dio3
        );

        let raw_irq = irq.bits();
        let raw_dio1 = dio1.bits();
        let raw_dio2 = dio2.bits();
        let raw_dio3 = dio3.bits();

        let data = [
            (raw_irq >> 8) as u8,
            (raw_irq & 0xff) as u8,
            (raw_dio1 >> 8) as u8,
            (raw_dio1 & 0xff) as u8,
            (raw_dio2 >> 8) as u8,
            (raw_dio2 & 0xff) as u8,
            (raw_dio3 >> 8) as u8,
            (raw_dio3 & 0xff) as u8,
        ];

        self.hal.write_cmd(Commands::SetDioIrqParams as u8, &data)
    }

    pub(crate) fn configure_modem(
        &mut self,
        config: &Modem,
    ) -> Result<(), <Hal as base::HalError>::E> {
        use Modem::*;

        debug!("Setting modem config: {:?}", config);

        // First update packet type (if required)
        let packet_type = PacketType::from(config);
        if self.packet_type != packet_type {
            trace!("Setting packet type: {:?}", packet_type);
            self.hal
                .write_cmd(Commands::SetPacketType as u8, &[packet_type.clone() as u8])?;
            self.packet_type = packet_type;
        }

        let data = match config {
            Gfsk(c) => [
                c.preamble_length as u8,
                c.sync_word_length as u8,
                c.sync_word_match as u8,
                c.header_type as u8,
                c.payload_length as u8,
                c.crc_mode as u8,
                c.whitening as u8,
            ],
            LoRa(c) | Ranging(c) => [
                c.preamble_length as u8,
                c.header_type as u8,
                c.payload_length as u8,
                c.crc_mode as u8,
                c.invert_iq as u8,
                0u8,
                0u8,
            ],
            Flrc(c) => [
                c.preamble_length as u8,
                c.sync_word_length as u8,
                c.sync_word_match as u8,
                c.header_type as u8,
                c.payload_length as u8,
                c.crc_mode as u8,
                c.whitening as u8,
            ],
            Ble(c) => [
                c.connection_state as u8,
                c.crc_field as u8,
                c.packet_type as u8,
                c.whitening as u8,
                0u8,
                0u8,
                0u8,
            ],
            None => [0u8; 7],
        };

        self.hal.write_cmd(Commands::SetPacketParams as u8, &data)?;

        // Apply patches
        match config {
            Flrc(c) if c.patch_syncword => {
                // Apply sync-word patch for FLRC mode
                self.patch_flrc_syncword()?;
            }
            Gfsk(c) if c.patch_preamble => {
                // Write preamble length for GFSK mode
                self.hal.write_reg(
                    Registers::GfskBlePreambleLength as u16,
                    c.preamble_length as u8,
                )?;
            }
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn get_rx_buffer_status(
        &mut self,
    ) -> Result<(u8, u8), <Hal as base::HalError>::E> {
        use device::lora::LoRaHeader;

        let mut status = [0u8; 2];

        self.hal
            .read_cmd(Commands::GetRxBufferStatus as u8, &mut status)?;

        let len = match &self.config.modem {
            Modem::LoRa(c) => match c.header_type {
                LoRaHeader::Implicit => self.hal.read_reg(Registers::LrPayloadLength as u16)?,
                LoRaHeader::Explicit => status[0],
            },
            // BLE status[0] does not include 2-byte PDU header
            Modem::Ble(_) => status[0] + 2,
            _ => status[0],
        };

        let rx_buff_ptr = status[1];

        trace!("RX buffer ptr: {} len: {}", rx_buff_ptr, len);

        Ok((rx_buff_ptr, len))
    }

    pub(crate) fn get_packet_info(
        &mut self,
        info: &mut PacketInfo,
    ) -> Result<(), <Hal as base::HalError>::E> {
        let mut data = [0u8; 5];
        self.hal
            .read_cmd(Commands::GetPacketStatus as u8, &mut data)?;

        info.packet_status = PacketStatus::from_bits_truncate(data[2]);
        info.tx_rx_status = TxRxStatus::from_bits_truncate(data[3]);
        info.sync_addr_status = data[4] & 0b0111;

        match self.packet_type {
            PacketType::Gfsk | PacketType::Flrc | PacketType::Ble => {
                info.rssi = -(data[1] as i16) / 2;
                let rssi_avg = -(data[0] as i16) / 2;
                trace!("Raw RSSI: {}", info.rssi);
                trace!("Average RSSI: {}", rssi_avg);
            }
            PacketType::LoRa | PacketType::Ranging => {
                info.rssi = -(data[0] as i16) / 2;
                info.snr = Some(match data[1] < 128 {
                    true => data[1] as i16 / 4,
                    false => (data[1] as i16 - 256) / 4,
                });
            }
            PacketType::None => unimplemented!(),
        }

        debug!("Info: {:?}", info);

        Ok(())
    }

    pub fn calibrate(
        &mut self,
        c: CalibrationParams,
    ) -> Result<(), <Hal as base::HalError>::E> {
        trace!("Calibrate {:?}", c);
        self.hal.write_cmd(Commands::Calibrate as u8, &[c.bits()])
    }

    pub(crate) fn set_regulator_mode(
        &mut self,
        r: RegulatorMode,
    ) -> Result<(), <Hal as base::HalError>::E> {
        trace!("Set regulator mode {:?}", r);
        self.hal
            .write_cmd(Commands::SetRegulatorMode as u8, &[r as u8])
    }

    // TODO: this could got into a mode config object maybe?
    #[allow(dead_code)]
    pub(crate) fn set_auto_tx(
        &mut self,
        a: AutoTx,
    ) -> Result<(), <Hal as base::HalError>::E> {
        let data = match a {
            AutoTx::Enabled(timeout_us) => {
                let compensated = timeout_us - AUTO_RX_TX_OFFSET;
                [(compensated >> 8) as u8, (compensated & 0xff) as u8]
            }
            AutoTx::Disabled => [0u8; 2],
        };
        self.hal.write_cmd(Commands::SetAutoTx as u8, &data)
    }

    pub(crate) fn set_buff_base_addr(
        &mut self,
        tx: u8,
        rx: u8,
    ) -> Result<(), <Hal as base::HalError>::E> {
        trace!("Set buff base address (tx: {}, rx: {})", tx, rx);
        self.hal
            .write_cmd(Commands::SetBufferBaseAddress as u8, &[tx, rx])
    }

    /// Set the sychronization mode for a given index (1-3).
    /// This is 5-bytes for GFSK mode and 4-bytes for FLRC and BLE modes.
    pub fn set_syncword(
        &mut self,
        index: u8,
        value: &[u8],
    ) -> Result<(), <Hal as base::HalError>::E> {
        trace!(
            "Attempting to set sync word index: {} to: {:?}",
            index,
            value
        );

        // Check sync words for errata 16.4
        if self.packet_type == PacketType::Flrc {
            match &value[0..2] {
                &[0x8C, 0x32] | &[0x63, 0x0E] => {
                    error!("Invalid sync word selected (see errata 16.4)");
                    return Err(Error::InvalidConfiguration);
                }
                _ => (),
            }
        }

        // Calculate sync word base address and expected length
        let (addr, len) = match (&self.packet_type, index) {
            (PacketType::Gfsk, 1) => (Registers::LrSyncWordBaseAddress1 as u16, 5),
            (PacketType::Gfsk, 2) => (Registers::LrSyncWordBaseAddress2 as u16, 5),
            (PacketType::Gfsk, 3) => (Registers::LrSyncWordBaseAddress3 as u16, 5),
            (PacketType::Flrc, 1) => (Registers::LrSyncWordBaseAddress1 as u16 + 1, 4),
            (PacketType::Flrc, 2) => (Registers::LrSyncWordBaseAddress2 as u16 + 1, 4),
            (PacketType::Flrc, 3) => (Registers::LrSyncWordBaseAddress3 as u16 + 1, 4),
            (PacketType::Ble, _) => (Registers::LrSyncWordBaseAddress1 as u16 + 1, 4),
            _ => {
                warn!(
                    "Invalid sync word configuration (mode: {:?} index: {} value: {:?}",
                    self.config.modem, index, value
                );
                return Err(Error::InvalidConfiguration);
            }
        };

        // Check length is correct
        if value.len() != len {
            warn!(
                "Incorrect sync word length for mode: {:?} (actual: {}, expected: {})",
                self.config.modem,
                value.len(),
                len
            );
            return Err(Error::InvalidConfiguration);
        }

        // Write sync word
        self.hal.write_regs(addr, value)?;

        Ok(())
    }

    /// Apply patch for sync-word match errata in FLRC mode
    fn patch_flrc_syncword(&mut self) -> Result<(), <Hal as base::HalError>::E> {
        // If we're in FLRC mode, patch to force 100% match on syncwords
        // because otherwise the 4 bit threshold is too low
        if let PacketType::Flrc = &self.packet_type {
            let r = self.hal.read_reg(Registers::LrSyncWordTolerance as u16)?;
            self.hal
                .write_reg(Registers::LrSyncWordTolerance as u16, r & 0xF0)?;
        }

        Ok(())
    }
}

impl<Hal> DelayUs for Sx128x<Hal>
where
    Hal: base::Hal,
{
    type Error = <Hal as base::HalError>::E;

    fn delay_us(&mut self, t: u32) -> Result<(), Self::Error> {
        self.hal.delay_us(t).map_err(|e| Error::Delay(e))
    }
}

/// `radio::State` implementation for the SX128x
impl<Hal> radio::State
    for Sx128x<Hal>
where
    Hal: base::Hal,
{
    type State = State;
    type Error = <Hal as base::HalError>::E;

    /// Fetch device state
    fn get_state(&mut self) -> Result<Self::State, Self::Error> {
        let mut d = [0u8; 1];
        self.hal.read_cmd(Commands::GetStatus as u8, &mut d)?;

        trace!("raw state: {}", d[0]);

        let mode = (d[0] & 0b1110_0000) >> 5;
        let m = State::try_from(mode).map_err(|_| Error::InvalidCircuitState(d[0]))?;

        let status = (d[0] & 0b0001_1100) >> 2;
        let s = CommandStatus::try_from(status).map_err(|_| Error::InvalidCommandStatus(d[0]))?;

        trace!("get state: {:?} status: {:?}", m, s);

        Ok(m)
    }

    /// Set device state
    fn set_state(&mut self, state: Self::State) -> Result<(), Self::Error> {
        let command = match state {
            State::Tx => Commands::SetTx,
            State::Rx => Commands::SetRx,
            //State::Cad => Commands::SetCad,
            State::Fs => Commands::SetFs,
            State::StandbyRc | State::StandbyXosc => Commands::SetStandby,
            State::Sleep => Commands::SetSleep,
            #[cfg(feature = "patch-unknown-state")]
            State::Unknown => return Err(Error::InvalidStateCommand),
        };

        trace!("Setting state {:?} ({})", state, command);

        self.hal.write_cmd(command as u8, &[0u8])
    }
}

/// `radio::Busy` implementation for the SX128x
impl<Hal> radio::Busy
    for Sx128x<Hal>
where
    Hal: base::Hal,
{
    type Error = <Hal as base::HalError>::E;

    /// Fetch device state
    fn is_busy(&mut self) -> Result<bool, Self::Error> {
        let irq = self.get_interrupts(false)?;

        if irq.contains(Irq::SYNCWORD_VALID)
            && !(irq.contains(Irq::RX_DONE) || irq.contains(Irq::CRC_ERROR))
        {
            return Ok(true);
        }

        Ok(false)
    }
}

/// `radio::Channel` implementation for the SX128x
impl<Hal> radio::Channel
    for Sx128x<Hal>
where
    Hal: base::Hal,
{
    /// Channel consists of an operating frequency and packet mode
    type Channel = Channel;

    type Error = <Hal as base::HalError>::E;

    /// Set operating channel
    fn set_channel(&mut self, ch: &Self::Channel) -> Result<(), Self::Error> {
        use Channel::*;

        debug!("Setting channel config: {:?}", ch);

        // Set frequency
        let freq = ch.frequency();
        if freq < FREQ_MIN || freq > FREQ_MAX {
            return Err(Error::InvalidFrequency);
        }

        self.set_frequency(freq)?;

        // First update packet type (if required)
        let packet_type = PacketType::from(ch);
        if self.packet_type != packet_type {
            self.hal
                .write_cmd(Commands::SetPacketType as u8, &[packet_type.clone() as u8])?;
            self.packet_type = packet_type;
        }

        // Then write modulation configuration
        let data = match ch {
            Gfsk(c) => [c.br_bw as u8, c.mi as u8, c.ms as u8],
            LoRa(c) | Ranging(c) => [c.sf as u8, c.bw as u8, c.cr as u8],
            Flrc(c) => [c.br_bw as u8, c.cr as u8, c.ms as u8],
            Ble(c) => [c.br_bw as u8, c.mi as u8, c.ms as u8],
        };

        self.hal
            .write_cmd(Commands::SetModulationParams as u8, &data)
    }
}

/// `radio::Power` implementation for the SX128x
impl<Hal> radio::Power
    for Sx128x<Hal>
where
    Hal: base::Hal,
{
    type Error = <Hal as base::HalError>::E;

    /// Set TX power in dBm
    fn set_power(&mut self, power: i8) -> Result<(), <Hal as base::HalError>::E> {
        let ramp_time = self.config.pa_config.ramp_time;
        self.set_power_ramp(power, ramp_time)
    }
}

/// `radio::Interrupts` implementation for the SX128x
impl<Hal> radio::Interrupts
    for Sx128x<Hal>
where
    Hal: base::Hal,
{
    type Irq = Irq;
    type Error = <Hal as base::HalError>::E;

    /// Fetch (and optionally clear) current interrupts
    fn get_interrupts(&mut self, clear: bool) -> Result<Self::Irq, Self::Error> {
        let mut data = [0u8; 2];

        self.hal.read_cmd(Commands::GetIrqStatus as u8, &mut data)?;
        let irq = Irq::from_bits((data[0] as u16) << 8 | data[1] as u16).unwrap();

        if clear && !irq.is_empty() {
            self.hal.write_cmd(Commands::ClearIrqStatus as u8, &data)?;
        }

        if !irq.is_empty() {
            trace!("irq: {:?}", irq);
        }

        Ok(irq)
    }
}

/// `radio::Transmit` implementation for the SX128x
impl<Hal> radio::Transmit
    for Sx128x<Hal>
where
    Hal: base::Hal,
{
    type Error = <Hal as base::HalError>::E;

    /// Start transmitting a packet
    fn start_transmit(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        debug!("TX start");

        // Set state to idle before we write configuration
        self.set_state(State::StandbyRc)?;

        let s = self.get_state()?;
        debug!("TX setup state: {:?}", s);

        // Set packet mode
        let mut modem_config = self.config.modem.clone();
        modem_config.set_payload_len(data.len() as u8);

        if let Err(e) = self.configure_modem(&modem_config) {
            if let Ok(s) = self.get_state() {
                error!(
                    "TX error setting modem (state: {:?})",
                    s
                );
            } else {
                error!(
                    "TX error setting modem",
                );
            }
            return Err(e);
        }

        // Reset buffer addr
        if let Err(e) = self.set_buff_base_addr(0, 0) {
            if let Ok(s) = self.get_state() {
                error!(
                    "TX error setting buffer base addr (state: {:?})",
                    s
                );
            } else {
                error!(
                    "TX error setting buffer base addr",
                );
            }

            return Err(e);
        }

        // Write data to be sent
        debug!("TX data: {:?}", data);
        self.hal.write_buff(0, data)?;

        // Configure ranging if used
        if PacketType::Ranging == self.packet_type {
            self.hal.write_cmd(
                Commands::SetRangingRole as u8,
                &[RangingRole::Initiator as u8],
            )?;
        }

        // Setup timout
        let config = [
            self.config.rf_timeout.step() as u8,
            ((self.config.rf_timeout.count() >> 8) & 0x00FF) as u8,
            (self.config.rf_timeout.count() & 0x00FF) as u8,
        ];

        // Enable IRQs
        let irqs = Irq::TX_DONE | Irq::CRC_ERROR | Irq::RX_TX_TIMEOUT;
        self.set_irq_dio_mask(irqs, irqs, DioMask::empty(), DioMask::empty())?;

        // Enter transmit mode
        self.hal.write_cmd(Commands::SetTx as u8, &config)?;

        trace!("TX start issued");

        let state = self.get_state()?;
        trace!("State: {:?}", state);

        Ok(())
    }

    /// Check for transmit completion
    fn check_transmit(&mut self) -> Result<bool, Self::Error> {
        // Poll on DIO and short-circuit if not asserted
        #[cfg(feature = "poll_irq")]
        if self.hal.get_dio()? == PinState::Low {
            return Ok(false);
        }

        let irq = self.get_interrupts(true)?;
        let state = self.get_state()?;

        trace!("TX poll (irq: {:?}, state: {:?})", irq, state);

        if irq.contains(Irq::TX_DONE) {
            debug!("TX complete");
            Ok(true)
        } else if irq.contains(Irq::RX_TX_TIMEOUT) {
            debug!("TX timeout");
            Err(Error::Timeout)
        } else {
            Ok(false)
        }
    }
}

/// `radio::Receive` implementation for the SX128x
impl<Hal> radio::Receive
    for Sx128x<Hal>
where
    Hal: base::Hal,
{
    /// Receive info structure
    type Info = PacketInfo;

    /// RF Error object
    type Error = <Hal as base::HalError>::E;

    /// Start radio in receive mode
    fn start_receive(&mut self) -> Result<(), Self::Error> {
        debug!("RX start");

        // Set state to idle before we write configuration
        self.set_state(State::StandbyRc)?;

        let s = self.get_state()?;
        debug!("RX setup state: {:?}", s);

        // Reset buffer addr
        if let Err(e) = self.set_buff_base_addr(0, 0) {
            if let Ok(s) = self.get_state() {
                error!(
                    "RX error setting buffer base addr (state: {:?})",
                    s
                );
            } else {
                error!(
                    "RX error setting buffer base addr",
                );
            }
            return Err(e);
        }

        // Set packet mode
        // TODO: surely this should not bre required _every_ receive?
        let modem_config = self.config.modem.clone();

        if let Err(e) = self.configure_modem(&modem_config) {
            if let Ok(s) = self.get_state() {
                error!(
                    "RX error setting configuration (state: {:?})",
                    s
                );
            } else {
                error!(
                    "RX error setting configuration",
                );
            }
            return Err(e);
        }

        // Configure ranging if used
        if PacketType::Ranging == self.packet_type {
            self.hal.write_cmd(
                Commands::SetRangingRole as u8,
                &[RangingRole::Responder as u8],
            )?;
        }

        // Setup timout
        let config = [
            self.config.rf_timeout.step() as u8,
            ((self.config.rf_timeout.count() >> 8) & 0x00FF) as u8,
            (self.config.rf_timeout.count() & 0x00FF) as u8,
        ];

        // Enable IRQs
        let irqs = Irq::RX_DONE
            | Irq::CRC_ERROR
            | Irq::RX_TX_TIMEOUT
            | Irq::SYNCWORD_VALID
            | Irq::SYNCWORD_ERROR
            | Irq::HEADER_VALID
            | Irq::HEADER_ERROR
            | Irq::PREAMBLE_DETECTED;

        self.set_irq_dio_mask(irqs, irqs, DioMask::empty(), DioMask::empty())?;

        // Enter transmit mode
        self.hal.write_cmd(Commands::SetRx as u8, &config)?;

        let state = self.get_state()?;

        debug!("RX started (state: {:?})", state);

        Ok(())
    }

    /// Check for a received packet
    fn check_receive(&mut self, restart: bool) -> Result<bool, Self::Error> {
        // Poll on DIO and short-circuit if not asserted
        #[cfg(feature = "poll_irq")]
        if self.hal.get_dio()? == PinState::Low {
            return Ok(false);
        }

        let irq = self.get_interrupts(true)?;
        let mut res = Ok(false);

        trace!("RX poll (irq: {:?})", irq);

        // Process flags
        if irq.contains(Irq::CRC_ERROR) {
            debug!("RX CRC error");
            res = Err(Error::InvalidCrc);
        } else if irq.contains(Irq::RX_TX_TIMEOUT) {
            debug!("RX timeout");
            res = Err(Error::Timeout);
        } else if irq.contains(Irq::SYNCWORD_ERROR) {
            debug!("Invalid syncword");
            res = Err(Error::InvalidSync);
        } else if irq.contains(Irq::RX_DONE) {
            debug!("RX complete");
            res = Ok(true);
        }

        // Auto-restart on failure if enabled
        match (restart, res) {
            (true, Err(_)) => {
                debug!("RX restarting");
                self.start_receive()?;
                Ok(false)
            }
            (_, r) => r,
        }
    }

    /// Fetch a received packet
    fn get_received<'a>(&mut self, data: &'a mut [u8]) -> Result<(usize, Self::Info), Self::Error> {
        // Fetch RX buffer information
        let (ptr, len) = self.get_rx_buffer_status()?;

        debug!("RX get received, ptr: {} len: {}", ptr, len);

        if data.len() < len as usize {
            return Err(Error::InvalidLength);
        }

        // TODO: check error packet status byte to ensure CRC is valid
        // as this may not result in a CRC error IRQ.
        // See chip errata for further details

        // Read from the buffer at the provided pointer
        self.hal.read_buff(ptr, &mut data[..len as usize])?;

        // Fetch related information
        let mut info = Self::Info::default();
        self.get_packet_info(&mut info)?;

        trace!("RX data: {:?} info: {:?}", &data[..len as usize], info);

        // Return read length
        Ok((len as usize, info))
    }
}

/// `radio::Rssi` implementation for the SX128x
impl<Hal> radio::Rssi
    for Sx128x<Hal>
where
    Hal: base::Hal,
{
    type Error = <Hal as base::HalError>::E;

    /// Poll for the current channel RSSI
    /// This should only be called when in receive mode
    fn poll_rssi(&mut self) -> Result<i16, <Hal as base::HalError>::E> {
        let mut raw = [0u8; 1];
        self.hal.read_cmd(Commands::GetRssiInst as u8, &mut raw)?;
        Ok(-(raw[0] as i16) / 2)
    }
}

#[cfg(all(feature = "std", test))]
mod tests {
    use crate::base::Hal;
    use crate::device::RampTime;
    use crate::Sx128x;

    use driver_pal::mock::{Mock, Spi};

    use radio::State as _;

    pub mod vectors;

    #[test]
    #[ignore] // Ignored awaiting further driver-pal revision
    fn test_api_reset() {
        let mut m = Mock::new();
        let (spi, sdn, _busy, delay) = (m.spi(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::<Spi, _, _, _>::build(spi.clone());

        m.expect(vectors::reset(&spi, &sdn, &delay));
        radio.hal.reset().unwrap();
        m.finalise();
    }

    #[test]
    #[ignore] // Ignored awaiting further driver-pal revision
    fn test_api_status() {
        let mut m = Mock::new();
        let (spi, sdn, _busy, delay) = (m.spi(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::<Spi, _, _, _>::build(spi.clone());

        m.expect(vectors::status(&spi, &sdn, &delay));
        radio.get_state().unwrap();
        m.finalise();
    }

    #[test]
    #[ignore] // Ignored awaiting further driver-pal revision
    fn test_api_firmware_version() {
        let mut m = Mock::new();
        let (spi, sdn, _busy, delay) = (m.spi(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::<Spi, _, _, _>::build(spi.clone());

        m.expect(vectors::firmware_version(&spi, &sdn, &delay, 16));
        let version = radio.firmware_version().unwrap();
        m.finalise();
        assert_eq!(version, 16);
    }

    #[test]
    #[ignore] // Ignored awaiting further driver-pal revision
    fn test_api_power_ramp() {
        let mut m = Mock::new();
        let (spi, sdn, _busy, delay) = (m.spi(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::<Spi, _, _, _>::build(spi.clone());

        m.expect(vectors::set_power_ramp(&spi, &sdn, &delay, 0x1f, 0xe0));
        radio.set_power_ramp(13, RampTime::Ramp20Us).unwrap();
        m.finalise();
    }
}
