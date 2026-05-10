# Installation Guide

## Prerequisites

- Rust toolchain (recommended: install via [rustup](https://rustup.rs/))
- Git

## Build and Install

```bash
# Clone the repository
git clone https://github.com/Raymer8639/ralr.git
cd ralr

# Build and install (requires network for dependency download)
./install.sh
```

`install.sh` performs the following steps:
1. `cargo build --release` — Builds all workspace crates in release mode
2. `cargo install --force --path bin/ralr` — Installs `ralr` (VM runtime) to `~/.cargo/bin/` (`--force` overwrites any existing installation)
3. `cargo install --force --path bin/ralr-asm` — Installs `ralr-asm` (assembler) to `~/.cargo/bin/` (`--force` overwrites any existing installation)

After installation, the `ralr` and `ralr-asm` commands are available in your terminal.

## Build Only (Without Installing)

```bash
cargo build --release
```

Build artifacts are located at `target/release/ralr` and `target/release/ralr_asm`.

## Uninstall

```bash
./uninstall.sh
```

Or manually:

```bash
cargo uninstall ralr
cargo uninstall ralr-asm
```

## Development Commands

```bash
cargo build                  # Debug build
cargo clippy                 # Run linter
cargo fmt                    # Format code
cd tests && cargo test       # Run tests (tests/ is a separate workspace)
cargo build -p vm_isa        # Build only the shared library
cargo build -p ralr          # Build only the VM runtime
cargo build -p ralr_asm      # Build only the assembler
```
