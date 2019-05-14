//! Vectors contains functions to generate test vectors for driver testing

extern crate embedded_spi;
use self::embedded_spi::mock::{Spi, Pin, Delay};
pub use self::embedded_spi::mock::{Mock, MockTransaction as Mt};

use std::vec::Vec;

use bindings as sx1280;

pub fn reset(_spi: &Spi, _cs: &Pin, sdn: &Pin, _busy: &Pin, delay: &Delay) -> Vec<Mt> {
    vec![
        Mt::set_low(sdn),
        Mt::delay_ms(delay, 1),
        Mt::set_high(sdn),
        Mt::delay_ms(delay, 10),
    ]
}


pub fn status(spi: &Spi, cs: &Pin, _sdn: &Pin, busy: &Pin, _delay: &Delay) -> Vec<Mt> {
    vec![
        Mt::is_high(&busy, false),
        Mt::set_low(&cs),
        Mt::write(&spi, &[sx1280::RadioCommands_u_RADIO_GET_STATUS as u8, 0]),
        Mt::transfer(&spi, &[0x00], &[0x00]),
        Mt::set_high(&cs),
        Mt::is_high(&busy, true),
        Mt::is_high(&busy, false),
    ]
}

pub fn firmware_version(spi: &Spi, cs: &Pin, _sdn: &Pin, busy: &Pin, _delay: &Delay, version: u16) -> Vec<Mt> {
    vec![
        Mt::is_high(&busy, false),
        Mt::set_low(&cs),
        Mt::spi_read(&spi, &[
            sx1280::RadioCommands_u_RADIO_READ_REGISTER as u8,
            (sx1280::REG_LR_FIRMWARE_VERSION_MSB >> 8) as u8, 
            (sx1280::REG_LR_FIRMWARE_VERSION_MSB >> 0) as u8,
            0
        ], &[
            (version >> 8) as u8,
            (version >> 0) as u8,
        ]),
        Mt::set_high(&cs),
        Mt::is_high(&busy, true),
        Mt::is_high(&busy, false),
    ]
}