//! Sx128x Radio Driver
// Copyright 2018 Ryan Kurte

#![no_std]

use core::marker::PhantomData;

extern crate libc;

extern crate log;

#[cfg(test)]
#[macro_use]
extern crate std;

#[macro_use]
extern crate bitflags;

extern crate embedded_hal as hal;
use hal::blocking::{delay};
use hal::digital::v2::{InputPin, OutputPin};
use hal::spi::{Mode as SpiMode, Phase, Polarity};
use hal::blocking::spi::{Transfer, Write};

extern crate embedded_spi;
use embedded_spi::{Error as WrapError, wrapper::Wrapper as SpiWrapper};

extern crate radio;
use radio::{Transmit, Receive, Channel, Interrupts};

pub mod base;
use base::Hal;

pub mod device;
use device::*;

/// Sx128x Spi operating mode
pub const SPI_MODE: SpiMode = SpiMode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// Sx128x device object
#[repr(C)]
pub struct Sx128x<Base, CommsError, PinError> {
    config: Config,
    
    hal: Base,
    settings: Settings,

    _ce: PhantomData<CommsError>, 
    _pe: PhantomData<PinError>,
}

pub struct Settings {
    pub xtal_freq: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            xtal_freq: 52000000
        }
    }
}

impl Settings {
    // Calculate frequency step for a given crystal frequency
    fn freq_step(&self) -> u32 {
        self.xtal_freq >> 18
    }

    fn freq_stepf(&self) -> f32 {
        self.xtal_freq as f32 / 2f32.powi(18)
    }

    fn freq_to_steps(&self, f: f32) -> f32 {
        f / self.freq_stepf()
    }

    fn steps_to_freq(&self, s: f32) -> f32 {
        s * self.freq_stepf()
    }
}

/// Sx128x error type
#[derive(Debug, Clone, PartialEq)]
pub enum Error<CommsError, PinError> {
    /// Communications (SPI or UART) error
    Comms(CommsError),
    /// Pin control error
    Pin(PinError),
    /// Transaction aborted
    Aborted,
}

impl <CommsError, PinError> From<WrapError<CommsError, PinError>> for Error<CommsError, PinError> {
    fn from(e: WrapError<CommsError, PinError>) -> Self {
        match e {
            WrapError::Spi(e) => Error::Comms(e),
            WrapError::Pin(e) => Error::Pin(e),
            WrapError::Aborted => Error::Aborted,
        }
    }
}

impl<Spi, CommsError, Output, Input, PinError, Delay> Sx128x<SpiWrapper<Spi, CommsError, Output, Input, PinError, Delay>, CommsError, PinError>
where
    Spi: Transfer<u8, Error = CommsError> + Write<u8, Error = CommsError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{
    /// Create an Sx128x with the provided `Spi` implementation and pins
    pub fn spi(spi: Spi, cs: Output, busy: Input, sdn: Output, delay: Delay, settings: Settings) -> Result<Self, Error<CommsError, PinError>> {
        // Create SpiWrapper over spi/cs/busy
        let mut hal = SpiWrapper::new(spi, cs, delay);
        hal.with_busy(busy);
        hal.with_reset(sdn);
        // Create instance with new hal
        Self::new(hal, settings)
    }
}


impl<Hal, CommsError, PinError> Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    /// Create a new Sx128x instance over a generic Hal implementation
    pub fn new(hal: Hal, settings: Settings) -> Result<Self, Error<CommsError, PinError>> {

        let mut sx128x = Self::build(hal, settings);

        // Reset IC
        sx128x.hal.reset()?;

        // Calibrate RX chain
        //sx1280::RxChainCalibration(&sx128x.c);

        // Init IRQs (..?)

        // Confiure modem(s)

        // Set state to idle


        Ok(sx128x)
    }

    pub(crate) fn build(hal: Hal, settings: Settings) -> Self {
        Sx128x { 
            config: Config::default(),
            hal,
            settings,
            _ce: PhantomData,
            _pe: PhantomData,
        }
    }

    pub fn get_status(&mut self) -> Result<u8, Error<CommsError, PinError>> {
        let mut d = [0u8; 1];
        self.hal.read_cmd(Commands::GetStatus as u8, &mut d)?;
        Ok(d[0])
    }

    pub fn firmware_version(&mut self) -> Result<u16, Error<CommsError, PinError>> {
        let mut d = [0u8; 2];

        self.hal.read_regs(Registers::LrFirmwareVersionMsb as u16, &mut d)?;

        Ok((d[0] as u16) << 8 | (d[1] as u16))
    }

    pub fn set_frequency(&mut self, f: u32) -> Result<(), Error<CommsError, PinError>> {
        let c = self.settings.freq_to_steps(f as f32) as u32;

        let data: [u8; 3] = [
            (c >> 16) as u8,
            (c >> 8) as u8,
            (c >> 0) as u8,
        ];

        self.hal.write_cmd(Commands::SetRfFrequency as u8, &data)
    }

    pub fn set_power(&mut self, power: i8) -> Result<(), Error<CommsError, PinError>> {
        // Limit to -18 to +13 dBm
        let power = core::cmp::min(power, -18);
        let power = core::cmp::max(power, 13);
        let power = (power + 18) as u8;

        self.hal.write_cmd(Commands::SetTxParams as u8, &[power])
    }

    pub(crate) fn write_modulation_params(&mut self, modulation: ModulationConfig) -> Result<(), Error<CommsError, PinError>> {
        use ModulationConfig::*;

        // First update packet type
        let packet_type = PacketType::from(&modulation);
        if self.config.packet_type != packet_type {
            self.hal.write_cmd(Commands::SetPacketType as u8, &[ packet_type.clone() as u8 ] )?;
            self.config.packet_type = packet_type;
        }

        // Then write modulation configuration
        let data = match &modulation {
            Gfsk(c) => [c.bitrate_bandwidth as u8, c.modulation_index as u8, c.modulation_shaping as u8],
            LoRa(c) => [c.spreading_factor as u8, c.bandwidth as u8, c.coding_rate as u8],
            Flrc(c) => [c.bitrate_bandwidth as u8, c.coding_rate as u8, c.modulation_shaping as u8],
            Ble(c) => [c.bitrate_bandwidth as u8, c.modulation_index as u8, c.modulation_shaping as u8],
        };

        self.hal.write_cmd(Commands::SetModulationParams as u8, &data)
    }

    pub(crate) fn write_packet_params(&mut self, packet: PacketConfig) -> Result<(), Error<CommsError, PinError>> {
        use PacketConfig::*;

        // First update packet type
        let packet_type = PacketType::from(&packet);
        if self.config.packet_type != packet_type {
            self.hal.write_cmd(Commands::SetPacketType as u8, &[ packet_type.clone() as u8 ] )?;
            self.config.packet_type = packet_type;
        }

        let data = match &packet {
            Gfsk(c) => [c.preamble_length as u8, c.sync_word_length as u8, c.sync_word_match as u8, c.header_type as u8, c.payload_length as u8, c.crc_type as u8, c.whitening as u8],
            LoRa(c) => [c.preamble_length as u8, c.header_type as u8, c.payload_length as u8, c.crc_mode as u8, c.invert_iq as u8, 0u8, 0u8],
            Flrc(c) => [c.preamble_length as u8, c.sync_word_length as u8, c.sync_word_match as u8, c.header_type as u8, c.payload_length as u8, c.crc_type as u8, c.whitening as u8],
            Ble(c) => [c.connection_state as u8, c.crc_field as u8, c.packet_type as u8, c.whitening as u8, 0u8, 0u8, 0u8],
            None => [0u8; 7],
        };
        self.hal.write_cmd(Commands::SetPacketParams as u8, &data)
    }

    pub(crate) fn get_rx_buffer_status(&mut self) -> Result<(), Error<CommsError, PinError>> {
        use device::lora::LoRaHeader;

        let mut status = [0u8; 2];

        self.hal.read_cmd(Commands::GetRxBufferStatus as u8, &mut status)?;

        let len = match self.config.mode.packet_config() {
            PacketConfig::LoRa(c) => {
                match c.header_type {
                    LoRaHeader::Implicit => self.hal.read_reg(Registers::LrPayloadLength as u8)?,
                    LoRaHeader::Explicit => status[0],
                }
            },
            // BLE status[0] does not include 2-byte PDU header
            PacketConfig::Ble(_) => status[0] + 2,
            _ => status[0]
        };

        let _rx_buff_ptr = status[1];

        Ok(())
    }

    
    pub(crate) fn get_packet_info(&mut self, info: &mut Info) -> Result<(), Error<CommsError, PinError>> {

        let mut data = [0u8; 5];
        self.hal.read_cmd(Commands::GetPacketStatus as u8, &mut data)?;

        info.packet_status = PacketStatus::from_bits(data[2]).unwrap();
        info.tx_rx_status = TxRxStatus::from_bits(data[3]).unwrap();
        info.sync_addr_status = data[4] & 0x70;

        match self.config.mode.packet_type() {
            PacketType::Gfsk | PacketType::Flrc | PacketType::Ble => {
                info.rssi = -(data[0] as i16) / 2;
                info.rssi_sync = Some(-(data[1] as i16) / 2);
            },
            PacketType::LoRa | PacketType::Ranging => {
                info.rssi = -(data[0] as i16) / 2;
                info.snr = Some(match data[1] < 128 {
                    true => data[1] as i16 / 4,
                    false => ( data[1] as i16 - 256 ) / 4
                });
            },
            PacketType::None => unimplemented!(),
        }

        Ok(())
    }

    pub(crate) fn get_rssi(&mut self) -> Result<i16, Error<CommsError, PinError>> {
        let mut raw = [0u8; 1];
        self.hal.read_cmd(Commands::GetRssiInst as u8, &mut raw)?;
        Ok(-(raw[0] as i16) / 2)
    }

    pub(crate) fn get_irq(&mut self, clear: bool) -> Result<Irq, Error<CommsError, PinError>> {
        let mut raw = [0u8; 2];
        // Read IRQ flags
        self.hal.read_cmd(Commands::GetIrqStatus as u8, &mut raw)?;
        let irq = Irq::from_bits((raw[0] as u16) << 8 | (raw[1] as u16)).unwrap();
        // Clear register if requested
        if clear {
            self.hal.write_cmd(Commands::ClearIrqStatus as u8, &raw)?;
        }
        // Return IRQ object
        Ok(irq)
    }

    pub(crate) fn calibrate(&mut self, c: CalibrationParams) -> Result<(), Error<CommsError, PinError>> {
        self.hal.write_cmd(Commands::Calibrate as u8, &[ c.bits() ])
    }

    pub(crate) fn set_regulator_mode(&mut self, r: RegulatorMode) -> Result<(), Error<CommsError, PinError>> {
        self.hal.write_cmd(Commands::SetRegulatorMode as u8, &[ r as u8 ])
    }

    pub(crate) fn set_auto_tx(&mut self, a: AutoTx) -> Result<(), Error<CommsError, PinError>> {
        let data = match a {
            AutoTx::Enabled(timeout_us) => {
                let compensated = timeout_us - AUTO_RX_TX_OFFSET;
                [(compensated >> 8) as u8, (compensated & 0xff) as u8]
            },
            AutoTx::Disabled => [0u8; 2],
        };
        self.hal.write_cmd(Commands::SetAutoTx as u8, &data)
    }



}

#[derive(Clone, Debug, PartialEq)]
pub struct Info {
    rssi: i16,
    rssi_sync: Option<i16>,
    snr: Option<i16>,

    packet_status: PacketStatus,
    tx_rx_status: TxRxStatus,
    sync_addr_status: u8,
}

impl<Hal, CommsError, PinError> Channel for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    type Channel = ();
    type Error = Error<CommsError, PinError>;

    fn set_channel(&mut self, ch: &Self::Channel) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

impl<Hal, CommsError, PinError> Interrupts for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    type Irq = ();
    type Error = Error<CommsError, PinError>;

    fn get_interrupts(&mut self, clear: bool) -> Result<Self::Irq, Self::Error> {
        unimplemented!()
    }
}

impl<Hal, CommsError, PinError> Transmit for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    type Error = Error<CommsError, PinError>;

    fn start_transmit(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn check_transmit(&mut self) -> Result<bool, Self::Error> {
        unimplemented!()
    }
}

impl<Hal, CommsError, PinError> Receive for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    type Info = ();
    type Error = Error<CommsError, PinError>;

    fn start_receive(&mut self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn check_receive(&mut self, restart: bool) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn get_received<'a>(&mut self, info: &mut Self::Info, data: &'a mut [u8]) -> Result<usize, Self::Error> {
        unimplemented!()
    }

}

#[cfg(test)]
mod tests {
    use crate::{Sx128x, Settings};
    use crate::base::Hal;

    extern crate embedded_spi;
    use self::embedded_spi::mock::{Mock, Spi, Pin};

    pub mod vectors;

    #[test]
    fn test_api_reset() {
        let mut m = Mock::new();
        let (spi, sdn, busy, delay) = (m.spi(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::<Spi, _, _>::build(spi.clone(), Settings::default());

        m.expect(vectors::reset(&spi, &sdn, &delay));
        radio.hal.reset().unwrap();
        m.finalise();
    }

    #[test]
    fn test_api_status() {
        let mut m = Mock::new();
        let (spi, sdn, busy, delay) = (m.spi(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::<Spi, _, _>::build(spi.clone(), Settings::default());

        m.expect(vectors::status(&spi, &sdn, &delay));
        radio.get_status().unwrap();
        m.finalise();
    }

    #[test]
    fn test_api_firmware_version() {
        let mut m = Mock::new();
        let (spi, sdn, busy, delay) = (m.spi(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::<Spi, _, _>::build(spi.clone(), Settings::default());

        m.expect(vectors::firmware_version(&spi, &sdn, &delay, 16));
        let version = radio.firmware_version().unwrap();
        m.finalise();
        assert_eq!(version, 16);
    }
}
