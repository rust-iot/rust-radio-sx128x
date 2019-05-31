//! Vectors contains functions to generate test vectors for driver testing

extern crate embedded_spi;

use self::embedded_spi::{PinState};
use self::embedded_spi::mock::{Spi, Pin, Delay};
pub use self::embedded_spi::mock::{Mock, MockTransaction as Mt};

use std::vec::Vec;

use crate::device::*;

pub fn reset(spi: &Spi, sdn: &Pin, delay: &Delay) -> Vec<Mt> {
    vec![
        Mt::reset(spi, PinState::Low),
        Mt::delay_ms(1),
        Mt::reset(spi, PinState::High),
        Mt::delay_ms(10),
    ]
}

pub fn status(spi: &Spi, _sdn: &Pin, _delay: &Delay) -> Vec<Mt> {
    vec![
        Mt::busy(&spi, PinState::Low),
        Mt::spi_read(&spi, &[Commands::GetStatus as u8, 0], &[0x00]),
        Mt::busy(&spi, PinState::Low),
    ]
}

pub fn firmware_version(spi: &Spi, _sdn: &Pin, _delay: &Delay, version: u16) -> Vec<Mt> {
    vec![
        Mt::busy(&spi, PinState::Low),
        Mt::spi_read(&spi, &[
            Commands::ReadRegister as u8,
            (Registers::LrFirmwareVersionMsb as u16 >> 8) as u8, 
            (Registers::LrFirmwareVersionMsb as u16 >> 0) as u8,
            0
        ], &[ (version >> 8) as u8, (version >> 0) as u8 ]),
        Mt::busy(&spi, PinState::Low),
    ]
}