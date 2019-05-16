# rust-radio-sx128x

A rust driver for the [Semtech SX128x](https://www.semtech.com/products/wireless-rf/24-ghz-transceivers/sx1280) 2.4GHz ISM band radio IC.

This currently uses [libsx128x](https://github.com/ryankurte/libsx128x) via FFI with the intent of slowly replacing the underlying c components with rust.


## Status

[![GitHub tag](https://img.shields.io/github/tag/ryankurte/rust-radio-sx128x.svg)](https://github.com/ryankurte/rust-radio-sx128x)
[![Build Status](https://travis-ci.com/ryankurte/rust-radio-sx128x.svg?branch=master)](https://travis-ci.com/ryankurte/rust-radio-sx128x)
[![Crates.io](https://img.shields.io/crates/v/radio-sx128x.svg)](https://crates.io/crates/radio-sx128x)
[![Docs.rs](https://docs.rs/radio-sx128x/badge.svg)](https://docs.rs/radio-sx128x)

[Open Issues](https://github.com/ryankurte/rust-radio-sx127x/issues)


## Useful Resources
- [Datasheet](https://www.semtech.com/uploads/documents/DS_SX1280-1_V2.2.pdf)
- [libsx128x](https://github.com/ryankurte/libsx128x) semtech c driver port


## Building

The build process will automatically clone `libsx128x` into the output directory, should an alternative directory be required (ie. for working on the c library) this can be set exporting the `LIBSX128X_DIR` environmental variable during the first cargo build.
