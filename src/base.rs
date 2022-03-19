//! Basic HAL functions for communicating with the radio device

use core::fmt::Debug;

use log::{error, trace};

use embedded_hal::digital::PinState;
use embedded_hal::digital::blocking::{InputPin, OutputPin};


use embedded_hal::delay::blocking::DelayUs;
use embedded_hal::spi::blocking::{Transactional, Operation};

use crate::{device::*, SpiBase};
use crate::Error;

/// Hal implementation can be generic over SPI or UART connections
pub trait Hal {
    type CommsError: Debug + 'static;
    type PinError: Debug + 'static;
    type DelayError: Debug + 'static;

    /// Reset the device
    fn reset(&mut self) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>>;

    /// Fetch radio device busy pin value
    fn get_busy(&mut self) -> Result<PinState, Error<Self::CommsError, Self::PinError, Self::DelayError>>;

    /// Fetch radio device ready / irq (DIO) pin value
    fn get_dio(&mut self) -> Result<PinState, Error<Self::CommsError, Self::PinError, Self::DelayError>>;

    /// Delay for the specified time
    fn delay_ms(&mut self, ms: u32) -> Result<(), Self::DelayError>;

    /// Delay for the specified time
    fn delay_us(&mut self, us: u32) -> Result<(), Self::DelayError>;

    /// Write the specified command and data
    fn write_cmd(
        &mut self,
        command: u8,
        data: &[u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>>;
    /// Read the specified command and data
    fn read_cmd(
        &mut self,
        command: u8,
        data: &mut [u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>>;

    /// Write to the specified register
    fn write_regs(
        &mut self,
        reg: u16,
        data: &[u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>>;
    /// Read from the specified register
    fn read_regs(
        &mut self,
        reg: u16,
        data: &mut [u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>>;

    /// Write to the specified buffer
    fn write_buff(
        &mut self,
        offset: u8,
        data: &[u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>>;

    /// Read from the specified buffer
    fn read_buff(
        &mut self,
        offset: u8,
        data: &mut [u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>>;

    /// Wait on radio device busy
    fn wait_busy(&mut self) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        // TODO: timeouts here
        let mut timeout = 0;
        while self.get_busy()? == PinState::High {
            self.delay_ms(1).map_err(Error::Delay)?;
            timeout += 1;

            if timeout > BUSY_TIMEOUT_MS {
                error!("Busy timeout after {} ms", BUSY_TIMEOUT_MS);
                return Err(Error::BusyTimeout);
            }
        }

        Ok(())
    }

    /// Read a single u8 value from the specified register
    fn read_reg(&mut self, reg: u16) -> Result<u8, Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        let mut incoming = [0u8; 1];
        self.read_regs(reg.into(), &mut incoming)?;
        Ok(incoming[0])
    }

    /// Write a single u8 value to the specified register
    fn write_reg(
        &mut self,
        reg: u16,
        value: u8,
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        self.write_regs(reg.into(), &[value])?;
        Ok(())
    }

    /// Update the specified register with the provided value & mask
    fn update_reg(
        &mut self,
        reg: u16,
        mask: u8,
        value: u8,
    ) -> Result<u8, Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        let existing = self.read_reg(reg)?;
        let updated = (existing & !mask) | (value & mask);
        self.write_reg(reg, updated)?;
        Ok(updated)
    }

    fn prefix_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>>;

    fn prefix_write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>>;
}

pub trait HalError {
    type E: Debug;
}

impl <T> HalError for T
where
    T: Hal,
{
    type E = Error<<T as Hal>::CommsError, <T as Hal>::PinError, <T as Hal>::DelayError>;
}

/// Base interface for radio device
pub struct Base <Spi: SpiBase, Cs: OutputPin, Busy: InputPin, Ready: InputPin, Sdn: OutputPin, Delay: DelayUs> {
    pub spi: Spi,
    pub cs: Cs,
    pub busy: Busy,
    pub ready: Ready,
    pub sdn: Sdn,
    pub delay: Delay,
}


impl<Spi, Cs, Busy, Ready, Sdn, PinError, Delay> Hal for Base<Spi, Cs, Busy, Ready, Sdn, Delay>
where
    Spi: SpiBase,
    <Spi as SpiBase>::Error: Debug + 'static,
    
    Cs: OutputPin<Error=PinError>,
    Busy: InputPin<Error=PinError>,
    Ready: InputPin<Error=PinError>,
    Sdn: OutputPin<Error=PinError>,
    PinError: Debug + 'static,

    Delay: DelayUs,
    <Delay as DelayUs>::Error: Debug + 'static,
{
    type CommsError = <Spi as SpiBase>::Error;
    type PinError = PinError;
    type DelayError = <Delay as DelayUs>::Error;

    /// Reset the radio
    fn reset(&mut self) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        self.delay_ms(20).map_err(Error::Delay)?;
        
        self.sdn.set_low().map_err(Error::Pin)?;
        
        self.delay_ms(50).map_err(Error::Delay)?;
        
        self.sdn.set_high().map_err(Error::Pin)?;
        
        self.delay_ms(20).map_err(Error::Delay)?;

        Ok(())
    }

    fn get_busy(&mut self) -> Result<PinState, Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        match self.busy.is_high().map_err(Error::Pin)? {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }

    fn get_dio(&mut self) -> Result<PinState, Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        match self.ready.is_high().map_err(Error::Pin)? {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }

    /// Delay for the specified time
    fn delay_ms(&mut self, ms: u32) -> Result<(), <Delay as DelayUs>::Error> {
        self.delay.delay_ms(ms)
    }

    /// Delay for the specified time
    fn delay_us(&mut self, ms: u32) -> Result<(), <Delay as DelayUs>::Error> {
        self.delay.delay_us(ms)
    }

    /// Write data with prefix, asserting CS as required
    fn prefix_write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        self.cs.set_low().map_err(Error::Pin)?;

        let r = self.spi.exec(&mut [
            Operation::Write(prefix),
            Operation::Write(data),
        ]).map_err(Error::Comms);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }

    /// Read data with prefix, asserting CS as required
    fn prefix_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        self.cs.set_low().map_err(Error::Pin)?;

        let r = self.spi.exec(&mut [
            Operation::Write(prefix),
            Operation::Read(data),
        ]).map_err(Error::Comms);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }

    /// Write the specified command and data
    fn write_cmd(
        &mut self,
        command: u8,
        data: &[u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        // Setup register write command
        let out_buf: [u8; 1] = [command as u8];

        trace!("write_cmd cmd: {:02x?} data: {:02x?}", out_buf, data);

        self.wait_busy()?;

        let r = self.prefix_write(&out_buf, data);
        
        self.wait_busy()?;
        r
    }

    /// Read the specified command and data
    fn read_cmd<'a>(
        &mut self,
        command: u8,
        data: &mut [u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        // Setup register read command
        let out_buf: [u8; 2] = [command as u8, 0x00];

        self.wait_busy()?;
        
        let r = self.prefix_read(&out_buf, data);
        
        self.wait_busy()?;

        trace!("read_cmd cmd: {:02x?} data: {:02x?}", out_buf, data);

        r
    }

    /// Write to the specified register
    fn write_regs(
        &mut self,
        reg: u16,
        data: &[u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        // Setup register write command
        let out_buf: [u8; 3] = [
            Commands::WiteRegister as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
        ];

        trace!("write_regs cmd: {:02x?} data: {:02x?}", out_buf, data);

        self.wait_busy()?;
        
        let r = self.prefix_write(&out_buf, data);

        self.wait_busy()?;
        r
    }

    /// Read from the specified register
    fn read_regs<'a>(
        &mut self,
        reg: u16,
        data: &mut [u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        // Setup register read command
        let out_buf: [u8; 4] = [
            Commands::ReadRegister as u8,
            ((reg & 0xFF00) >> 8) as u8,
            (reg & 0x00FF) as u8,
            0,
        ];

        self.wait_busy()?;
        
        let r = self.prefix_read(&out_buf, data);

        self.wait_busy()?;

        trace!("read_regs cmd: {:02x?} data: {:02x?}", out_buf, data);

        r
    }

    /// Write to the specified buffer
    fn write_buff(
        &mut self,
        offset: u8,
        data: &[u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        // Setup register write command
        let out_buf: [u8; 2] = [Commands::WriteBuffer as u8, offset];

        trace!("write_buff cmd: {:02x?}", out_buf);

        self.wait_busy()?;
        
        let r = self.prefix_write(&out_buf, data);

        self.wait_busy()?;
        r
    }

    /// Read from the specified buffer
    fn read_buff<'a>(
        &mut self,
        offset: u8,
        data: &mut [u8],
    ) -> Result<(), Error<Self::CommsError, Self::PinError, Self::DelayError>> {
        // Setup register read command
        let out_buf: [u8; 3] = [Commands::ReadBuffer as u8, offset, 0];
        trace!(" data: {:02x?}", out_buf);

        self.wait_busy()?;
        
        let r = self.prefix_read(&out_buf, data);

        self.wait_busy()?;

        trace!("read_buff cmd: {:02x?} data: {:02x?}", out_buf, data);

        r
    }
}
