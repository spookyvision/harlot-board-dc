#!/bin/bash
set -e

#rm -rf target/release/.fingerprint/harlot-board-*; rm -rf target/release/build/harlot-board-40c2b4c9ad3b58a3;

pushd ../color-mixer-ws/mixer-dioxus/
#HARLOT_BOARD="http://harharlot/" trunk build
HARLOT_BOARD="http://192.168.71.1/" trunk build
cp public/style.css dist
popd
#cargo espflash --release --monitor --speed 800000
cargo espflash --monitor --release --speed 800000 /dev/cu.SLAB_USBtoUART
