//! Sx128x Radio Driver
//! Copyright 2018 Ryan Kurte

#![no_std]
extern crate libc;

extern crate futures;
extern crate nb;

#[cfg(test)]
#[macro_use]
extern crate std;

extern crate embedded_hal as hal;
use hal::blocking::{spi, delay};
use hal::digital::v2::{InputPin, OutputPin};
use hal::spi::{Mode, Phase, Polarity};


extern crate embedded_spi;
use embedded_spi::{Error as WrapError};
use embedded_spi::wrapper::{Wrapper as SpiWrapper};

pub mod bindings;
use bindings::{self as sx1280};

#[cfg(feature = "ffi")]
use bindings::SX1280_s;

pub mod base;
use base::Comms;

#[cfg(feature = "ffi")]
pub mod ffi;

/// Sx128x Spi operating mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// Sx128x device object
#[repr(C)]
pub struct Sx128x<Spi, CommsError, Output, Input, PinError, Delay> {
    spi: SpiWrapper<Spi, CommsError, Output, PinError>,

    sdn: Output,
    busy: Input,
    delay: Delay,

    #[cfg(feature = "ffi")]
    c: Option<SX1280_s>,
    #[cfg(feature = "ffi")]
    err: Option<Sx128xError<CommsError, PinError>>,
}

pub struct Settings {

}

impl Default for Settings {
    fn default() -> Self {
        Self{}
    }
}

/// Sx128x error type
#[derive(Debug, Clone, PartialEq)]
pub enum Sx128xError<CommsError, PinError> {
    /// Communications (SPI or UART) error
    Comms(CommsError),
    /// Pin control error
    Pin(PinError),
    /// Transaction aborted
    Aborted,
}

impl <CommsError, PinError> From<WrapError<CommsError, PinError>> for Sx128xError<CommsError, PinError> {
    fn from(e: WrapError<CommsError, PinError>) -> Self {
        match e {
            WrapError::Spi(e) => Sx128xError::Comms(e),
            WrapError::Pin(e) => Sx128xError::Pin(e),
            WrapError::Aborted => Sx128xError::Aborted,
        }
    }
}


impl<Spi, CommsError, Output, Input, PinError, Delay> Sx128x<Spi, CommsError, Output, Input, PinError, Delay>
where
    Spi: spi::Transfer<u8, Error = CommsError> + spi::Write<u8, Error = CommsError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{
    pub fn new(spi: Spi, cs: Output, sdn: Output, busy: Input, delay: Delay, settings: Settings) -> Result<Self, Sx128xError<CommsError, PinError>> {

        let mut sx128x = Self::build(spi, sdn, cs, busy, delay, settings);

        // Reset IC
        sx128x.reset()?;

        // Calibrate RX chain
        //sx1280::RxChainCalibration(&sx128x.c);

        // Init IRQs (..?)

        // Confiure modem(s)

        // Set state to idle


        Ok(sx128x)
    }

    pub(crate) fn build(spi: Spi, cs: Output, sdn: Output, busy: Input, delay: Delay, _settings: Settings) -> Self {
        Sx128x { 
            spi: SpiWrapper::new(spi, cs),
            sdn, busy, delay, 
            #[cfg(feature = "ffi")]
            c: None, 
            #[cfg(feature = "ffi")]
            err: None,
        }
    }

    pub fn status(&mut self) -> Result<u8, Sx128xError<CommsError, PinError>> {
        let mut d = [0u8; 1];
        self.cmd_read(sx1280::RadioCommands_u_RADIO_GET_STATUS as u8, &mut d)?;
        Ok(d[0])
    }

    pub fn firmware_version(&mut self) -> Result<u16, Sx128xError<CommsError, PinError>> {
        let mut d = [0u8; 2];

        self.reg_read(sx1280::REG_LR_FIRMWARE_VERSION_MSB as u16, &mut d)?;

        Ok((d[0] as u16) << 8 | (d[1] as u16))
    }
}


#[cfg(test)]
mod tests {
    use crate::{Sx128x, Settings};
    use crate::base::Comms;

    extern crate embedded_spi;
    use self::embedded_spi::mock::{Mock};

    pub mod vectors;

    #[test]
    fn test_api_reset() {
        let mut m = Mock::new();
        let (spi, cs, sdn, busy, delay) = (m.spi(), m.pin(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::build(spi.clone(), cs.clone(), sdn.clone(), busy.clone(), delay.clone(), Settings::default());

        m.expect(vectors::reset(&spi, &cs, &sdn, &busy, &delay));
        radio.reset().unwrap();
        m.finalise();
    }

    #[test]
    fn test_api_status() {
        let mut m = Mock::new();
        let (spi, cs, sdn, busy, delay) = (m.spi(), m.pin(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::build(spi.clone(), cs.clone(), sdn.clone(), busy.clone(), delay.clone(), Settings::default());

        m.expect(vectors::status(&spi, &cs, &sdn, &busy, &delay));
        radio.status().unwrap();
        m.finalise();
    }

    #[test]
    fn test_api_firmware_version() {
        let mut m = Mock::new();
        let (spi, cs, sdn, busy, delay) = (m.spi(), m.pin(), m.pin(), m.pin(), m.delay());
        let mut radio = Sx128x::build(spi.clone(), cs.clone(), sdn.clone(), busy.clone(), delay.clone(), Settings::default());

        m.expect(vectors::firmware_version(&spi, &cs, &sdn, &busy, &delay, 16));
        let version = radio.firmware_version().unwrap();
        m.finalise();
        assert_eq!(version, 16);
    }
}
