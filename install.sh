#!/usr/bin/env bash
set -x

cargo build --release

cargo install --path bin/ralr
cargo install --path bin/ralr-asm
