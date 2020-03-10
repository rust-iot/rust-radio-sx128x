#!/bin/bash

RUSTC_WRAPPER="sccache"

if [ -z "$SCCACHE_DIR"]; then
    export SCCACHE_DIR=$HOME/.cache
fi

if [ -z "$CARGO_HOME"]; then
    export CARGO_HOME=$HOME/.cargo
fi


DOCKER_CMD="docker run --rm -v=`pwd`:/work -v$CARGO_HOME/registry:/root/.cargo/registry -v$SCCACHE_DIR:/root/.cache --workdir=/work --user=:`id -g` --env PKG_CONFIG_ALLOW_CROSS=1"

DOCKER_IMG=ryankurte/rust-embedded



echo "Building for target: $1 (cargo cache: $CARGO_HOME, sccache $SCCACHE_DIR)"

# Args for cross build for armhf, windows tricks are in .cargo/config
ARMHF_ARGS="--env SYSROOT=/usr/arm-linux-gnueabihf --env PKG_CONFIG_ALLOW_CROSS=1 --env PKG_CONFIG_LIBDIR=/usr/lib/arm-linux-gnueabihf/pkgconfig --env PKG_CONFIG_SYSROOT_DIR=/usr/arm-linux-gnueabihf --env PKG_CONFIG_SYSTEM_LIBRARY_PATH=/usr/lib/arm-linux-gnueabihf --env PKG_CONFIG_SYSTEM_INCLUDE_PATH=/usr/arm-linux-gnueabihf/include"


if [ "$1" == "x86_64-unknown-linux-gnu" ]; then
    $DOCKER_CMD $DOCKER_IMG /bin/bash -c "cargo build --target=$1 --release --features embedded-spi/hal-linux"

elif [ "$1" == "armv7-unknown-linux-gnueabihf" ]; then
    $DOCKER_CMD $ARMHF_ARGS $DOCKER_IMG /bin/bash -c "cargo build --target=$1 --release --features embedded-spi/hal-linux"

elif [ "$1" == "x86_64-apple-darwin" ]; then
    cargo build --target=$1 --release

elif [ "$1" == "i686-pc-windows-gnu" ]; then
    $DOCKER_CMD $DOCKER_IMG /bin/bash -c "cargo build --target=$1 --release"

else

    echo "Unsupported target: $1"
    exit 1

fi
