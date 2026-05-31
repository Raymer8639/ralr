# Ralr

&emsp; [![CI](https://github.com/Raymer8639/ralr/actions/workflows/rust.yml/badge.svg)](https://github.com/Raymer8639/ralr/actions/workflows/rust.yml) [![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

> **R**aymer's **A**ssembler and **L**oader — a simple register-based VM with its own assembler, written in Rust.

---

## Overview

ralr is a lightweight virtual machine with a custom register-based instruction set. It comes with:

| Component  | Description                                             |
| ---------- | ------------------------------------------------------- |
| `ralr-asm` | Assembler — compiles `.ralr` source to `.abin` bytecode |
| `ralr`     | VM runtime — loads and executes `.abin` files           |
| `vm-isa`   | Shared library — instruction set architecture types     |

The VM supports variables, control flow (`if`/`else`/`while`), functions with parameters and return values, I/O operations, and a full expression system with operator precedence.

---

## Installation

### From source

```bash
git clone https://github.com/Raymer8639/ralr.git
cd ralr
./install.sh
```

This builds the release binaries and installs them to `~/.cargo/bin/`. Requires [Rust](https://rustup.rs/) 1.85+ (edition 2024).

### Manual build

```bash
cargo build --release
# Binaries are at target/release/ralr and target/release/ralr-asm
```

### Uninstall

```bash
./uninstall.sh
```

---

## Quick start

```bash
# Assemble a source file
ralr-asm examples/add.ralr -o add.abin

# Run the bytecode
ralr add.abin
```

---

## Language tour

### Expression assignment

```ralr
$a1 = (1 + 2) * 3;          // arithmetic with precedence
$a2 = $a1 < 10;             // comparison → Bool
$a3 = true && false;        // logical
$a4 = 255 & 15;             // bitwise
$a5 = -5;                   // unary negation
```

### I/O

```ralr
io writeln "Hello, world!";
io write "Enter a number: ";
io read $a1;
io writeln $a1;
```

### Control flow

```ralr
if $a1 > 0 {
    io writeln "positive";
} else if $a1 == 0 {
    io writeln "zero";
} else {
    io writeln "negative";
}

while $a1 < 10 {
    $a1 = $a1 + 1;
}
```

### Variables

```ralr
let x = 42;
let mut y = 10;
y = y + 1;
io writeln y;
```

### Functions

```ralr
fn add(a, b) {
    return a + b;
}

$a1 = call add(3, 5);
io writeln $a1;             // 8
```

---

## Architecture

```
.ralr (text) ──[ralr-asm]──→ .abin (bincode) ──[ralr]──→ register state
```

- **5 general-purpose registers** (`$a1`–`$a5`) + 1 internal buffer
- **Variable store** — hashmap-backed, mutable/immutable
- **Function table** — definitions registered at runtime, replayed on each call
- **Binary format** — `bincode` serialization of the `OpCode` AST

See [CLAUDE.md](CLAUDE.md) for the full architecture reference.

---

## Examples

| File                                    | Demonstrates                                    |
| --------------------------------------- | ----------------------------------------------- |
| [`add.ralr`](examples/add.ralr)         | Arithmetic with register chaining               |
| [`print.ralr`](examples/print.ralr)     | Output and escape sequences                     |
| [`block.ralr`](examples/block.ralr)     | Nested `{ }` code blocks                        |
| [`expr.ralr`](examples/expr.ralr)       | Expression system with full operator precedence |
| [`if_else.ralr`](examples/if_else.ralr) | Conditional branching                           |
| [`while.ralr`](examples/while.ralr)     | While loops                                     |
| [`io.ralr`](examples/io.ralr)           | I/O operations                                  |
| [`var.ralr`](examples/var.ralr)         | Variables and mutation                          |
| [`fn.ralr`](examples/fn.ralr)           | Functions, parameters, closures                 |

---

## Documentation

Multi-language docs under [`docs/`](docs/):

| Language | Directory              |
| -------- | ---------------------- |
| 中文     | [`docs/zh/`](docs/zh/) |
| English  | [`docs/en/`](docs/en/) |

Start at [`docs/INDEX.md`](docs/INDEX.md) for the language selector.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines. Pull requests are welcome.

---

## License

[Apache 2.0](LICENSE)
