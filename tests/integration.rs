//! Sx128x Integration testing
//!
//! Copyright 2019 Ryan Kurte

use std::thread;
use std::time::Duration;

use log::{debug, info};

use driver_pal::hal::*;
use driver_pal::wrapper::Wrapper;

use radio::{Receive, Transmit};
use radio_sx128x::prelude::*;

pub type SpiWrapper =
    Wrapper<HalSpi, HalOutputPin, HalInputPin, HalInputPin, HalOutputPin, HalDelay>;

pub type Radio = Sx128x<SpiWrapper, HalError, HalError, HalError>;

#[derive(Debug, serde::Deserialize)]
pub struct TestConfig {
    radio1: DeviceConfig,
    radio2: DeviceConfig,
}

fn load_radio(rf_config: &Config, device_config: &DeviceConfig) -> Radio {
    debug!("Connecting to radio");

    let HalInst { base: _, spi, pins } =
        HalInst::load(&device_config).expect("error connecting to HAL");

    let radio = Sx128x::spi(
        spi,
        pins.cs,
        pins.busy,
        pins.ready,
        pins.reset,
        HalDelay {},
        rf_config,
    )
    .expect("error creating device");

    debug!("Radio initialised");

    radio
}

fn load_radios(rf_config: &Config) -> (Radio, Radio) {
    let config_file = std::env::var("TEST_CONFIG").unwrap_or("config.toml".to_string());
    let config_data = std::fs::read_to_string(config_file).expect("Error reading test config");
    let hw_config: TestConfig = toml::from_str(&config_data).expect("Error parsing test config");

    let r1 = load_radio(rf_config, &hw_config.radio1);
    let r2 = load_radio(rf_config, &hw_config.radio2);

    (r1, r2)
}

fn test_tx_rx(radio1: &mut Radio, radio2: &mut Radio) {
    info!("Testing send/receive");

    let data = &[0x11, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00];

    // Poll on tx and rx complete
    let mut sent = false;
    let mut received = false;
    let mut buff = [0u8; 1024];
    let mut n = 0;

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
            let (n, info) = radio1.get_received(&mut buff).unwrap();
            received = true;
            println!("RX complete ({:?} {:?}, {})", info, &buff[..n], i);
        }

        if sent && received {
            println!("Success!");
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }

    assert!(sent, "Send not completed");
    assert!(received, "Receive not completed");
    assert_eq!(data, &buff[..n]);
}

fn log_init() {
    // TODO: re-enable env logging
    //let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
#[ignore]
fn lora_tx_rx() {
    log_init();

    let mut config = Config::default();
    config.modem = Modem::LoRa(LoRaConfig::default());

    let channel = LoRaChannel::default();
    config.channel = Channel::LoRa(channel);

    info!("Loading radios");

    let (mut radio1, mut radio2) = load_radios(&config);

    info!("Running test");
    test_tx_rx(&mut radio1, &mut radio2);
}

#[test]
#[ignore]
fn flrc_tx_rx() {
    log_init();

    let mut config = Config::default();
    config.modem = Modem::Flrc(FlrcConfig::default());

    let channel = FlrcChannel::default();
    config.channel = Channel::Flrc(channel);

    info!("Loading radios");
    let (mut radio1, mut radio2) = load_radios(&config);

    info!("Running test");
    test_tx_rx(&mut radio1, &mut radio2);
}

#[test]
#[ignore]
fn gfsk_tx_rx() {
    log_init();

    let mut config = Config::default();
    config.modem = Modem::Gfsk(GfskConfig::default());

    let channel = GfskChannel::default();
    config.channel = Channel::Gfsk(channel);

    info!("Loading radios");
    let (mut radio1, mut radio2) = load_radios(&config);

    info!("Running test");
    test_tx_rx(&mut radio1, &mut radio2);
}
