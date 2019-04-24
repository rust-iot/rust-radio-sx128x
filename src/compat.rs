
extern crate std;

use core::{mem, ptr, slice};

use hal::blocking::{spi, delay};
use hal::digital::v2::{InputPin, OutputPin};

use crate::{Sx128x, Settings};
use crate::sx1280::{SX1280_s};

impl<SpiError, Spi, PinError, Output, Input, Delay> Sx128x<Spi, Output, Input, Delay>
where
    Spi: spi::Transfer<u8, Error = SpiError> + spi::Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{
    /// Build a rust object containing a c object containing a pointer to the rust object
    pub(crate) fn build(spi: Spi, sdn: Output, cs: Output, busy: Input, delay: Delay) -> Self {
        unsafe {
            // Create base C object
            let c = SX1280_s {
                ctx: mem::uninitialized(),
                reset: Some(Self::ext_reset),
                
                write_command: Some(Self::write_command),
                read_command: Some(Self::read_command),
                write_registers: Some(Self::write_registers),
                read_registers: Some(Self::read_registers),
                write_register: Some(Self::write_register),
                read_register: Some(Self::read_register),
                write_buffer: Some(Self::write_buffer),
                read_buffer: Some(Self::read_buffer),

                delay_ms: Some(Self::delay_ms),
            };

            // Create rust object
            let mut sx128x = Sx128x { spi, sdn, cs, busy, delay, c: c };

            // Bind rust object pointer to c object context
            sx128x.c.ctx = sx128x.to_c();

            // Return rust object
            sx128x
        }
    }

    pub(crate) fn to_c(&mut self) -> *mut libc::c_void {
        let ptr = unsafe {
            self as *mut Self as *mut libc::c_void
        };
        self.c.ctx = ptr;
        std::println!("Radio: {:?}", ptr);
        ptr;
    }

    // extern functions used by c hal
    // todo: errors are not bubbled through these functions

    fn from_c<'a>(ctx: *mut libc::c_void) -> &'a mut Self {
        unsafe {
            //assert!(ctx == ptr::null());
            let sx1280 = ctx as *mut SX1280_s;
            let sx128x_ptr = (*sx1280).ctx as *mut libc::c_void;
            let sx128x = sx128x_ptr as *mut Sx128x<Spi, Output, Input, Delay>;
            &mut *sx128x
        }
    }

    fn from_existing<'a>(&self, ctx: *mut libc::c_void) -> &'a mut Self {
        Self::from_c(ctx)
    }

    extern fn ext_reset(ctx: *mut libc::c_void) {
        let sx128x = Self::from_c(ctx);
        let _ = sx128x.reset();
    }

    extern fn write_buffer(ctx: *mut libc::c_void, offset: u8, buffer: *mut u8, size: u8) {
        let sx128x = Self::from_c(ctx);
        let data: &[u8] = unsafe {slice::from_raw_parts_mut(buffer, size as usize) };
        let _ = sx128x.buff_write(offset, data);
    }
    
    extern fn read_buffer(ctx: *mut libc::c_void, offset: u8, buffer: *mut u8, size: u8) {
        let sx128x = Self::from_c(ctx);
        let data: &mut [u8] = unsafe { slice::from_raw_parts_mut(buffer, size as usize) };
        let _ = sx128x.buff_read(offset, data);
    }

    extern fn write_command(ctx: *mut libc::c_void, command: u8, buffer: *mut u8, size: u8) {
        let sx128x = Self::from_c(ctx);
        let data: &[u8] = unsafe { slice::from_raw_parts(buffer, size as usize) };
        let _ = sx128x.cmd_write(command, data);
    }
    
    extern fn read_command(ctx: *mut libc::c_void, command: u8, buffer: *mut u8, size: u8) {
        let sx128x = Self::from_c(ctx);
        let data: &mut [u8] = unsafe{ slice::from_raw_parts_mut(buffer, size as usize) };
        let _ = sx128x.cmd_read(command, data);
    }

    extern fn write_registers(ctx: *mut libc::c_void, address: u16, buffer: *mut u8, size: u8) {
        let sx128x = Self::from_c(ctx);
        let data: &[u8] = unsafe{ slice::from_raw_parts(buffer, size as usize) };
        let _ = sx128x.reg_write(address, data);
    }
    
    extern fn read_registers(ctx: *mut libc::c_void, address: u16, buffer: *mut u8, size: u8) {
        let sx128x = Self::from_c(ctx);
        let data: &mut [u8] = unsafe{ slice::from_raw_parts_mut(buffer, size as usize) };
        let _ = sx128x.reg_read(address, data);
    }

    extern fn write_register(ctx: *mut libc::c_void, address: u16, value: u8) {
        let sx128x = Self::from_c(ctx);
        let data: [u8; 1] = [value];
        let _ = sx128x.reg_write(address, &data);
    }
    extern fn read_register(ctx: *mut libc::c_void, address: u16) -> u8 {
        let sx128x = Self::from_c(ctx);
        let mut data: [u8; 1] = [0];
        let _ = sx128x.reg_read(address, &mut data);
        data[0]
    }

    extern fn delay_ms(ctx: *mut libc::c_void, ms: u32) {
        let sx128x = Self::from_c(ctx);;
        let _ = sx128x.delay.delay_ms(ms as u32);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    extern crate std;

    extern crate embedded_hal_mock;
    use self::embedded_hal_mock::engine::*;


    #[test]
    fn ffi_casts() {
        let mut engine = Engine::new();

        let mut spi = engine.spi();
        let mut sdn = engine.pin();
        let mut cs = engine.pin();
        let mut busy = engine.pin();
        let mut delay = engine.delay();

        let mut radio = Sx128x::build(spi.clone(), sdn.clone(), cs.clone(), busy.clone(), delay.clone());

        let ptr = radio.to_c();

        assert!(ptr != (0 as *mut libc::c_void), "to_c is not void");

        std::println!("Radio: {:?}, ctx: {:?}", ptr, radio.c.ctx);

        let r = radio.from_existing(ptr);

        //assert_eq!(radio.c.ctx, r.c.ctx);


    }
}