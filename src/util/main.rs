

extern crate libc;

use structopt::StructOpt;
use log::{debug, info};

use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::filter::{EnvFilter};

use driver_pal::hal::{HalInst, HalDelay};


use radio_sx128x::prelude::*;


mod options;
use options::*;

mod operations;
use operations::*;

fn main() {
    // Load options
    let opts = Options::from_args();

    // Initialise logging
    let filter = EnvFilter::from_default_env()
    .add_directive(format!("radio_sx128x={}", opts.log_level).parse().unwrap())
    .add_directive(format!("sx128x_util={}", opts.log_level).parse().unwrap())
    .add_directive(format!("driver_cp2130=warn").parse().unwrap());

    let _ = FmtSubscriber::builder()
        .with_env_filter(filter)
        .without_time()
        .try_init();

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

