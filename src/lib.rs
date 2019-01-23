//! Sx128x Radio Driver
//! Copyright 2018 Ryan Kurte

#![no_std]
extern crate embedded_hal as hal;
extern crate futures;
extern crate libc;
extern crate nb;

use core::{mem, ptr, slice};

use hal::blocking::{spi, delay};
use hal::digital::{InputPin, OutputPin};
use hal::spi::{Mode, Phase, Polarity};


pub mod sx1280;
use sx1280::{SX1280_s};


/// Sx128x SPI operating mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// Sx128x device object
#[repr(C)]
pub struct Sx128x<SPI, OUTPUT, INPUT, DELAY> {
    spi: SPI,
    sdn: OUTPUT,
    cs: OUTPUT,
    gpio: [Option<INPUT>; 4],
    delay: DELAY,
    c: SX1280_s,
}

pub struct Settings {

}

extern fn DelayMs(ms: u32) {

}

pub enum Sx127xError<SPIError> {
    SPI(SPIError)
}

impl <SPIError>From<SPIError> for Sx127xError<SPIError> {
	fn from(e: SPIError) -> Sx127xError<SPIError> {
		Sx127xError::SPI(e)
	}
}


impl<E, SPI, OUTPUT, INPUT, DELAY> Sx128x<SPI, OUTPUT, INPUT, DELAY>
where
    SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
    OUTPUT: OutputPin,
    INPUT: InputPin,
    DELAY: delay::DelayMs<usize>,
{
    pub fn new(spi: SPI, sdn: OUTPUT, cs: OUTPUT, gpio: [Option<INPUT>; 4], delay: DELAY, settings: Settings) -> Result<Self, Sx127xError<E>> {
        unsafe {
            let mut c = SX1280_s{
                ctx: mem::uninitialized(),
                reset: Some(Self::ext_reset),
                write_buffer: Some(Self::write_buffer),
                read_buffer: Some(Self::read_buffer),
                delay_ms: Some(Self::delay_ms),
            };

            let mut sx128x = Sx128x { spi, sdn, cs, gpio, delay, c: c };
            
            sx128x.c.ctx = &mut sx128x as *mut Sx128x<SPI, OUTPUT, INPUT, DELAY> as *mut libc::c_void;

            // Reset IC
            sx128x.reset();

            // Calibrate RX chain
            //sx1280::RxChainCalibration(&sx128x.c);

            // Init IRQs (..?)

            // Confiure modem(s)

            // Set state to idle


            Ok(sx128x)
        }
    }

     extern fn ext_reset(ctx: *mut libc::c_void) {
        unsafe {
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x = (*sx1280).ctx as *mut Sx128x<SPI, OUTPUT, INPUT, DELAY>;
            (*sx128x).reset();
        }
    }

    extern fn write_buffer(ctx: *mut libc::c_void, addr: u8, buffer: *mut u8, size: u8) {
        unsafe {
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x = (*sx1280).ctx as *mut Sx128x<SPI, OUTPUT, INPUT, DELAY>;
            let data: &[u8] = slice::from_raw_parts(buffer, size as usize);
            (*sx128x).reg_write(addr, data);
        }
    }
    
    extern fn read_buffer(ctx: *mut libc::c_void, addr: u8, buffer: *mut u8, size: u8) {
        unsafe {
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x = (*sx1280).ctx as *mut Sx128x<SPI, OUTPUT, INPUT, DELAY>;
            let data: &mut [u8] = slice::from_raw_parts_mut(buffer, size as usize);
            (*sx128x).reg_read(addr, data);
        }
    }

    extern fn delay_ms(ctx: *mut libc::c_void, ms: u32) {
        unsafe {
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x = (*sx1280).ctx as *mut Sx128x<SPI, OUTPUT, INPUT, DELAY>;
            (*sx128x).delay.delay_ms(ms as usize);
        }
    }

    fn from_c<'a>(sx1280: * mut SX1280_s) -> *mut Self {
        unsafe {
            let sx128x_ptr = (*sx1280).ctx as *mut libc::c_void;
            let sx128x = sx128x_ptr as *mut Sx128x<SPI, OUTPUT, INPUT, DELAY>;
            sx128x
        }
    }

    pub fn reset(&mut self) {
        self.sdn.set_low();
        self.delay.delay_ms(1);
        self.sdn.set_high();
        self.delay.delay_ms(10);
    }

    /// Read data from a specified register address
    /// This consumes the provided input data array and returns a reference to this on success
    fn reg_read<'a>(&mut self, reg: u8, data: &'a mut [u8]) -> Result<&'a [u8], Sx127xError<E>> {
        // Setup read command
        let out_buf: [u8; 1] = [reg as u8 & 0x7F];
        // Assert CS
        self.cs.set_low();
        // Write command
        match self.spi.write(&out_buf) {
            Ok(_r) => (),
            Err(e) => {
                self.cs.set_high();
                return Err(Sx127xError::SPI(e));
            }
        };
        // Transfer data
        let res = match self.spi.transfer(data) {
            Ok(r) => r,
            Err(e) => {
                self.cs.set_high();
                return Err(Sx127xError::SPI(e));
            }
        };
        // Clear CS
        self.cs.set_high();
        // Return result (contains returned data)
        Ok(res)
    }

    /// Write data to a specified register address
    pub fn reg_write(&mut self, reg: u8, data: &[u8]) -> Result<(), Sx127xError<E>> {
        // Setup write command
        let out_buf: [u8; 1] = [reg as u8 | 0x80];
        // Assert CS
        self.cs.set_low();
        // Write command
        match self.spi.write(&out_buf) {
            Ok(_r) => (),
            Err(e) => {
                self.cs.set_high();
                return Err(Sx127xError::SPI(e));
            }
        };
        // Transfer data
        match self.spi.write(&data) {
            Ok(_r) => (),
            Err(e) => {
                self.cs.set_high();
                return Err(Sx127xError::SPI(e));
            }
        };
        // Clear CS
        self.cs.set_high();

        Ok(())
    }

}

#[cfg(test)]
mod tests {

    extern crate embedded_hal_mock;
    use tests::embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};

    #[test]
    fn mock_test() {
        let expectations = [
            SpiTransaction::send(0x09),
        ];

        let mut spi = SpiMock::new(&expectations);


        assert_eq!(2 + 2, 4);
    }
}
