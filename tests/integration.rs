//! Sx128x Integration testing
//!
//! Copyright 2019 Ryan Kurte

use std::io::Error as IoError;
use std::thread;
use std::time::Duration;

extern crate embedded_hal;
use embedded_hal::digital::v2::OutputPin;

extern crate embedded_spi;
use embedded_spi::wrapper::Wrapper;
use embedded_spi::utils::{PinError};

extern crate linux_embedded_hal;
use linux_embedded_hal::{spidev, Spidev, Pin as PinDev, Delay};
use linux_embedded_hal::sysfs_gpio::Direction;

extern crate radio_sx128x;
use radio_sx128x::prelude::*;

extern crate radio;
use radio::{Receive, Transmit};

#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::{SimpleLogger, LevelFilter};

pub type SpiWrapper = Wrapper<Spidev, IoError, PinDev, PinDev, (), PinDev, PinError, Delay>;

pub type Radio = Sx128x<SpiWrapper, IoError, PinError>;

pub struct HwConfig<'a> {
    spi: &'a str, 
    baud: u32, 
    cs: u64, 
    rst: u64, 
    busy: u64, 
    ant: u64,
}

const RADIO1_CONFIG: HwConfig = HwConfig {
    spi: "/dev/spidev0.0", baud: 1_000_000, cs: 16, rst: 17, busy: 5, ant: 23
};

const RADIO2_CONFIG: HwConfig = HwConfig {
    spi: "/dev/spidev0.1", baud: 1_000_000, cs: 13, rst: 18, busy: 8, ant: 22
};

fn load_radio(config: &Config, hw: &HwConfig) -> Radio
 {
    debug!("Connecting to radio");

    // Connect to hardware
    let mut spi = Spidev::open(hw.spi).expect("error opening spi device");
    let mut spi_config = spidev::SpidevOptions::new();
    spi_config.mode(spidev::SpiModeFlags::SPI_MODE_0 | spidev::SpiModeFlags::SPI_NO_CS);
    spi_config.max_speed_hz(hw.baud);
    spi.configure(&spi_config).expect("error configuring spi device");

    let cs = PinDev::new(hw.cs);
    cs.export().expect("error exporting cs pin");
    cs.set_direction(Direction::Out).expect("error setting cs pin direction");

    let rst = PinDev::new(hw.rst);
    rst.export().expect("error exporting rst pin");
    rst.set_direction(Direction::Out).expect("error setting rst pin direction");

    // Configure (optional) antenna control output pin
    let mut ant = PinDev::new(hw.ant);
    ant.export().expect("error exporting rst ant");
    ant.set_direction(Direction::Out).expect("error setting ant pin direction");
    ant.set_high().expect("error setting ANT pin state");

    // Configure busy input pin
    let busy = PinDev::new(hw.busy);
    busy.export().expect("error exporting busy pin");
    busy.set_direction(Direction::Out).expect("error setting busy pin direction");

    let hal: SpiWrapper = Wrapper::new(spi, cs, busy, (), rst, Delay{});

    let radio = Sx128x::new(hal, config).expect("error creating radio");

    debug!("Radio initialised");

    radio
}


fn test_tx_rx(radio1: &mut Radio, radio2: &mut Radio) {
    info!("Testing send/receive");

    let data = &[0x11, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00];

    // Poll on tx and rx complete
    let mut sent = false;
    let mut received = false;
    let mut buff = [0u8; 1024];
    let mut n = 0;
    let mut info = PacketInfo::default();


    // Configure receive
    radio1.start_receive().unwrap();

    thread::sleep(Duration::from_millis(500));

    // Start transmit
    radio2.start_transmit(data).unwrap();


    for i in 0..10 {
        // Check TX state
        if !sent && radio2.check_transmit().unwrap() {
            println!("TX complete ({})", i);
            sent = true;
        }

        // Check RX state
        if !received && radio1.check_receive(false).unwrap() {
            n = radio1.get_received(&mut info, &mut buff).unwrap();
            received = true;
            println!("RX complete ({:?} {:?}, {})", info, &buff[..n], i);
        }

        if sent && received {
            println!("Success!");
            break
        }

        thread::sleep(Duration::from_millis(50));
    }

    assert!(sent, "Send not completed");
    assert!(received, "Receive not completed");
    assert_eq!(data, &buff[..n]);
}

#[test]
#[ignore]
fn lora_tx_rx() {
    // Setup logging
    let _ = SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default());

    let mut config = Config::default();
    config.modem = Modem::LoRa(LoRaConfig::default());

    let channel = LoRaChannel::default();
    config.channel = Channel::LoRa(channel);

    info!("Loading radios");
    let mut radio1 = load_radio(&config, &RADIO1_CONFIG);
    let mut radio2 = load_radio(&config, &RADIO2_CONFIG);

    info!("Running test");
    test_tx_rx(&mut radio1, &mut radio2);
}


#[test]
#[ignore]
fn flrc_tx_rx() {
    // Setup logging
    let _ = SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default());

    let mut config = Config::default();
    config.modem = Modem::Flrc(FlrcConfig::default());

    let channel = FlrcChannel::default();
    config.channel = Channel::Flrc(channel);

    info!("Loading radios");
    let mut radio1 = load_radio(&config, &RADIO1_CONFIG);
    let mut radio2 = load_radio(&config, &RADIO2_CONFIG);

    info!("Running test");
    test_tx_rx(&mut radio1, &mut radio2);
}

#[test]
#[ignore]
fn gfsk_tx_rx() {
    // Setup logging
    let _ = SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default());

    let mut config = Config::default();
    config.modem = Modem::Gfsk(GfskConfig::default());

    let channel = GfskChannel::default();
    config.channel = Channel::Gfsk(channel);

    info!("Loading radios");
    let mut radio1 = load_radio(&config, &RADIO1_CONFIG);
    let mut radio2 = load_radio(&config, &RADIO2_CONFIG);

    info!("Running test");
    test_tx_rx(&mut radio1, &mut radio2);
}