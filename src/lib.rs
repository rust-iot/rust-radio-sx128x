//! Sx128x Radio Driver
//! Copyright 2018 Ryan Kurte

#![no_std]
extern crate libc;
extern crate embedded_hal as hal;
extern crate futures;
extern crate nb;

use hal::blocking::{spi, delay};
use hal::digital::v2::{InputPin, OutputPin};
use hal::spi::{Mode, Phase, Polarity};


pub mod sx1280;
use sx1280::{SX1280_s};

pub mod compat;

/// Sx128x Spi operating mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// Sx128x device object
#[repr(C)]
pub struct Sx128x<Spi, Output, Input, Delay> {
    spi: Spi,
    sdn: Output,
    cs: Output,
    busy: Input,
    delay: Delay,
    c: Option<SX1280_s>,
}

pub struct Settings {

}

impl Default for Settings {
    fn default() -> Self {
        Self{}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sx128xError<SpiError, PinError> {
    Spi(SpiError),
    Pin(PinError),
}

impl<SpiError, Spi, PinError, Output, Input, Delay> Sx128x<Spi, Output, Input, Delay>
where
    Spi: spi::Transfer<u8, Error = SpiError> + spi::Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{
    pub fn new(spi: Spi, sdn: Output, cs: Output, busy: Input, delay: Delay, _settings: Settings) -> Result<Self, Sx128xError<SpiError, PinError>> {

        let mut sx128x = Sx128x { spi, sdn, cs, busy, delay, c: None };

        // Reset IC
        sx128x.reset()?;

        // Calibrate RX chain
        //sx1280::RxChainCalibration(&sx128x.c);

        // Init IRQs (..?)

        // Confiure modem(s)

        // Set state to idle


        Ok(sx128x)
    }

    /// Reset the radio device
    pub fn reset(&mut self) -> Result<(), Sx128xError<SpiError, PinError>> {
        self.sdn.set_low().map_err(|e| Sx128xError::Pin(e) )?;
        self.delay.delay_ms(1);
        self.sdn.set_high().map_err(|e| Sx128xError::Pin(e) )?;
        self.delay.delay_ms(10);

        Ok(())
    }

    /// Read data from a specified register address
    /// This consumes the provided input data array and returns a reference to this on success
    fn read<'a>(&mut self, command: &[u8], mut data: &'a mut [u8]) -> Result<&'a [u8], Sx128xError<SpiError, PinError>> {
        // TODO: wait on busy

        // Assert CS
        self.cs.set_low().map_err(|e| Sx128xError::Pin(e) )?;

        // Write command
        let mut res = self.spi.write(&command);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.transfer(&mut data).map(|_r| () );
        }

        // Clear CS
        self.cs.set_high().map_err(|e| Sx128xError::Pin(e) )?;

        // TODO: wait on busy

        // Return result (contains returned data)
        match res {
            Err(e) => Err(Sx128xError::Spi(e)),
            Ok(_) => Ok(data),
        }
    }

    /// Write data to a specified register address
    pub fn write(&mut self, command: &[u8], data: &[u8]) -> Result<(), Sx128xError<SpiError, PinError>> {
        // TODO: wait on busy

        // Assert CS
        self.cs.set_low().map_err(|e| Sx128xError::Pin(e) )?;

        // Write command
        let mut res = self.spi.write(&command);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.write(&data);
        }

        // Clear CS
        self.cs.set_high().map_err(|e| Sx128xError::Pin(e) )?;

        // TODO: wait on busy

        match res {
            Err(e) => Err(Sx128xError::Spi(e)),
            Ok(_) => Ok(()),
        }
    }
    
    pub fn cmd_write(&mut self, command: u8, data: &[u8]) -> Result<(), Sx128xError<SpiError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 1] = [command as u8];
        self.write(&out_buf, data)
    }
    pub fn cmd_read<'a>(&mut self, command: u8, mut data: &'a mut [u8]) -> Result<&'a [u8], Sx128xError<SpiError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 2] = [command as u8, 0x00];
        self.read(&out_buf, data)
    }


    pub fn reg_write(&mut self, reg: u16, data: &[u8]) -> Result<(), Sx128xError<SpiError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 3] = [
            sx1280::RadioCommands_u_RADIO_WRITE_REGISTER as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
        ];
        self.write(&out_buf, data)
    }
    pub fn reg_read<'a>(&mut self, reg: u16, mut data: &'a mut [u8]) -> Result<&'a [u8], Sx128xError<SpiError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 4] = [
            sx1280::RadioCommands_u_RADIO_READ_REGISTER as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
            0,
        ];
        self.read(&out_buf, data)
    }

     pub fn buff_write(&mut self, offset: u8, data: &[u8]) -> Result<(), Sx128xError<SpiError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 2] = [
            sx1280::RadioCommands_u_RADIO_WRITE_BUFFER as u8,
            offset,
        ];
        self.write(&out_buf, data)
    }
    pub fn buff_read<'a>(&mut self, offset: u8, mut data: &'a mut [u8]) -> Result<&'a [u8], Sx128xError<SpiError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 3] = [
            sx1280::RadioCommands_u_RADIO_READ_BUFFER as u8,
            offset,
            0
        ];
        self.read(&out_buf, data)
    }
    
}


#[cfg(test)]
mod tests {
    use super::*;

    extern crate std;
    use tests::std::boxed::Box;
    use tests::std::vec::*;



    extern crate embedded_hal_mock;
    use tests::embedded_hal_mock::engine::*;
    use tests::embedded_hal_mock::MockError;
    use tests::embedded_hal_mock::spi::Mock as SpiMock;

    extern crate embedded_hal;
    use tests::embedded_hal::blocking::spi::{Transfer, Write};


    trait Test<E, F>: spi::Transfer<u8, Error=E> + spi::Write<u8, Error=F> {}

    #[test]
    fn mock_test() {
        let mut engine = Engine::new();

        let mut spi = engine.spi();
        let mut sdn = engine.pin();
        let mut cs = engine.pin();
        let mut busy = engine.pin();
        let mut delay = engine.delay();

        //let s: Box<Test<_, _>> = Box::new(spi.clone());

        let mut radio = Sx128x{spi: spi.clone(), sdn: sdn.clone(), cs: cs.clone(), busy: busy.clone(), delay: delay.clone(), c: None};

        radio.bind();

        engine.done();

        sdn.expect(PinTransaction::set(PinState::Low));
        delay.expect(1);
        sdn.expect(PinTransaction::set(PinState::High));
        delay.expect(10);

        radio.reset();

        engine.done();

        //spi.inner().expect(SpiTransaction::write(vec![sx1280::RadioCommands_u_RADIO_GET_STATUS as u8]));

        radio.status();

        engine.done();
    }
}
