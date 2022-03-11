extern crate libc;

use log::{debug, error, info, trace};
use structopt::StructOpt;

use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::FmtSubscriber;

use driver_pal::hal::{HalDelay, HalInst};
use radio::helpers::do_operation;
use radio_sx128x::prelude::*;

mod options;
use options::*;

fn main() {
    // Load options
    let opts = Options::from_args();

    // Initialise logging
    let filter = EnvFilter::from_default_env()
        .add_directive(format!("radio_sx128x={}", opts.log_level).parse().unwrap())
        .add_directive(format!("sx128x_util={}", opts.log_level).parse().unwrap())
        .add_directive(format!("driver_cp2130=info").parse().unwrap());

    let _ = FmtSubscriber::builder()
        .with_env_filter(filter)
        .without_time()
        .try_init();

    debug!("Connecting to platform SPI");
    trace!("with config: {:?}", opts.spi_config);

    let HalInst { base: _, spi, pins } = match HalInst::load(&opts.spi_config) {
        Ok(v) => v,
        Err(e) => {
            error!("Error connecting to platform HAL: {:?}", e);
            return;
        }
    };

    let rf_config = opts.rf_config();

    debug!("Config: {:?}", rf_config);

    info!("Initialising Radio");
    let mut radio = Sx128x::spi(
        spi,
        pins.cs,
        pins.busy,
        pins.ready,
        pins.reset,
        HalDelay {},
        &rf_config,
    )
    .expect("error creating device");

    let operation = opts.command.operation();

    info!("Executing command");
    match &opts.command {
        Command::FirmwareVersion => {
            let version = radio
                .firmware_version()
                .expect("error fetching chip version");
            info!("Silicon version: 0x{:X}", version);
            return;
        }
        _ => {
            if let Some(mut syncword) = opts.syncword {
                if let Err(e) = radio.set_syncword(1, &mut syncword.0) {
                    error!("Error setting syncword: {:?}", e);
                }
                debug!("Syncword: 0x{:x?}", syncword.0);
            }
            do_operation(&mut radio, operation.unwrap()).expect("error executing command");
        }
    }

    //let _ = radio.set_state(State::Sleep);
}
