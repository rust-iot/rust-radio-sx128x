//! Sx128x Radio Driver
//! Copyright 2018 Ryan Kurte

#![no_std]
extern crate embedded_hal as hal;
extern crate futures;
extern crate libc;
extern crate nb;

use core::{mem, ptr, slice};

use hal::blocking::{spi, delay};
use hal::digital::v2::{InputPin, OutputPin};
use hal::spi::{Mode, Phase, Polarity};


pub mod sx1280;
use sx1280::{SX1280_s};


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
    c: SX1280_s,
}

pub struct Settings {

}

impl Default for Settings {
    fn default() -> Self {
        Self{}
    }
}

extern fn DelayMs(ms: u32) {

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
    pub fn new(spi: Spi, sdn: Output, cs: Output, busy: Input, delay: Delay, settings: Settings) -> Result<Self, Sx128xError<SpiError, PinError>> {

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

    fn build(spi: Spi, sdn: Output, cs: Output, busy: Input, delay: Delay, settings: Settings) -> Self {
        unsafe {
            let mut c = SX1280_s{
                ctx: mem::uninitialized(),
                reset: Some(Self::ext_reset),
                write_buffer: Some(Self::write_buffer),
                read_buffer: Some(Self::read_buffer),
                delay_ms: Some(Self::delay_ms),
            };

            let mut sx128x = Sx128x { spi, sdn, cs, busy, delay, c: c };
            
            sx128x.c.ctx = &mut sx128x as *mut Sx128x<Spi, Output, Input, Delay> as *mut libc::c_void;

            sx128x
        }
    }

    // extern functions used by c hal
    // todo: errors are not bubbled through these functions

    extern fn ext_reset(ctx: *mut libc::c_void) {
        unsafe {
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x = (*sx1280).ctx as *mut Sx128x<Spi, Output, Input, Delay>;
            let _ = (*sx128x).reset();
        }
    }

    extern fn write_buffer(ctx: *mut libc::c_void, addr: u8, buffer: *mut u8, size: u8) {
        unsafe {
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x = (*sx1280).ctx as *mut Sx128x<Spi, Output, Input, Delay>;
            let data: &[u8] = slice::from_raw_parts(buffer, size as usize);
            let _ = (*sx128x).reg_write(addr, data);
        }
    }
    
    extern fn read_buffer(ctx: *mut libc::c_void, addr: u8, buffer: *mut u8, size: u8) {
        unsafe {
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x = (*sx1280).ctx as *mut Sx128x<Spi, Output, Input, Delay>;
            let data: &mut [u8] = slice::from_raw_parts_mut(buffer, size as usize);
            let _ = (*sx128x).reg_read(addr, data);
        }
    }

    extern fn delay_ms(ctx: *mut libc::c_void, ms: u32) {
        unsafe {
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x = (*sx1280).ctx as *mut Sx128x<Spi, Output, Input, Delay>;
            let _ = (*sx128x).delay.delay_ms(ms as u32);
        }
    }

    fn from_c<'a>(sx1280: * mut SX1280_s) -> *mut Self {
        unsafe {
            let sx128x_ptr = (*sx1280).ctx as *mut libc::c_void;
            let sx128x = sx128x_ptr as *mut Sx128x<Spi, Output, Input, Delay>;
            sx128x
        }
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
    fn reg_read<'a>(&mut self, reg: u8, mut data: &'a mut [u8]) -> Result<&'a [u8], Sx128xError<SpiError, PinError>> {
        // Setup read command
        let out_buf: [u8; 1] = [reg as u8 & 0x7F];
        // Assert CS
        self.cs.set_low().map_err(|e| Sx128xError::Pin(e) )?;

        // Write command
        let mut res = self.spi.write(&out_buf);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.transfer(&mut data).map(|_r| () );
        }

        // Clear CS
        self.cs.set_high().map_err(|e| Sx128xError::Pin(e) )?;

        // Return result (contains returned data)
        match res {
            Err(e) => Err(Sx128xError::Spi(e)),
            Ok(_) => Ok(data),
        }
    }

    /// Write data to a specified register address
    pub fn reg_write(&mut self, reg: u8, data: &[u8]) -> Result<(), Sx128xError<SpiError, PinError>> {
        // Setup write command
        let out_buf: [u8; 1] = [reg as u8 | 0x80];
        // Assert CS
        self.cs.set_low().map_err(|e| Sx128xError::Pin(e) )?;

        // Write command
        let mut res = self.spi.write(&out_buf);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.write(&data);
        }

        // Clear CS
        self.cs.set_high().map_err(|e| Sx128xError::Pin(e) )?;;

        match res {
            Err(e) => Err(Sx128xError::Spi(e)),
            Ok(_) => Ok(()),
        }
    }

    pub fn status(&mut self) -> Result<sx1280::RadioStatus_t, Sx128xError<SpiError, PinError>> {
        let status = unsafe { sx1280::SX1280GetStatus(&mut self.c) };
        Ok(status)
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

        let mut radio = Sx128x::build(spi.clone(), sdn.clone(), cs.clone(), busy.clone(), delay.clone(), Settings::default());

        engine.done();

        sdn.expect(PinTransaction::set(PinState::Low));
        delay.expect(1);
        sdn.expect(PinTransaction::set(PinState::High));
        delay.expect(10);

        radio.reset();

        engine.done();

        spi.inner().expect(SpiTransaction::write(vec![sx1280::RadioCommands_u_RADIO_GET_STATUS as u8]));

        radio.status();

        engine.done();
    }
}
