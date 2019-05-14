
#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::{TermLogger, LevelFilter};

extern crate structopt;
use structopt::StructOpt;

extern crate linux_embedded_hal;
use linux_embedded_hal::{spidev, Spidev, Pin as PinDev, Delay};
use linux_embedded_hal::sysfs_gpio::Direction;


extern crate radio_sx128x;
use radio_sx128x::{Sx128x, Settings};

#[derive(StructOpt)]
#[structopt(name = "Sx128x-util", about = "A Command Line Interface (CLI) for interacting with a local Sx128x radio device")]
pub struct Options {

    #[structopt(subcommand)]
    /// Request for remote-hal server
    command: Command,


    #[structopt(long = "spi", default_value = "/dev/spidev0.0", env = "SX128X_SPI")]
    /// Specify the hostname of the remote-hal server
    spi: String,

    /// Chip Select (output) pin
    #[structopt(long = "cs-pin", default_value = "4", env = "SX128X_CS")]
    cs: u64,

    /// Reset (output) pin
    #[structopt(long = "rst-pin", default_value = "18", env = "SX128X_RST")]
    rst: u64,

    /// Busy (input) pin
    #[structopt(long = "busy-pin", default_value = "17", env = "SX128X_BUSY")]
    busy: u64,

    /// Busy (input) pin
    #[structopt(long = "baud", default_value = "1000000", env = "SX128X_BAUD")]
    baud: u32,


    #[structopt(long = "log-level", default_value = "info")]
    /// Enable verbose logging
    level: LevelFilter,
}

#[derive(StructOpt, PartialEq, Debug)]
pub enum Command {
    #[structopt(name="firmware-version")]
    /// Fetch the device firmware version
    FirmwareVersion,
}

fn main() {
    // Load options
    let opts = Options::from_args();

    // Setup logging
    TermLogger::init(opts.level, simplelog::Config::default()).unwrap();

    // Connect to hardware
    let mut spi = Spidev::open(opts.spi).expect("error opening spi device");
    let mut config = spidev::SpidevOptions::new();
    config.mode(spidev::SPI_MODE_0);
    config.max_speed_hz(opts.baud);
    spi.configure(&config).expect("error configuring spi device");

    let cs = PinDev::new(opts.cs);
    cs.export().expect("error exporting cs pin");
    cs.set_direction(Direction::Out).expect("error setting cs pin direction");

    let rst = PinDev::new(opts.rst);
    rst.export().expect("error exporting rst pin");
    rst.set_direction(Direction::Out).expect("error setting rst pin direction");

    let busy = PinDev::new(opts.busy);
    busy.export().expect("error exporting busy pin");
    busy.set_direction(Direction::Out).expect("error setting busy pin direction");

    let settings = Settings::default();
    let mut radio = Sx128x::spi(spi, cs, busy, rst, Delay{}, settings).expect("error creating device");

    // TODO: the rest
    match opts.command {
        Command::FirmwareVersion => {
            let version = radio.firmware_version().expect("error fetching firmware version");
            info!("Firmware version: {}", version);
        }
        //_ => warn!("unsuppored command: {:?}", opts.command),
    }

}