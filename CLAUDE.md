# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build                    # Dev build all workspace crates
cargo build --release          # Release build
cargo fmt                      # Format
cargo clippy                   # Lint
./install.sh                   # Build release + cargo install --force both binaries to ~/.cargo/bin/
./uninstall.sh                 # Remove installed binaries (cargo uninstall)
cd tests && cargo test         # Run all tests (tests/ is a separate Cargo workspace)
cd tests && cargo test <name>  # Run a single test by name
RUST_LOG=trace ralr <file.abin> # Run VM with full tracing output
```

### End-to-end pipeline

```bash
ralr-asm examples/add.ralr -o add.abin && ralr add.abin
```

`.abin` files are generated artifacts — do not commit ad-hoc outputs. The committed `examples/add.abin` and `examples/print.abin` are test fixtures (the `tests/` workspace deserializes `add.abin` to validate binary format stability).

## Architecture

This is a simple register-based VM with its own assembler. A Cargo workspace with three crates:

### `vm_isa` (shared library — `crates/vm-isa/`)
The instruction set architecture. Defines the core types that both the assembler and VM depend on:

- **`OpCode`** — Four arithmetic instructions: `Add`, `Sub`, `Mul`, `Div`, each taking `(Operand, Operand, Register)` for two source operands and a destination register. Plus `Println(Operand)` and `Print(Operand)` for printing values to stdout.
- **`Operand`** — `Literal(Value)` or `Register(Register)`. Separates immediate values from register references at the type level, eliminating the need for `Arc` indirection.
- **`Value`** — A tagged union: `None`, `I32(i32)`, `I128(i128)`, `U8(u8)`, `U32(u32)`, `U128(u128)`, `F32(f32)`, `F64(f64)`, `Bool(bool)`, `String(String)`. Implements `Add`/`Sub`/`Mul`/`Div` via the `op!` macro that generates type-matching arms. No longer carries register references — those live in `Operand`.
- **`Register`** — A `Copy` enum: `A1`–`A5` (discriminants only, no wrapped data). `index()` maps each to `0..4` for array access.
- **`Registers`** — Runtime register file backed by `[Value; 5]`. `read(Register) -> &Value` and `write(Register, Value)` provide O(1) array-indexed access. Replaces the old `AllRegister` with named fields.

**Key design:** Register resolution happens at the `OpCode` level via `Operand`, not inside `Value`. The VM's `resolve()` function dispatches `Operand::Literal(v)` → clone or `Operand::Register(r)` → array lookup. This eliminates the `Arc<Register>` heap allocation and nested match that existed in the old `Value::Register(Arc<Register>)` design.

**Error behavior:** `Value` arithmetic operators **panic** on type-mismatched operands (e.g., `I32(1) + String("x")` panics) — there is no graceful error propagation at runtime.

All crates use Rust edition 2024. Workspace dependencies: `clap` (CLI), `anyhow` (errors), `bincode` + `serde` (serialization), `tokio` + `tracing` (async runtime + logging in `ralr`). String escape processing is handled by `unescape_str()` in `reader.rs` (no external crate needed).

### `ralr-asm` (assembler — `bin/ralr-asm/`)
Reads `.ralr` source files and emits `.abin` binary files (default output: `output.abin`).

**`.ralr` instruction format:**
```
arithmetic:  keyword operand1 operand2 $dest_register   ; e.g. add 1 2 $a1
print:       _println|_print value                       ; e.g. _println "hello"
```

Each instruction must be terminated with `;`. Lines can contain multiple `;`-delimited instructions. The trailing empty segment after the final `;` is discarded. Operands for arithmetic instructions are `(Value, Value, Register)` — the destination must be a register (`$a1`–`$a5`).

- `main.rs` — Parses CLI args (`-o` for output name, multiple input files supported), reads lines, passes each to `reader()`.
- `reader.rs` — Parses instruction tokens. `to_value()` converts literal tokens only (no register prefix): string literals must be double-quoted (`"..."`) and are processed through `unescape_str()` for escape sequence handling (`\n`, `\t`, `\\`, `\"`, `\r`, `\0`, `\xNN`, `\u{NNNN}`); bare words `true`/`false` map to `Bool`; numbers are parsed via a `try_parse!` macro chain (`u8` → `u32` → `u128` → `i32` → `i128` → `f32` → `f64`). `to_register()` handles `$a1`–`$a5` → `Register::A1`–`A5`. `to_operand()` dispatches: `$`-prefixed tokens go to `Operand::Register`, everything else to `Operand::Literal`. `tokenize()` splits instructions on whitespace while respecting quoted strings as single tokens. Unrecognized tokens error. Supports `add`, `sub`, `mul`, `div`, `_println`, and `_print` keywords. Unrecognized keywords are silently ignored via the `_ => ()` catch-all.

### `ralr` (VM runtime — `bin/ralr/`)
Reads `.abin` binary files, deserializes them, and executes instructions against a `Registers` state.

- `main.rs` — Async (tokio). Uses `tracing` for structured logging. Reads the file, bincode-deserializes to `Vec<OpCode>`, calls `runner()`.
- `runner.rs` — Synchronous execution loop. `resolve()` dispatches `Operand::Literal` (clone) vs `Operand::Register` (array index into `Registers`). Arithmetic ops resolve both operands, perform the operation, and write the result into the destination register. `Println`/`Print` resolve and display. `OpCode::Print` explicitly flushes stdout since `print!` does not.

## Tests

Tests live in `tests/` — a **separate Cargo workspace** (not a member of the root workspace). This lets them depend on `vm_isa` while avoiding circular dev-dependencies. Run with `cd tests && cargo test`. Covers: `Value` arithmetic (including type-mismatch panics), `Registers` initialization and read/write, `Register` index stability, and `OpCode` bincode roundtrip serialization (including deserializing a real `examples/add.abin`).

**CI note:** The GitHub Actions workflow (`.github/workflows/rust.yml`) runs `cargo test` from the root workspace, which does **not** execute the test suite in `tests/`. Tests must be run manually with `cd tests && cargo test`.

## Data flow

```
.ralr (text) ──[ralr-asm]──→ .abin (bincode) ──[ralr]──→ register state
```

## Documentation

- `docs/keywords.md` — Full keyword reference, value types, escape sequences
- `docs/assembly-guide.md` — Assembly language guide with examples
- `docs/install.md` — Install and development setup guide
- `docs/INDEX.md` — Docs index

## Examples

`examples/add.ralr` — Computes `((1 + 2) - 1) * 2 / 2` using register `$a1` for intermediate results.
`examples/print.ralr` — Demonstrates `_println` and `_print` with a register value, quoted string literals, integer literals, and `\n` escape sequences.
