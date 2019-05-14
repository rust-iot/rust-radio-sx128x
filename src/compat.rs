
extern crate std;

use core::{mem, ptr, slice};

use embedded_spi::compat::{Cursed, Conv};

use hal::blocking::{spi, delay};
use hal::digital::v2::{InputPin, OutputPin};

use crate::{Sx128x, Sx128xError, Settings};
use crate::bindings::{self as sx1280, SX1280_s};

impl<Spi, SpiError, Output, Input, PinError, Delay> Sx128x<Spi, SpiError, Output, Input, PinError, Delay>
where
    Spi: spi::Transfer<u8, Error = SpiError> + spi::Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
    SpiError: core::fmt::Debug,
    PinError: core::fmt::Debug,
{
    /// Build a rust object containing a c object containing a pointer to the rust object
    pub(crate) fn bind(&mut self) {
        let ctx = self.to_c();

        std::println!("Binding: {:?}", ctx);

        // Create base C object
        let c = SX1280_s {
            ctx,

            set_reset: Some(Self::set_reset),
            get_busy: Some(Self::get_busy),
            
            spi_write: Some(Self::spi_write),
            spi_read: Some(Self::spi_read),

            get_dio: [None; 4],

            delay_ms: Some(Self::delay_ms),
        };

        // Store C object in structure
        self.c = Some(c);
    }

    pub(crate) fn to_c(&mut self) -> *mut libc::c_void {
        let ptr = unsafe {
            self as *mut Self as *mut libc::c_void
        };
        ptr
    }

    pub(crate) fn from_c<'a>(ctx: *mut libc::c_void) -> &'a mut Self {
        unsafe {
            //std::println!("Retrieving: {:?}", ctx);
            //assert!(ctx == ptr::null());
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x_ptr = (*sx1280).ctx as *mut libc::c_void;
            let sx128x = sx128x_ptr as *mut Self;
            &mut *sx128x
        }
    }

    // extern functions used by c hal
    // todo: errors are not bubbled through these functions

    fn from_existing<'a>(&self, ctx: *mut libc::c_void) -> &'a mut Self {
        Self::from_c(ctx)
    }

    extern fn set_reset(ctx: *mut libc::c_void, value: bool) -> i32 {
        let sx128x = Self::from_c(ctx);
        let r = match value {
            true => sx128x.sdn.set_high(),
            false => sx128x.sdn.set_low(),
        };
        match r {
            Ok(_) => 0,
            Err(e) => {
                sx128x.err = Some(Sx128xError::Pin(e));
                -1
            }
        }
    }

    extern fn get_busy(ctx: *mut libc::c_void) -> i32 {
        let sx128x = Self::from_c(ctx);
        let r = sx128x.busy.is_high();
        match r {
            Ok(true) => 1,
            Ok(false) => 0,
            Err(e) => {
                sx128x.err = Some(Sx128xError::Pin(e));
                -1
            }
        }
    }

    extern fn spi_write(ctx: *mut libc::c_void, prefix: *mut u8, prefix_len: u16, data: *mut u8, data_len: u16) -> i32 {
        // Coerce back into rust
        let s = Self::from_c_ptr(ctx);

        // Parse buffers
        let prefix: &[u8] = unsafe { core::slice::from_raw_parts(prefix, prefix_len as usize) };
        let data: &[u8] = unsafe { core::slice::from_raw_parts(data, data_len as usize) };

        // Execute command and handle errors
        match s.write(&prefix, &data) {
            Ok(_) => 0,
            Err(e) => {
                s.err = Some(e);
                -1
            },
        }
    }
    
    extern fn spi_read(ctx: *mut libc::c_void, prefix: *mut u8, prefix_len: u16, data: *mut u8, data_len: u16) -> i32 {
         // Coerce back into rust
        let s = Self::from_c_ptr(ctx);

        // Parse buffers
        let prefix: &[u8] = unsafe { core::slice::from_raw_parts(prefix, prefix_len as usize) };
        let mut data: &mut [u8] = unsafe { core::slice::from_raw_parts_mut(data, data_len as usize) };

        // Execute command and handle errors
        match s.read(&prefix, &mut data) {
            Ok(_) => 0,
            Err(e) => {
                s.err = Some(e);
                -1
            },
        }
    }

    extern fn delay_ms(ctx: *mut libc::c_void, ms: u32) {
        let sx128x = Self::from_c(ctx);;
        let _ = sx128x.delay.delay_ms(ms as u32);
    }

}

impl<Spi, SpiError, Output, Input, PinError, Delay> Sx128x<Spi, SpiError, Output, Input, PinError, Delay>
where
    Spi: spi::Transfer<u8, Error = SpiError> + spi::Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{


    pub fn status(&mut self) -> Result<sx1280::RadioStatus_t, Sx128xError<SpiError, PinError>> {
        // Update rust object pointer to c object context
        let mut ctx = self.c.unwrap();

        let status = unsafe { sx1280::SX1280GetStatus(&mut ctx) };
        Ok(status)
    }

}


#[cfg(test)]
mod tests {
    use crate::Sx128x;

    extern crate std;

    extern crate embedded_spi;
    use self::embedded_spi::mock::Mock;

    extern crate color_backtrace;

    #[test]
    fn ffi_casts() {
        color_backtrace::install();

        let mut m = Mock::new();

        let mut spi = m.spi();
        let mut cs = m.pin();
        let mut sdn = m.pin();
        
        let mut busy = m.pin();
        let mut delay = m.delay();

        let mut radio = Sx128x{spi: spi.clone(), sdn: sdn.clone(), cs: cs.clone(), busy: busy.clone(), delay: delay.clone(), c: None, err: None };

        std::println!("new {:p}", &radio);

        radio.bind();

        let ptr = radio.to_c();

        assert!(ptr != (0 as *mut libc::c_void), "to_c is not void");

        std::println!("Radio: {:?}, ctx: {:?}", ptr, radio.c.unwrap().ctx);

        let r = radio.from_existing(ptr);
    }
}