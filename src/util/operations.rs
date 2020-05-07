

use std::time::{Duration, SystemTime};
use std::fs::File;

use embedded_hal::blocking::delay::DelayUs;

use embedded_spi::hal::{HalDelay};
use pcap_file::PcapWriter;

use super::options::*;

pub fn do_command<T, I, E>(radio: &mut T, operation: Operation) -> Result<(), E> 
where
    T: radio::Transmit<Error=E> + radio::Power<Error=E> + radio::Receive<Info=I, Error=E>  + radio::Rssi<Error=E> + radio::Power<Error=E>,
    I: Default + std::fmt::Debug,
    E: std::fmt::Debug,
{
    // TODO: the rest
    match operation {
        Operation::Transmit(config) => {
            do_transmit(radio, config.data.as_bytes(), config.power, config.continuous, *config.period, *config.poll_interval)
                .expect("Transmit error")
        },
        Operation::Receive(config) => {
            let mut buff = [0u8; 255];
            let mut info = I::default();

            do_receive(radio, &mut buff, &mut info, config.continuous, *config.poll_interval, config.pcap_file)
                .expect("Receive error");
        },
        Operation::Repeat(config) => {
            let mut buff = [0u8; 255];
            let mut info = I::default();

            do_repeat(radio, &mut buff, &mut info, config.power, config.continuous, *config.delay, *config.poll_interval)
                .expect("Repeat error");
        }
        Operation::Rssi(config) => {
            do_rssi(radio, config.continuous, *config.period)
                .expect("RSSI error");
        },
        //_ => warn!("unsuppored command: {:?}", opts.command),
    }

    Ok(())
}


fn do_transmit<T, E>(radio: &mut T, data: &[u8], power: Option<i8>, continuous: bool, period: Duration, poll_interval: Duration) -> Result<(), E> 
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
            HalDelay{}.delay_us(poll_interval.as_micros() as u32);
        }

        if !continuous {  break; }
        HalDelay{}.delay_us(poll_interval.as_micros() as u32);
    }

    Ok(())
}

fn do_receive<T, I, E>(radio: &mut T, mut buff: &mut [u8], mut info: &mut I, continuous: bool, poll_interval: Duration, pcap_file: Option<String>) -> Result<usize, E> 
where
    T: radio::Receive<Info=I, Error=E>,
    I: std::fmt::Debug,
{
    // Load PCAP file
    let mut pcap = match pcap_file {
        Some(n) => {
            let f = File::create(n).expect("Error creating PCAP file");
            let w = PcapWriter::new(f).expect("Error writing to PCAP file");
            Some(w)
        },
        None => None,
    };
    let t = SystemTime::now();

    // Start receive mode
    radio.start_receive()?;

    loop {
        if radio.check_receive(true)? {
            let n = radio.get_received(&mut info, &mut buff)?;

            match std::str::from_utf8(&buff[0..n as usize]) {
                Ok(s) => info!("Received: '{}' info: {:?}", s, info),
                Err(_) => info!("Received: '{:?}' info: {:?}", &buff[0..n as usize], info),
            }

            if let Some(p) = &mut pcap {
                let d = t.elapsed().unwrap();
                
                p.write(d.as_secs() as u32, d.as_nanos() as u32 % 1_000_000, &buff[0..n], n as u32).expect("Error writing pcap file");
            }
            
            if !continuous { 
                return Ok(n)
            }

            radio.start_receive()?;
        }

        HalDelay{}.delay_us(poll_interval.as_micros() as u32);
    }
}

fn do_rssi<T, I, E>(radio: &mut T, continuous: bool, period: Duration) -> Result<(), E> 
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

        HalDelay{}.delay_us(period.as_micros() as u32);

        if !continuous {
            break
        }
    }

    Ok(())
}

fn do_repeat<T, I, E>(radio: &mut T, mut buff: &mut [u8], mut info: &mut I, power: Option<i8>, continuous: bool, delay: Duration, poll_interval: Duration) -> Result<usize, E> 
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

            HalDelay{}.delay_us(delay.as_micros() as u32);

            radio.start_transmit(&buff[..n])?;
            loop {
                if radio.check_transmit()? {
                    debug!("Send complete");
                    break;
                }
                HalDelay{}.delay_us(poll_interval.as_micros() as u32);
            }
            
            if !continuous { return Ok(n) }
        }

        HalDelay{}.delay_us(poll_interval.as_micros() as u32);
    }
}
