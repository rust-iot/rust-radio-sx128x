//! Basic HAL functions for communicating with the radio device

use hal::blocking::{spi, delay};
use hal::digital::v2::{InputPin, OutputPin};


use crate::{Sx128x, Sx128xError};
use crate::bindings::{self as sx1280};

impl<Spi, SpiError, Output, Input, PinError, Delay> Sx128x<Spi, SpiError, Output, Input, PinError, Delay>
where
    Spi: spi::Transfer<u8, Error = SpiError> + spi::Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{

    /// Reset the radio
    pub fn reset(&mut self) -> Result<(), Sx128xError<SpiError, PinError>> {
        self.sdn.set_low().map_err(|e| Sx128xError::Pin(e) )?;
        self.delay.delay_ms(1);
        self.sdn.set_high().map_err(|e| Sx128xError::Pin(e) )?;
        self.delay.delay_ms(10);

        Ok(())
    }

    /// Wait on radio device busy
    pub fn wait_busy(&mut self) -> Result<(), Sx128xError<SpiError, PinError>> {
        // TODO: timeouts here
        while self.busy.is_high().map_err(|e| Sx128xError::Pin(e) )? {}

        Ok(())
    }

    /// Read data from a specified register address
    /// This consumes the provided input data array and returns a reference to this on success
    pub fn read<'a>(&mut self, command: &[u8], mut data: &'a mut [u8]) -> Result<&'a [u8], Sx128xError<SpiError, PinError>> {
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
    
    /// Write the specified command and data
    pub fn cmd_write(&mut self, command: u8, data: &[u8]) -> Result<(), Sx128xError<SpiError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 1] = [command as u8];
        self.write(&out_buf, data)
    }

    /// Read the specified command and data
    pub fn cmd_read<'a>(&mut self, command: u8, data: &'a mut [u8]) -> Result<&'a [u8], Sx128xError<SpiError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 2] = [command as u8, 0x00];
        self.read(&out_buf, data)
    }

    /// Write to the specified register
    pub fn reg_write(&mut self, reg: u16, data: &[u8]) -> Result<(), Sx128xError<SpiError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 3] = [
            sx1280::RadioCommands_u_RADIO_WRITE_REGISTER as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
        ];
        self.write(&out_buf, data)
    }

    /// Read from the specified register
    pub fn reg_read<'a>(&mut self, reg: u16, data: &'a mut [u8]) -> Result<&'a [u8], Sx128xError<SpiError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 4] = [
            sx1280::RadioCommands_u_RADIO_READ_REGISTER as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
            0,
        ];
        self.read(&out_buf, data)
    }

    /// Write to the specified buffer
    pub fn buff_write(&mut self, offset: u8, data: &[u8]) -> Result<(), Sx128xError<SpiError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 2] = [
            sx1280::RadioCommands_u_RADIO_WRITE_BUFFER as u8,
            offset,
        ];
        self.write(&out_buf, data)
    }

    /// Read from the specified buffer
    pub fn buff_read<'a>(&mut self, offset: u8, data: &'a mut [u8]) -> Result<&'a [u8], Sx128xError<SpiError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 3] = [
            sx1280::RadioCommands_u_RADIO_READ_BUFFER as u8,
            offset,
            0
        ];
        self.read(&out_buf, data)
    }
}