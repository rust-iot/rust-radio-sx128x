# rust-radio-sx128x

A rust driver (and command line utility) for the [Semtech SX128x](https://www.semtech.com/products/wireless-rf/24-ghz-transceivers/sx1280) 2.4GHz ISM band radio IC.


## Status

WIP. Basic LoRa functionality working.

[![GitHub tag](https://img.shields.io/github/tag/rust-iot/rust-radio-sx128x.svg)](https://github.com/rust-iot/rust-radio-sx128x)
![Build Status](https://github.com/rust-iot/rust-radio-sx128x/workflows/Rust/badge.svg)
[![Integration Test Status](https://badge.buildkite.com/a8caa71f875a6ec62091a5dda4dbf7dc0e35eb4e02c8d0933b.svg)](https://buildkite.com/rust-iot/rust-radio-sx128x)
[![Crates.io](https://img.shields.io/crates/v/radio-sx128x.svg)](https://crates.io/crates/radio-sx128x)
[![Docs.rs](https://docs.rs/radio-sx128x/badge.svg)](https://docs.rs/radio-sx128x)
[![Snap Status](https://build.snapcraft.io/badge/rust-iot/rust-radio-sx128x.svg)](https://build.snapcraft.io/user/rust-iot/rust-radio-sx128x)

[Open Issues](https://github.com/rust-iot/rust-radio-sx128x/issues)

## Usage

Add to your project with `cargo add radio-sx128x`

Install the utility via one of the following methods:

- `cargo install radio-sx128x` to install from source
- `cargo binstall radio-sx128x` to install a pre-compiled binary via [cargo-binstall](https://github.com/ryankurte/cargo-binstall)
- Manually fetch the latest [release](https://github.com/rust-iot/rust-radio-sx128x/releases/)

## Useful Resources
- [Sx128x Datasheet](https://www.semtech.com/uploads/documents/DS_SX1280-1_V2.2.pdf)
- [libsx128x](https://github.com/ryankurte/libsx128x) semtech c driver port


