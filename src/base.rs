//! Basic HAL functions for communicating with the radio device

use hal::blocking::{delay};
use hal::digital::v2::{InputPin, OutputPin};

use embedded_spi::{Transactional, PinState};
use embedded_spi::{Error as WrapError};

use crate::{Sx128x, Sx128xError};
use crate::bindings::{self as sx1280};

/// Comms implementation can be generic over SPI or UART connections
pub trait Hal<CommsError, PinError> {
    /// Wait on radio device busy
    fn wait_busy(&mut self) -> Result<(), Sx128xError<CommsError, PinError>>;
    
    /// Write the specified command and data
    fn cmd_write(&mut self, command: u8, data: &[u8]) -> Result<(), Sx128xError<CommsError, PinError>>;
    /// Read the specified command and data
    fn cmd_read(&mut self, command: u8, data: &mut [u8]) -> Result<(), Sx128xError<CommsError, PinError>>;
    /// Write to the specified register
    fn reg_write(&mut self, reg: u16, data: &[u8]) -> Result<(), Sx128xError<CommsError, PinError>>;
    /// Read from the specified register
    fn reg_read(&mut self, reg: u16, data: &mut [u8]) -> Result<(), Sx128xError<CommsError, PinError>>;
    /// Write to the specified buffer
    fn buff_write(&mut self, offset: u8, data: &[u8]) -> Result<(), Sx128xError<CommsError, PinError>>;
    /// Read from the specified buffer
    fn buff_read(&mut self, offset: u8, data: &mut [u8]) -> Result<(), Sx128xError<CommsError, PinError>>;
}

impl<Comms, CommsError, Output, Input, PinError, Delay> Sx128x<Comms, CommsError, Output, Input, PinError, Delay>
where
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{ 
    /// Reset the radio
    pub fn reset(&mut self) -> Result<(), Sx128xError<CommsError, PinError>> {
        self.sdn.set_low().map_err(|e| Sx128xError::Pin(e) )?;
        self.delay.delay_ms(1);
        self.sdn.set_high().map_err(|e| Sx128xError::Pin(e) )?;
        self.delay.delay_ms(10);

        Ok(())
    }
}

impl<T, CommsError, PinError> Hal<CommsError, PinError> for T
where
    T: Transactional<Error=WrapError<CommsError, PinError>>,
{    
    /// Wait on radio device busy
    fn wait_busy(&mut self) -> Result<(), Sx128xError<CommsError, PinError>> {
        // TODO: timeouts here
        while self.spi_busy()? == PinState::High {}

        Ok(())
    }

    /// Write the specified command and data
    fn cmd_write(&mut self, command: u8, data: &[u8]) -> Result<(), Sx128xError<CommsError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 1] = [command as u8];
        self.wait_busy()?;
        let r = self.spi_write(&out_buf, data).map_err(|e| e.into() );
        self.wait_busy()?;
        r
    }

    /// Read the specified command and data
    fn cmd_read<'a>(&mut self, command: u8, data: &mut [u8]) -> Result<(), Sx128xError<CommsError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 2] = [command as u8, 0x00];
        self.wait_busy()?;
        let r = self.spi_read(&out_buf, data).map(|_| () ).map_err(|e| e.into() );
        self.wait_busy()?;
        r
    }

    /// Write to the specified register
    fn reg_write(&mut self, reg: u16, data: &[u8]) -> Result<(), Sx128xError<CommsError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 3] = [
            sx1280::RadioCommands_u_RADIO_WRITE_REGISTER as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
        ];
        self.wait_busy()?;
        let r = self.spi_write(&out_buf, data).map_err(|e| e.into() );
        self.wait_busy()?;
        r
    }

    /// Read from the specified register
    fn reg_read<'a>(&mut self, reg: u16, data: &mut [u8]) -> Result<(), Sx128xError<CommsError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 4] = [
            sx1280::RadioCommands_u_RADIO_READ_REGISTER as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
            0,
        ];
        self.wait_busy()?;
        let r = self.spi_read(&out_buf, data).map(|_| () ).map_err(|e| e.into() );
        self.wait_busy()?;
        r
    }

    /// Write to the specified buffer
    fn buff_write(&mut self, offset: u8, data: &[u8]) -> Result<(), Sx128xError<CommsError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 2] = [
            sx1280::RadioCommands_u_RADIO_WRITE_BUFFER as u8,
            offset,
        ];
        self.wait_busy()?;
        let r = self.spi_write(&out_buf, data).map_err(|e| e.into() );
        self.wait_busy()?;
        r
    }

    /// Read from the specified buffer
    fn buff_read<'a>(&mut self, offset: u8, data: &mut [u8]) -> Result<(), Sx128xError<CommsError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 3] = [
            sx1280::RadioCommands_u_RADIO_READ_BUFFER as u8,
            offset,
            0
        ];
        self.wait_busy()?;
        let r = self.spi_read(&out_buf, data).map(|_| () ).map_err(|e| e.into() );
        self.wait_busy()?;
        r
    }
}

