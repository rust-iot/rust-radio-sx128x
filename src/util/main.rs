
use std::time::Duration;

#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::{TermLogger, LevelFilter};

extern crate structopt;
use structopt::StructOpt;

extern crate humantime;
use humantime::{Duration as HumanDuration};

extern crate linux_embedded_hal;
use linux_embedded_hal::{spidev, Spidev, Pin as PinDev, Delay};
use linux_embedded_hal::sysfs_gpio::Direction;

extern crate embedded_hal;
use embedded_hal::digital::v2::OutputPin;

extern crate radio;

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

    #[structopt(name="repeat")]
    /// Repeat received messages
    Repeat(Repeat),
}

#[derive(StructOpt, PartialEq, Debug)]
pub struct Transmit {
    /// Data to be transmitted
    #[structopt(long = "data")]
    data: String,

    /// Run continuously
    #[structopt(long = "continuous")]
    continuous: bool,

    /// Power in dBm (range -18dBm to 13dBm)
    #[structopt(long = "power")]
    power: Option<i8>,

    /// Specify period for transmission
    #[structopt(long = "period", default_value="1s")]
    pub period: HumanDuration,

    /// Specify period for polling for device status
    #[structopt(long = "poll-interval", default_value="10ms")]
    poll_interval: HumanDuration,
}

#[derive(StructOpt, PartialEq, Debug)]
pub struct Receive {
    /// Run continuously
    #[structopt(long = "continuous")]
    continuous: bool,

    /// Specify period for polling for device status
    #[structopt(long = "poll-interval", default_value="10ms")]
    poll_interval: HumanDuration,
}

#[derive(StructOpt, PartialEq, Debug)]
pub struct Rssi {
    /// Specify period for RSSI polling
    #[structopt(long = "period", default_value="1s")]
    pub period: HumanDuration,

    /// Run continuously
    #[structopt(long = "continuous")]
    continuous: bool,
}

#[derive(StructOpt, PartialEq, Debug)]
pub struct Repeat {
    /// Run continuously
    #[structopt(long = "continuous")]
    continuous: bool,
    
    /// Power in dBm (range -18dBm to 13dBm)
    #[structopt(long = "power")]
    power: Option<i8>,

    /// Specify period for polling for device status
    #[structopt(long = "poll-interval", default_value="1ms")]
    poll_interval: HumanDuration,

    /// Specify delay for response message
    #[structopt(long = "delay", default_value="100ms")]
    delay: HumanDuration,

    /// Append RSSI and LQI to repeated message
    #[structopt(long = "append-info")]
    append_info: bool,
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
    config.mode(spidev::SPI_MODE_0 | spidev::SPI_NO_CS);
    config.max_speed_hz(opts.baud);
    spi.configure(&config).expect("error configuring spi device");

    debug!("Configuring I/O pins");

    let cs = PinDev::new(opts.cs);
    cs.export().expect("error exporting cs pin");
    cs.set_direction(Direction::Out).expect("error setting cs pin direction");

    let rst = PinDev::new(opts.rst);
    rst.export().expect("error exporting rst pin");
    rst.set_direction(Direction::Out).expect("error setting rst pin direction");

    let mut ant = PinDev::new(opts.ant);
    ant.export().expect("error exporting rst ant");
    ant.set_direction(Direction::Out).expect("error setting ant pin direction");
    ant.set_high().expect("error setting ANT pin state");

    // TODO: set ant output

    let busy = PinDev::new(opts.busy);
    busy.export().expect("error exporting busy pin");
    busy.set_direction(Direction::Out).expect("error setting busy pin direction");

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
            do_transmit(radio, config.data.as_bytes(), config.power, config.continuous, *config.period, *config.poll_interval)
                .expect("Transmit error")
        },
        Command::Receive(config) => {
            let mut buff = [0u8; 255];
            let mut info = Info::default();

            do_receive(radio, &mut buff, &mut info, config.continuous, *config.poll_interval)
                .expect("Receive error");
        },
        Command::Repeat(config) => {
            let mut buff = [0u8; 255];
            let mut info = Info::default();

            do_repeat(radio, &mut buff, &mut info, config.power, config.continuous, *config.delay, *config.poll_interval)
                .expect("Repeat error");
        }
        Command::Rssi(config) => {
            do_rssi(radio, config.continuous, *config.period)
                .expect("RSSI error");
        }
        //_ => warn!("unsuppored command: {:?}", opts.command),
    }
}


fn do_transmit<T, E>(mut radio: T, data: &[u8], power: Option<i8>, continuous: bool, period: Duration, poll_interval: Duration) -> Result<(), E> 
where
    T: radio::Transmit<Error=E> + radio::Power<Error=E>
{
    // Set output power if specified
    if let Some(p) = power {
        radio.set_power(p)?;
    }

    loop {
        radio.start_transmit(data)?;
        loop {
            if radio.check_transmit()? {
                debug!("Send complete");
                break;
            }
            std::thread::sleep(poll_interval);
        }

        if !continuous {  break; }
        std::thread::sleep(period);
    }

    Ok(())
}

fn do_receive<T, I, E>(mut radio: T, mut buff: &mut [u8], mut info: &mut I, continuous: bool, poll_interval: Duration) -> Result<usize, E> 
where
    T: radio::Receive<Info=I, Error=E>,
    I: std::fmt::Debug,
{
    // Start receive mode
    radio.start_receive()?;

    loop {
        if radio.check_receive(true)? {
            let n = radio.get_received(&mut info, &mut buff)?;

            match std::str::from_utf8(&buff[0..n as usize]) {
                Ok(s) => info!("Received: '{}' info: {:?}", s, info),
                Err(_) => info!("Received: '{:?}' info: {:?}", &buff[0..n as usize], info),
            }
            
            if !continuous { return Ok(n) }
        }

        std::thread::sleep(poll_interval);
    }
}

fn do_rssi<T, I, E>(mut radio: T, continuous: bool, period: Duration) -> Result<(), E> 
where
    T: radio::Receive<Info=I, Error=E> + radio::Rssi<Error=E>,
    I: std::fmt::Debug,
{
    // Enter receive mode
    radio.start_receive()?;

    // Poll for RSSI
    loop {
        let rssi = radio.poll_rssi()?;

        info!("rssi: {}", rssi);

        radio.check_receive(true)?;

        std::thread::sleep(period);

        if !continuous {
            break
        }
    }

    Ok(())
}

fn do_repeat<T, I, E>(mut radio: T, mut buff: &mut [u8], mut info: &mut I, power: Option<i8>, continuous: bool, delay: Duration, poll_interval: Duration) -> Result<usize, E> 
where
    T: radio::Receive<Info=I, Error=E> + radio::Transmit<Error=E> + radio::Power<Error=E>,
    I: std::fmt::Debug,
{
     // Set output power if specified
    if let Some(p) = power {
        radio.set_power(p)?;
    }

    // Start receive mode
    radio.start_receive()?;

    loop {
        if radio.check_receive(true)? {
            let n = radio.get_received(&mut info, &mut buff)?;

            match std::str::from_utf8(&buff[0..n as usize]) {
                Ok(s) => info!("Received: '{}' info: {:?}", s, info),
                Err(_) => info!("Received: '{:?}' info: {:?}", &buff[0..n as usize], info),
            }

            std::thread::sleep(delay);

            radio.start_transmit(&buff[..n])?;
            loop {
                if radio.check_transmit()? {
                    debug!("Send complete");
                    break;
                }
                std::thread::sleep(poll_interval);
            }
            
            if !continuous { return Ok(n) }
        }

        std::thread::sleep(poll_interval);
    }
}
