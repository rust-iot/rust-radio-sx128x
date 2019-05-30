
#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::{TermLogger, LevelFilter};

extern crate structopt;
use structopt::StructOpt;

extern crate humantime;
use humantime::Duration;

extern crate linux_embedded_hal;
use linux_embedded_hal::{spidev, Spidev, Pin as PinDev, Delay};
use linux_embedded_hal::sysfs_gpio::Direction;

extern crate radio;
use radio::{Transmit as _, Receive as _, Rssi as _};

extern crate radio_sx128x;
use radio_sx128x::{Sx128x, Info, Settings, Config};

#[derive(StructOpt)]
#[structopt(name = "Sx128x-util")]
/// A Command Line Interface (CLI) for interacting with a local Sx128x radio device
/// 
/// Configuration 1:  --spi=/dev/spidev0.0 --cs-pin 16 --rst-pin 17 --busy-pin 5 --ant-pin 23
/// 
/// Configuration 2:  --spi=/dev/spidev0.1 --cs-pin 13 --rst-pin 18 --busy-pin 8 --ant-pin ??
/// 
pub struct Options {

    #[structopt(subcommand)]
    /// Request for remote-hal server
    command: Command,


    #[structopt(long = "spi", default_value = "/dev/spidev0.0", env = "SX128X_SPI")]
    /// Specify the hostname of the remote-hal server
    spi: String,

    /// Chip Select (output) pin
    #[structopt(long = "cs-pin", default_value = "16", env = "SX128X_CS")]
    cs: u64,

    /// Reset (output) pin
    #[structopt(long = "rst-pin", default_value = "17", env = "SX128X_RST")]
    rst: u64,

    /// Antenna control pin
    #[structopt(long = "ant-pin", default_value = "23", env = "SX128X_ANT")]
    ant: u64,

    /// Busy (input) pin
    #[structopt(long = "busy-pin", default_value = "5", env = "SX128X_BUSY")]
    busy: u64,

    /// DIO1 pin
    #[structopt(long = "dio1-pin", default_value = "20", env = "SX128X_DIO1")]
    dio1: u64,

    /// Baud rate setting
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

    #[structopt(name="tx")]
    /// Transmit a (string) packet
    Transmit(Transmit),

    #[structopt(name="rx")]
    /// Receive a (string) packet
    Receive(Receive),

    #[structopt(name="rssi")]
    /// Poll for RSSI on the specified channel
    Rssi(Rssi),
}

#[derive(StructOpt, PartialEq, Debug)]
pub struct Transmit {
    /// Data to be transmitted
    #[structopt(long = "data")]
    data: String,

    /// Run continuously
    #[structopt(long = "continuous")]
    continuous: bool,

    /// Power in dBm
    #[structopt(long = "power", default_value="13")]
    power: i8,

    /// Specify period for transmission
    #[structopt(long = "period", default_value="1s")]
    pub period: Duration,

    /// Specify period for polling for device status
    #[structopt(long = "poll-interval", default_value="10ms")]
    poll_interval: Duration,
}

#[derive(StructOpt, PartialEq, Debug)]
pub struct Receive {
    /// Run continuously
    #[structopt(long = "continuous")]
    continuous: bool,

    /// Specify period for polling for device status
    #[structopt(long = "poll-interval", default_value="10ms")]
    poll_interval: Duration,
}

#[derive(StructOpt, PartialEq, Debug)]
pub struct Rssi {
    /// Specify period for RSSI polling
    #[structopt(long = "period", default_value="1s")]
    pub period: Duration,

    /// Run continuously
    #[structopt(long = "continuous")]
    continuous: bool,
}


fn main() {
    // Load options
    let opts = Options::from_args();

    // Setup logging
    TermLogger::init(opts.level, simplelog::Config::default()).unwrap();

    debug!("Connecting to SPI device");

    // Connect to hardware
    let mut spi = Spidev::open(opts.spi).expect("error opening spi device");
    let mut config = spidev::SpidevOptions::new();
    config.mode(spidev::SPI_MODE_0);
    config.max_speed_hz(opts.baud);
    spi.configure(&config).expect("error configuring spi device");

    debug!("Configuring I/O pins");

    let cs = PinDev::new(opts.cs);
    cs.export().expect("error exporting cs pin");
    cs.set_direction(Direction::Out).expect("error setting cs pin direction");

    let rst = PinDev::new(opts.rst);
    rst.export().expect("error exporting rst pin");
    rst.set_direction(Direction::Out).expect("error setting rst pin direction");

    let ant = PinDev::new(opts.ant);
    ant.export().expect("error exporting rst ant");
    ant.set_direction(Direction::Out).expect("error setting ant pin direction");

    // TODO: set ant output

    let busy = PinDev::new(opts.busy);
    busy.export().expect("error exporting busy pin");
    busy.set_direction(Direction::Out).expect("error setting busy pin direction");

    let dio1 = PinDev::new(opts.dio1);
    dio1.export().expect("error exporting dio1 pin");
    dio1.set_direction(Direction::Out).expect("error setting dio1 pin direction");

    debug!("Creating radio instance");

    let settings = Settings::default();
    let config = Config::default();

    debug!("Settings: {:?}", settings);
    debug!("Config: {:?}", config);

    let mut radio = Sx128x::spi(spi, cs, busy, rst, Delay{}, settings, &config).expect("error creating device");

    debug!("Executing command");

    // TODO: the rest
    match opts.command {
        Command::FirmwareVersion => {
            let version = radio.firmware_version().expect("error fetching firmware version");
            info!("Firmware version: 0x{:X}", version);
        },
        Command::Transmit(config) => {
            radio.set_power(config.power).expect("error setting power");

            loop {
                radio.start_transmit( config.data.as_bytes() ).expect("error starting send");
                loop {
                    let tx = radio.check_transmit().expect("error checking send");
                    if tx {
                        info!("Send complete");
                        break;
                    }
                    std::thread::sleep(*config.poll_interval);
                }

                if !config.continuous {
                    break;
                }

                std::thread::sleep(*config.period);
            }
        },
        Command::Receive(config) => {
            radio.start_receive().expect("error starting receive");
            loop {
                let rx = radio.check_receive(true).expect("error checking receive");
                if rx {
                    let mut buff = [0u8; 255];
                    let mut info = Info::default();
                    let n = radio.get_received(&mut info, &mut buff).expect("error fetching received data");

                    debug!("received data: {:?}", &buff[0..n as usize]);

                    let d = std::str::from_utf8(&buff[0..n as usize]).expect("error converting response to string");

                    info!("Received: '{}'", d);

                    if !config.continuous {
                        break
                    }
                }
                std::thread::sleep(*config.poll_interval);
            }
        },
        Command::Rssi(config) => {
            radio.start_receive().expect("error starting receive");
            loop {
                let rssi = radio.poll_rssi().expect("error fetching RSSI");

                info!("rssi: {}", rssi);

                radio.check_receive(true).expect("error checking receive");

                std::thread::sleep(*config.period);

                if !config.continuous {
                    break
                }
            }
        }
        //_ => warn!("unsuppored command: {:?}", opts.command),
    }

}