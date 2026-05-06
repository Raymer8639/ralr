# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build --release          # Build all workspace crates
cargo fmt                      # Format
cargo clippy                   # Lint
./install.sh                   # Build release + cargo install --force both binaries to ~/.cargo/bin/
./uninstall.sh                 # Remove installed binaries (cargo uninstall)
cd tests && cargo test         # Run all tests (tests/ is a separate Cargo workspace)
cd tests && cargo test <name>  # Run a single test by name
```

## Architecture

This is a simple register-based VM with its own assembler. A Cargo workspace with three crates:

### `vm_isa` (shared library — `crates/vm-isa/`)
The instruction set architecture. Defines three core types that both the assembler and VM depend on:

- **`OpCode`** — Four arithmetic instructions: `Add`, `Sub`, `Mul`, `Div`, each taking `(Value, Value, Register)` for two operands and a destination register. Plus `Println(Value)` and `Print(Value)` for printing values to stdout.
- **`Value`** — A tagged union: `None`, `I32(i32)`, `I128(i128)`, `U8(u8)`, `U32(u32)`, `U128(u128)`, `F32(f32)`, `F64(f64)`, `Bool(bool)`, `String(String)`, `Register(Arc<Register>)`. Implements `Add`/`Sub`/`Mul`/`Div` via macros that generate the type-matching arms.
- **`Register`** — Five variants (`A1`–`A5`), each wrapping a `Value`. Used both at parse time (wrapping `Value::None` as placeholders) and at runtime (wrapping actual computed values).
- **`AllRegister`** — The runtime register file with fields `a1`–`a5`. Holds the "live" values that change during execution.

**Key design choice:** There are two different "register resolution" paths:
- At **parse time** (`ralr-asm`), registers are represented as `Value::Register(Arc::new(Register::A1(Value::None)))` — the `Value::None` is a placeholder.
- At **runtime** (`ralr`), the `AllRegister` struct holds real values. The `update_register!` macro (in `value.rs`) and `update_register_from_allreister!` macro (in `runner.rs`) resolve `Value::Register(...)` indirections by looking up the current value from `AllRegister` and substituting it before the arithmetic operation.

All crates use Rust edition 2024. Workspace dependencies: `clap` (CLI), `anyhow` (errors), `bincode` + `serde` (serialization), `unescape` (string escape processing in `ralr-asm`), `tokio` + `tracing` (async runtime + logging in `ralr`).

### `ralr-asm` (assembler — `bin/ralr-asm/`)
Reads `.ralr` source files (semicolon-delimited instructions like `add 1 2 $a1;`) and emits `.abin` binary files.

- `main.rs` — Parses CLI args (`-o` for output name, multiple input files supported), reads lines, passes each to `reader()`.
- `reader.rs` — Parses a single instruction line. `to_value()` converts tokens: `$a1`–`$a5` become register placeholders; string literals must be double-quoted (`"..."`) and are processed through the `unescape` crate for escape sequence handling (`\n`, `\t`, `\\`, `\"`, `\r`, `\0`, `\xNN`, `\u{NNNN}`); bare words `true`/`false` map to `Bool`; numbers are parsed via a `try_parse!` macro chain (`u8` → `u32` → `u128` → `i32` → `i128` → `f32` → `f64`). Unrecognized tokens error. Supports `add`, `sub`, `mul`, `div`, `_println`, and `_print` keywords. Unrecognized keywords are silently ignored via the `_ => ()` catch-all.

### `ralr` (VM runtime — `bin/ralr/`)
Reads `.abin` binary files, deserializes them, and executes instructions against an `AllRegister` state.

- `main.rs` — Async (tokio). Uses `tracing` for structured logging. Reads the file, bincode-deserializes to `Vec<OpCode>`, calls `runner()`.
- `runner.rs` — Synchronous execution loop. Uses three macros to reduce boilerplate across the five registers for each opcode. `update_register_from_allreister!` resolves register references from `AllRegister` for arithmetic ops; `op_assign!` performs the arithmetic and writes the result into the correct `AllRegister` field; `resolve_value!` resolves a `Value::Register` indirection by looking up the current value from `AllRegister` (used by `Println`/`Print`). `OpCode::Print` explicitly flushes stdout since `print!` does not.

## Tests

Tests live in `tests/` — a **separate Cargo workspace** (not a member of the root workspace). This lets them depend on `vm_isa` while avoiding circular dev-dependencies. Run with `cd tests && cargo test`. Covers: `Value` arithmetic (including register resolution and type-mismatch panics), `AllRegister` initialization, and `OpCode` bincode roundtrip serialization.

## Data flow

```
.ralr (text) ──[ralr-asm]──→ .abin (bincode) ──[ralr]──→ register state
```

## Documentation

Reference docs live in `docs/`: keyword reference, assembly language guide, and install guide.

## Examples

`examples/add.ralr` — Computes `((1 + 2) - 1) * 2 / 2` using register `$a1` for intermediate results.
`examples/print.ralr` — Demonstrates `_println` and `_print` with a register value, quoted string literals, integer literals, and `\n` escape sequences.
