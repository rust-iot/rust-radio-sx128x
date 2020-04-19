//! Basic HAL functions for communicating with the radio device

use core::fmt::Debug;

use hal::blocking::delay::DelayMs;
use hal::blocking::spi::Transactional;

use embedded_spi::{Reset, Busy, PinState, PrefixRead, PrefixWrite};
use embedded_spi::{Error as SpiError};

use crate::{Error};
use crate::device::*;

/// Hal implementation can be generic over SPI or UART connections
pub trait Hal<CommsError: Debug + Sync + Send, PinError: Debug + Sync + Send> {

    /// Reset the device
    fn reset(&mut self) -> Result<(), Error<CommsError, PinError>>;

    /// Wait on radio device busy
    fn get_busy(&mut self) -> Result<PinState, Error<CommsError, PinError>>;

    /// Delay for the specified time
    fn delay_ms(&mut self, ms: u32);

    /// Write the specified command and data
    fn write_cmd(&mut self, command: u8, data: &[u8]) -> Result<(), Error<CommsError, PinError>>;
    /// Read the specified command and data
    fn read_cmd(&mut self, command: u8, data: &mut [u8]) -> Result<(), Error<CommsError, PinError>>;
    
    /// Write to the specified register
    fn write_regs(&mut self, reg: u16, data: &[u8]) -> Result<(), Error<CommsError, PinError>>;
    /// Read from the specified register
    fn read_regs(&mut self, reg: u16, data: &mut [u8]) -> Result<(), Error<CommsError, PinError>>;
    
    /// Write to the specified buffer
    fn write_buff(&mut self, offset: u8, data: &[u8]) -> Result<(), Error<CommsError, PinError>>;
    /// Read from the specified buffer
    fn read_buff(&mut self, offset: u8, data: &mut [u8]) -> Result<(), Error<CommsError, PinError>>;

    /// Wait on radio device busy
    fn wait_busy(&mut self) -> Result<(), Error<CommsError, PinError>> {
        // TODO: timeouts here
        let mut timeout = 0;
        while self.get_busy()? == PinState::High {
            self.delay_ms(1);
            timeout += 1;

            if timeout > BUSY_TIMEOUT_MS {
                error!("Busy timeout after {} ms", BUSY_TIMEOUT_MS);
                return Err(Error::BusyTimeout)
            }
        }

        Ok(())
    }

    /// Read a single u8 value from the specified register
    fn read_reg(&mut self, reg: u16) -> Result<u8, Error<CommsError, PinError>> {
        let mut incoming = [0u8; 1];
        self.read_regs(reg.into(), &mut incoming)?;
        Ok(incoming[0])
    }

    /// Write a single u8 value to the specified register
    fn write_reg(&mut self, reg: u16, value: u8) -> Result<(), Error<CommsError, PinError>> {
        self.write_regs(reg.into(), &[value])?;
        Ok(())
    }

    /// Update the specified register with the provided value & mask
    fn update_reg(&mut self, reg: u16, mask: u8, value: u8) -> Result<u8, Error<CommsError, PinError>> {
        let existing = self.read_reg(reg)?;
        let updated = (existing & !mask) | (value & mask);
        self.write_reg(reg, updated)?;
        Ok(updated)
    }
}

impl<T, CommsError, PinError> Hal<CommsError, PinError> for T
where
    T: Transactional<u8, Error=SpiError<CommsError, PinError>> + PrefixRead<Error=SpiError<CommsError, PinError>> + PrefixWrite<Error=SpiError<CommsError, PinError>>,
    T: Reset<Error=SpiError<CommsError, PinError>>,
    T: Busy<Error=SpiError<CommsError, PinError>>,
    T: DelayMs<u32>,
    CommsError: Debug + Sync + Send,
    PinError: Debug + Sync + Send,
{    
    /// Reset the radio
    fn reset(&mut self) -> Result<(), Error<CommsError, PinError>> {
        self.delay_ms(20);
        self.set_reset(PinState::Low).map_err(|e| Error::from(e) )?;
        self.delay_ms(50);
        self.set_reset(PinState::High).map_err(|e| Error::from(e) )?;
        self.delay_ms(20);

        Ok(())
    }

    fn get_busy(&mut self) -> Result<PinState, Error<CommsError, PinError>> {
        let busy = self.get_busy()?;
        Ok(busy)
    }


    /// Delay for the specified time
    fn delay_ms(&mut self, ms: u32) {
        self.delay_ms(ms);
    }

    /// Write the specified command and data
    fn write_cmd(&mut self, command: u8, data: &[u8]) -> Result<(), Error<CommsError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 1] = [command as u8];
        
        trace!("write_cmd cmd: {:02x?} data: {:02x?}", out_buf, data);

        self.wait_busy()?;
        let r = self.spi_write(&out_buf, data).map_err(|e| e.into() );
        self.wait_busy()?;
        r
    }

    /// Read the specified command and data
    fn read_cmd<'a>(&mut self, command: u8, data: &mut [u8]) -> Result<(), Error<CommsError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 2] = [command as u8, 0x00];
        
        self.wait_busy()?;
        let r = self.spi_read(&out_buf, data).map(|_| () ).map_err(|e| e.into() );
        self.wait_busy()?;

        trace!("read_cmd cmd: {:02x?} data: {:02x?}", out_buf, data);

        r
    }

    /// Write to the specified register
    fn write_regs(&mut self, reg: u16, data: &[u8]) -> Result<(), Error<CommsError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 3] = [
            Commands::WiteRegister as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
        ];

        trace!("write_regs cmd: {:02x?} data: {:02x?}", out_buf, data);

        self.wait_busy()?;
        let r = self.spi_write(&out_buf, data).map_err(|e| e.into() );
        self.wait_busy()?;
        r
    }

    /// Read from the specified register
    fn read_regs<'a>(&mut self, reg: u16, data: &mut [u8]) -> Result<(), Error<CommsError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 4] = [
            Commands::ReadRegister as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
            0,
        ];

        self.wait_busy()?;
        let r = self.spi_read(&out_buf, data).map(|_| () ).map_err(|e| e.into() );
        self.wait_busy()?;

        trace!("read_regs cmd: {:02x?} data: {:02x?}", out_buf, data);
        
        r
    }

    /// Write to the specified buffer
    fn write_buff(&mut self, offset: u8, data: &[u8]) -> Result<(), Error<CommsError, PinError>> {
        // Setup register write command
        let out_buf: [u8; 2] = [
            Commands::WriteBuffer as u8,
            offset,
        ];
        
        trace!("write_buff cmd: {:02x?}", out_buf);

        self.wait_busy()?;
        let r = self.spi_write(&out_buf, data).map_err(|e| e.into() );
        self.wait_busy()?;
        r
    }

    /// Read from the specified buffer
    fn read_buff<'a>(&mut self, offset: u8, data: &mut [u8]) -> Result<(), Error<CommsError, PinError>> {
        // Setup register read command
        let out_buf: [u8; 3] = [
            Commands::ReadBuffer as u8,
            offset,
            0
        ];
        trace!(" data: {:02x?}", out_buf);
        self.wait_busy()?;
        let r = self.spi_read(&out_buf, data).map(|_| () ).map_err(|e| e.into() );
        self.wait_busy()?;

        trace!("read_buff cmd: {:02x?} data: {:02x?}", out_buf, data);

        r
    }
}

