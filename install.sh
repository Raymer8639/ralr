#!/usr/bin/env bash
set -x

cargo build --release

cargo install --force --path bin/ralr
cargo install --force --path bin/ralr-asm
