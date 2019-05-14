
extern crate std;

use embedded_spi::compat::{Conv};

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
    /// Create and bind an internal C object to support the bound C api
    pub fn bind(s: &mut Self) {
        std::println!("Binding Object: {:p}", s);

        let ctx = Self::to_c_ptr(s);

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
        s.c = Some(c);
    }

    #[allow(dead_code)]
    extern fn set_reset(ctx: *mut libc::c_void, value: bool) -> i32 {
        let sx128x = Self::from_c_ptr(ctx);
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

    #[allow(dead_code)]
    extern fn get_busy(ctx: *mut libc::c_void) -> i32 {
        let sx128x = Self::from_c_ptr(ctx);
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

    #[allow(dead_code)]
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
    
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    extern fn delay_ms(ctx: *mut libc::c_void, ms: u32) {
        let sx128x = Self::from_c_ptr(ctx);;
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

    pub fn status2(&mut self) -> Result<sx1280::RadioStatus_t, Sx128xError<SpiError, PinError>> {
        // Update rust object pointer to c object context
        let mut ctx = self.c.unwrap();

        let status = unsafe { sx1280::SX1280GetStatus(&mut ctx) };
        Ok(status)
    }

}


#[cfg(test)]
mod tests {
    use crate::Sx128x;
    use crate::bindings as sx1280;

    extern crate std;

    extern crate embedded_spi;
    use embedded_spi::compat::{Conv};
    use self::embedded_spi::mock::{Mock, MockTransaction as Mt};

    extern crate color_backtrace;

    type Radio = Sx128x<embedded_spi::mock::Spi, embedded_spi::mock::Error<(), ()>, embedded_spi::mock::Pin, embedded_spi::mock::Pin, (), embedded_spi::mock::Delay>;

    #[test]
    fn test_compat() {
        color_backtrace::install();

        let mut m = Mock::new();

        let spi = m.spi();
        let cs = m.pin();
        let sdn = m.pin();
        
        let busy = m.pin();
        let delay = m.delay();

        let mut radio = Sx128x{spi: spi.clone(), sdn: sdn.clone(), cs: cs.clone(), busy: busy.clone(), delay: delay.clone(), c: None, err: None };

        std::println!("new {:p}", &radio);

        Radio::bind(&mut radio);
        let ptr = Radio::to_c_ptr(&mut radio);

        assert!(ptr != (0 as *mut libc::c_void), "to_c is not void");

        std::println!("Radio: {:?}, ctx: {:?}", ptr, radio.c.unwrap().ctx);
        assert_eq!(ptr, radio.c.unwrap().ctx);


        m.expect(&[
            Mt::set_high(&sdn),
        ]);

        Radio::set_reset(ptr, true);
        
        m.finalise();

    }

    #[test]
    fn test_bindings() {
        color_backtrace::install();

        let mut m = Mock::new();

        let spi = m.spi();
        let cs = m.pin();
        let sdn = m.pin();
        
        let busy = m.pin();
        let delay = m.delay();

        let mut radio = Sx128x{spi: spi.clone(), sdn: sdn.clone(), cs: cs.clone(), busy: busy.clone(), delay: delay.clone(), c: None, err: None };

        std::println!("new {:p}", &radio);

        Sx128x::bind(&mut radio);

        std::println!("Test status command");

        m.expect(&[
            Mt::set_low(&cs),
            Mt::write(&spi, &[sx1280::RadioCommands_u_RADIO_GET_STATUS as u8, 0]),
            Mt::transfer(&spi, &[0x00], &[0x00]),
            Mt::set_high(&cs),
        ]);

        radio.status2();

        m.finalise();
    }
}