
#[macro_use] extern crate log;

extern crate structopt;
use structopt::StructOpt;

extern crate humantime;

extern crate embedded_spi;
use embedded_spi::hal::{HalInst, HalDelay};

extern crate embedded_hal;

extern crate radio;
use radio::{State as _};

extern crate radio_sx128x;
use radio_sx128x::prelude::*;

extern crate pcap_file;

mod options;
use options::*;

mod operations;
use operations::*;

fn main() {
    // Load options
    let opts = Options::from_args();

    // Setup logging
    opts.log.init();

    debug!("Connecting to SPI device");

    let HalInst{base: _, spi, pins} = opts.spi_config.load().unwrap();

    let rf_config = opts.rf_config();

    debug!("Config: {:?}", rf_config);

    info!("Initialising Radio");
    let mut radio = Sx128x::spi(spi, pins.cs, pins.busy, pins.ready, pins.reset, HalDelay{}, &rf_config).expect("error creating device");

    let operation = opts.command.operation();

    info!("Executing command");
    match &opts.command {
        Command::FirmwareVersion => {
            let version = radio.firmware_version().expect("error fetching chip version");
            info!("Silicon version: 0x{:X}", version);
            return
        },
        _ => {
            do_command(&mut radio, operation.unwrap()).expect("error executing command");
        }
    }

    //let _ = radio.set_state(State::Sleep);
}

