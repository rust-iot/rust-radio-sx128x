
#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::TermLogger;

extern crate structopt;
use structopt::StructOpt;

extern crate humantime;

extern crate linux_embedded_hal;
use linux_embedded_hal::{spidev, Spidev, Pin as PinDev, Delay};
use linux_embedded_hal::sysfs_gpio::Direction;

extern crate embedded_hal;
use embedded_hal::digital::v2::OutputPin;

extern crate radio;
use radio::{State as _};

extern crate radio_sx128x;
use radio_sx128x::prelude::*;

mod options;
use options::*;

mod operations;
use operations::*;

fn main() {
    // Load options
    let opts = Options::from_args();

    // Setup logging
    TermLogger::init(opts.level, simplelog::Config::default()).unwrap();

    debug!("Connecting to SPI device");

    // Connect to hardware
    let mut spi = Spidev::open(opts.spi).expect("error opening spi device");
    let mut config = spidev::SpidevOptions::new();
    config.mode(spidev::SpiModeFlags::SPI_MODE_0 | spidev::SpiModeFlags::SPI_NO_CS);
    config.max_speed_hz(opts.baud);
    spi.configure(&config).expect("error configuring spi device");

    debug!("Configuring I/O pins");

    let cs = PinDev::new(opts.cs);
    cs.export().expect("error exporting cs pin");
    cs.set_direction(Direction::Out).expect("error setting cs pin direction");

    let rst = PinDev::new(opts.rst);
    rst.export().expect("error exporting rst pin");
    rst.set_direction(Direction::Out).expect("error setting rst pin direction");

    // Configure (optional) antenna control output pin
    let mut ant = PinDev::new(opts.ant);
    ant.export().expect("error exporting rst ant");
    ant.set_direction(Direction::Out).expect("error setting ant pin direction");
    ant.set_high().expect("error setting ANT pin state");

    // Configure busy input pin
    let busy = PinDev::new(opts.busy);
    busy.export().expect("error exporting busy pin");
    busy.set_direction(Direction::Out).expect("error setting busy pin direction");

    debug!("Creating radio instance");

    let mut config = Config::default();

    // Generate configurations
    match &opts.command {
        Command::LoRa(lora_config) => {
            // Set to lora mode
            config.modem = Modem::LoRa(LoRaConfig::default());

            let mut channel = LoRaChannel::default();
            channel.freq = (lora_config.frequency * 1e9) as u32;

            config.channel = Channel::LoRa(channel);
        },
        Command::Flrc(flrc_config) => {
            // Set to Gfsk mode
            config.modem = Modem::Flrc(FlrcConfig::default());

            let mut channel = FlrcChannel::default();
            channel.freq = (flrc_config.frequency * 1e9) as u32;
            channel.br_bw = flrc_config.bitrate_bandwidth;

            config.channel = Channel::Flrc(channel);
        }
        Command::Gfsk(gfsk_config) => {
            // Set to Gfsk mode
            config.modem = Modem::Gfsk(GfskConfig::default());

            let mut channel = GfskChannel::default();
            channel.freq = (gfsk_config.frequency * 1e9) as u32;

            config.channel = Channel::Gfsk(channel);
        },
        _ => (),
    }

    debug!("Config: {:?}", config);

    info!("Initialising Radio");
    let mut radio = Sx128x::spi(spi, cs, busy, rst, Delay{}, &config).expect("error creating device");

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

    radio.set_state(State::Sleep).expect("Error setting sleep mode");
}

