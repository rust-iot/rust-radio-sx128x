//! Sx128x Radio Driver
// Copyright 2018 Ryan Kurte

#![no_std]

use core::marker::PhantomData;
use core::convert::TryFrom;

extern crate libc;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

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
pub use radio::{Transmit, Receive, Channel, Power, Interrupts, Rssi};

pub mod base;

pub mod device;
pub use device::Mode;
use device::*;
pub use device::Config;

/// Sx128x Spi operating mode
pub const SPI_MODE: SpiMode = SpiMode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// Sx128x device object
pub struct Sx128x<Base, CommsError, PinError> {
    config: Config,
    
    hal: Base,
    settings: Settings,

    _ce: PhantomData<CommsError>, 
    _pe: PhantomData<PinError>,
}

#[derive(Clone, PartialEq, Debug)]
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
    /// Timeout by device
    Timeout,
    /// CRC error on received message
    InvalidCrc,
    /// Radio returned an invalid device firmware version
    InvalidDevice(u16),
    /// Radio returned an invalid response
    InvalidResponse(u8),
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
    pub fn spi(spi: Spi, cs: Output, busy: Input, sdn: Output, delay: Delay, settings: Settings, config: &Config) -> Result<Self, Error<CommsError, PinError>> {
        // Create SpiWrapper over spi/cs/busy
        let mut hal = SpiWrapper::new(spi, cs, delay);
        hal.with_busy(busy);
        hal.with_reset(sdn);
        // Create instance with new hal
        Self::new(hal, settings, config)
    }
}


impl<Hal, CommsError, PinError> Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    /// Create a new Sx128x instance over a generic Hal implementation
    pub fn new(hal: Hal, settings: Settings, config: &Config) -> Result<Self, Error<CommsError, PinError>> {

        let mut sx128x = Self::build(hal, settings);

        // Reset IC
        sx128x.hal.reset()?;

        // Check communication with the radio
        let firmware_version = sx128x.firmware_version()?;
        if firmware_version != 0xA9B5 {
            return Err(Error::InvalidDevice(firmware_version));
        }

        // TODO: do we need to calibrate things here?
        //sx128x.calibrate(CalibrationParams::default())?;

        // Configure device prior to use
        sx128x.configure(config, true)?;

        // Ensure state is idle
        sx128x.set_mode(Mode::StandbyRc)?;

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

    pub fn configure(&mut self, config: &Config, force: bool) -> Result<(), Error<CommsError, PinError>> {
        // Switch to standby mode
        self.set_mode(Mode::StandbyRc)?;

        // Update regulator mode
        if self.config.regulator_mode != config.regulator_mode || force {
            self.set_regulator_mode(config.regulator_mode)?;
            self.config.regulator_mode = config.regulator_mode;
        }

        // Update modulation and packet configuration
        if self.config.modulation_config != config.modulation_config || force {
            self.set_modulation_mode(&config.modulation_config)?;
            self.config.modulation_config = config.modulation_config.clone();
        }

        if self.config.packet_config != config.packet_config || force {
            self.set_packet_mode(&config.packet_config)?;
            self.config.packet_config = config.packet_config.clone();
        }

        // Set frequency
        self.set_frequency(config.frequency)?;

        // Update power amplifier configuration
        if self.config.pa_config != config.pa_config || force {
            self.set_power_ramp(config.pa_config.power, config.pa_config.ramp_time)?;
            self.config.pa_config = config.pa_config.clone();
        }

        Ok(())
    }

    pub fn get_mode(&mut self) -> Result<Mode, Error<CommsError, PinError>> {
        let mut d = [0u8; 1];
        self.hal.read_cmd(Commands::GetStatus as u8, &mut d)?;
        let m = Mode::try_from(d[0]).map_err(|_| Error::InvalidResponse(d[0]) )?;
        Ok(m)
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<(), Error<CommsError, PinError>> {
        let command = match mode {
            Mode::Tx => Commands::SetTx,
            Mode::Rx => Commands::SetRx,
            Mode::Cad => Commands::SetCad,
            Mode::Fs => Commands::SetFs,
            Mode::StandbyRc | Mode::StandbyXosc => Commands::SetStandby,
            Mode::Sleep => Commands::SetSleep,
        };

        self.hal.write_cmd(command as u8, &[ 0u8 ])
    }

    pub fn firmware_version(&mut self) -> Result<u16, Error<CommsError, PinError>> {
        let mut d = [0u8; 2];

        self.hal.read_regs(Registers::LrFirmwareVersionMsb as u16, &mut d)?;

        Ok((d[0] as u16) << 8 | (d[1] as u16))
    }

    pub fn set_frequency(&mut self, f: u32) -> Result<(), Error<CommsError, PinError>> {
        let c = self.settings.freq_to_steps(f as f32) as u32;

        debug!("Setting frequency ({:?} MHz, {} index)", f / 1000 / 1000, c);

        let data: [u8; 3] = [
            (c >> 16) as u8,
            (c >> 8) as u8,
            (c >> 0) as u8,
        ];

        self.hal.write_cmd(Commands::SetRfFrequency as u8, &data)
    }

    pub (crate) fn set_power_ramp(&mut self, power: i8, ramp: RampTime) -> Result<(), Error<CommsError, PinError>> {
        
        if power > 13 || power < -18 {
            warn!("TX power out of range (-18 < p < 13)");
        }

        // Limit to -18 to +13 dBm
        let power = core::cmp::min(power, -18);
        let power = core::cmp::max(power, 13);
        let power_reg = (power + 18) as u8;

        debug!("Setting TX power to {} dBm {:?} ramp ({}, {})", power, ramp, power_reg, ramp as u8);
        self.config.pa_config.power = power;
        self.config.pa_config.ramp_time = ramp;

        self.hal.write_cmd(Commands::SetTxParams as u8, &[ power_reg, ramp as u8 ])
    }

    pub fn set_irq_mask(&mut self, irq: Irq) -> Result<(), Error<CommsError, PinError>> {
        debug!("Setting IRQ mask {:?}", irq);
        let raw = irq.bits();
        self.hal.write_cmd(Commands::SetDioIrqParams as u8, &[ (raw >> 8) as u8, (raw & 0xff) as u8])
    }

    pub(crate) fn set_modulation_mode(&mut self, modulation: &ModulationMode) -> Result<(), Error<CommsError, PinError>> {
        use ModulationMode::*;

        debug!("Setting modulation config: {:?}", modulation);

        // First update packet type
        let packet_type = PacketType::from(modulation);
        if self.config.packet_type != packet_type {
            self.hal.write_cmd(Commands::SetPacketType as u8, &[ packet_type.clone() as u8 ] )?;
            self.config.packet_type = packet_type;
        }

        // Then write modulation configuration
        let data = match modulation {
            Gfsk(c) => [c.bitrate_bandwidth as u8, c.modulation_index as u8, c.modulation_shaping as u8],
            LoRa(c) | Ranging(c) => [c.spreading_factor as u8, c.bandwidth as u8, c.coding_rate as u8],
            Flrc(c) => [c.bitrate_bandwidth as u8, c.coding_rate as u8, c.modulation_shaping as u8],
            Ble(c) => [c.bitrate_bandwidth as u8, c.modulation_index as u8, c.modulation_shaping as u8],
        };

        self.hal.write_cmd(Commands::SetModulationParams as u8, &data)
    }

    pub(crate) fn set_packet_mode(&mut self, packet: &PacketMode) -> Result<(), Error<CommsError, PinError>> {
        use PacketMode::*;

        debug!("Setting packet config: {:?}", packet);

        // First update packet type
        let packet_type = PacketType::from(packet);
        if self.config.packet_type != packet_type {
            self.hal.write_cmd(Commands::SetPacketType as u8, &[ packet_type.clone() as u8 ] )?;
            self.config.packet_type = packet_type;
        }

        let data = match packet {
            Gfsk(c) => [c.preamble_length as u8, c.sync_word_length as u8, c.sync_word_match as u8, c.header_type as u8, c.payload_length as u8, c.crc_type as u8, c.whitening as u8],
            LoRa(c) | Ranging(c) => [c.preamble_length as u8, c.header_type as u8, c.payload_length as u8, c.crc_mode as u8, c.invert_iq as u8, 0u8, 0u8],
            Flrc(c) => [c.preamble_length as u8, c.sync_word_length as u8, c.sync_word_match as u8, c.header_type as u8, c.payload_length as u8, c.crc_type as u8, c.whitening as u8],
            Ble(c) => [c.connection_state as u8, c.crc_field as u8, c.packet_type as u8, c.whitening as u8, 0u8, 0u8, 0u8],
            None => [0u8; 7],
        };
        self.hal.write_cmd(Commands::SetPacketParams as u8, &data)
    }

    pub(crate) fn get_rx_buffer_status(&mut self) -> Result<(u8, u8), Error<CommsError, PinError>> {
        use device::lora::LoRaHeader;

        let mut status = [0u8; 2];

        self.hal.read_cmd(Commands::GetRxBufferStatus as u8, &mut status)?;

        let len = match &self.config.packet_config {
            PacketMode::LoRa(c) => {
                match c.header_type {
                    LoRaHeader::Implicit => self.hal.read_reg(Registers::LrPayloadLength as u8)?,
                    LoRaHeader::Explicit => status[0],
                }
            },
            // BLE status[0] does not include 2-byte PDU header
            PacketMode::Ble(_) => status[0] + 2,
            _ => status[0]
        };

        let rx_buff_ptr = status[1];

        debug!("RX buffer ptr: {} len: {}", rx_buff_ptr, len);

        Ok((rx_buff_ptr, len))
    }

    
    pub(crate) fn get_packet_info(&mut self, info: &mut Info) -> Result<(), Error<CommsError, PinError>> {

        let mut data = [0u8; 5];
        self.hal.read_cmd(Commands::GetPacketStatus as u8, &mut data)?;

        info.packet_status = PacketStatus::from_bits_truncate(data[2]);
        info.tx_rx_status = TxRxStatus::from_bits_truncate(data[3]);
        info.sync_addr_status = data[4] & 0x70;

        match self.config.packet_type {
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

        debug!("RX packet info {:?}", info);

        Ok(())
    }

    pub fn calibrate(&mut self, c: CalibrationParams) -> Result<(), Error<CommsError, PinError>> {
        debug!("Calibrate {:?}", c);
        self.hal.write_cmd(Commands::Calibrate as u8, &[ c.bits() ])
    }

    pub(crate) fn set_regulator_mode(&mut self, r: RegulatorMode) -> Result<(), Error<CommsError, PinError>> {
        debug!("Set regulator mode {:?}", r);
        self.hal.write_cmd(Commands::SetRegulatorMode as u8, &[ r as u8 ])
    }

    // TODO: this could got into a mode config object maybe?
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

    pub(crate) fn set_buff_base_addr(&mut self, tx: u8, rx: u8) -> Result<(), Error<CommsError, PinError>> {
        debug!("Set buff base address (tx: {}, rx: {})", tx, rx);
        self.hal.write_cmd(Commands::SetBufferBaseAddress as u8, &[ tx, rx ])
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

impl Default for Info {
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

/// `radio::Channel` implementation for the SX128x
impl<Hal, CommsError, PinError> Channel for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    type Channel = ();
    type Error = Error<CommsError, PinError>;

    /// Set operating channel
    fn set_channel(&mut self, _ch: &Self::Channel) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

/// `radio::Power` implementation for the SX128x
impl<Hal, CommsError, PinError> Power for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    type Error = Error<CommsError, PinError>;

    /// Set TX power in dBm
    fn set_power(&mut self, power: i8) -> Result<(), Error<CommsError, PinError>> {
        let ramp_time = self.config.pa_config.ramp_time;
        self.set_power_ramp(power, ramp_time)
    }
}

/// `radio::Interrupts` implementation for the SX128x
impl<Hal, CommsError, PinError> Interrupts for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    type Irq = Irq;
    type Error = Error<CommsError, PinError>;

    /// Fetch (and optionally clear) current interrupts
    fn get_interrupts(&mut self, clear: bool) -> Result<Self::Irq, Self::Error> {
        let mut data = [0u8; 2];

        self.hal.read_cmd(Commands::GetIrqStatus as u8, &mut data)?;
        let irq = Irq::from_bits((data[0] as u16) << 8 | data[1] as u16).unwrap();

        if clear && !irq.is_empty() {
            self.hal.write_cmd(Commands::ClearIrqStatus as u8, &data)?;
        }

        Ok(irq)
    }
}

/// `radio::Transmit` implementation for the SX128x
impl<Hal, CommsError, PinError> Transmit for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    type Error = Error<CommsError, PinError>;

    /// Start transmitting a packet
    fn start_transmit(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        info!("TX start");

        // Set packet mode
        let mut config = self.config.packet_config.clone();
        config.set_payload_len(data.len() as u8);
        self.set_packet_mode(&config)?;

        // Reset buffer addr
        self.set_buff_base_addr(0, 0)?;
        
        // Write data to be sent
        self.hal.write_buff(0, data)?;
        
        // Configure ranging if used
        if PacketType::Ranging == self.config.packet_type {
            self.hal.write_cmd(Commands::SetRangingRole as u8, &[ RangingRole::Initiator as u8 ])?;
        }

        // Setup timout
        let config = [
            self.config.timeout.step() as u8,
            (( self.config.timeout.count() >> 8 ) & 0x00FF ) as u8,
            (self.config.timeout.count() & 0x00FF ) as u8,
        ];
        
        // Enable IRQs
        self.set_irq_mask(Irq::TX_DONE | Irq::CRC_ERROR | Irq::RX_TX_TIMEOUT)?;

        // Enter transmit mode
        self.hal.write_cmd(Commands::SetTx as u8, &config)?;

        Ok(())
    }

    /// Check for transmit completion
    fn check_transmit(&mut self) -> Result<bool, Self::Error> {
        let irq = self.get_interrupts(true)?;

        if irq.contains(Irq::TX_DONE) {
            debug!("TX complete");
            Ok(true)
        } else if irq.contains(Irq::RX_TX_TIMEOUT) {
            debug!("TX timeout");
            Err(Error::Timeout)
        } else {
            trace!("TX poll (irq: {:?}", irq);
            Ok(false)
        }
    }
}

/// `radio::Receive` implementation for the SX128x
impl<Hal, CommsError, PinError> Receive for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    /// Receive info structure
    type Info = Info;

    /// RF Error object
    type Error = Error<CommsError, PinError>;

    /// Start radio in receive mode
    fn start_receive(&mut self) -> Result<(), Self::Error> {
        debug!("RX start");

        // Reset buffer addr
        self.set_buff_base_addr(0, 0)?;
        
        // Set packet mode
        let config = self.config.packet_config.clone();
        self.set_packet_mode(&config)?;

        // Configure ranging if used
        if PacketType::Ranging == self.config.packet_type {
            self.hal.write_cmd(Commands::SetRangingRole as u8, &[ RangingRole::Responder as u8 ])?;
        }

        // Setup timout
        let config = [
            self.config.timeout.step() as u8,
            (( self.config.timeout.count() >> 8 ) & 0x00FF ) as u8,
            (self.config.timeout.count() & 0x00FF ) as u8,
        ];
        
        // Enable IRQs
        self.set_irq_mask(Irq::RX_DONE | Irq::CRC_ERROR | Irq::RX_TX_TIMEOUT)?;

        // Enter transmit mode
        self.hal.write_cmd(Commands::SetRx as u8, &config)?;

        Ok(())
    }

    /// Check for a received packet
    fn check_receive(&mut self, restart: bool) -> Result<bool, Self::Error> {
        let irq = self.get_interrupts(true)?;
        let mut res = Ok(false);
       
        // Process flags
        if irq.contains(Irq::RX_DONE) {
            debug!("RX complete");
            res = Ok(true);
        } else if irq.contains(Irq::CRC_ERROR) {
            debug!("RX CRC error");
            res = Err(Error::InvalidCrc);
        } else if irq.contains(Irq::RX_TX_TIMEOUT) {
            debug!("RX timeout");
            res = Err(Error::Timeout);
        } else   {
            trace!("RX poll (irq: {:?})", irq);
        }

        match (restart, res) {
            (true, Err(_)) => {
                debug!("RX restarting");
                self.start_receive()?;
                Ok(false)
            },
            (_, r) => r
        }
    }

    /// Fetch a received packet
    fn get_received<'a>(&mut self, info: &mut Self::Info, data: &'a mut [u8]) -> Result<usize, Self::Error> {
        // Fetch RX buffer information
        let (ptr, len) = self.get_rx_buffer_status()?;

        debug!("RX get received, ptr: {} len: {}", ptr, len);

        // Read from the buffer at the provided pointer
        self.hal.read_buff(ptr, &mut data[..len as usize])?;

        // Fetch related information
        self.get_packet_info(info)?;

        debug!("RX info: {:?}", info);

        // Return read length
        Ok(len as usize)
    }

}

/// `radio::Rssi` implementation for the SX128x
impl<Hal, CommsError, PinError> Rssi for Sx128x<Hal, CommsError, PinError>
where
    Hal: base::Hal<CommsError, PinError>,
{
    type Error = Error<CommsError, PinError>;

    /// Poll for the current channel RSSI
    /// This should only be called when in receive mode
    fn poll_rssi(&mut self) -> Result<i16, Error<CommsError, PinError>> {
        let mut raw = [0u8; 1];
        self.hal.read_cmd(Commands::GetRssiInst as u8, &mut raw)?;
        Ok(-(raw[0] as i16) / 2)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Sx128x, Settings};
    use crate::base::Hal;
    use crate::device::RampTime;

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
        radio.get_mode().unwrap();
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

    #[test]
    fn test_api_power_ramp() {
        let mut m = Mock::new();
        let (spi, sdn, busy, delay) = (m.spi(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::<Spi, _, _>::build(spi.clone(), Settings::default());

        m.expect(vectors::set_power_ramp(&spi, &sdn, &delay, 0x1f, 0xe0));
        let version = radio.set_power_ramp(13, RampTime::Ramp20Us).unwrap();
        m.finalise();
    }
}
