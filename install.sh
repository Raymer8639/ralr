#!/usr/bin/env bash
set -x

cargo build --release

cargo install --force --path ralr
cargo install --force --path ralr-asm
