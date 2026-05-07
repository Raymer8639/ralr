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

- **`OpCode`** — Arithmetic instructions: `Add`, `Sub`, `Mul`, `Div` each taking `(Operand, Operand, Register)` for two source operands and a destination register. I/O: `Println(Operand)` and `Print(Operand)` for printing values to stdout. Control flow: `Block(Vec<OpCode>)` for nested instruction sequences, `Expr(Expr, Register)` for expression tree evaluation, and `If(Expr, Vec<OpCode>, Option<Vec<OpCode>>)` for conditional branching (condition expression, then-body, optional else-body).
- **`Expr`** (`crates/vm-isa/src/expr.rs`) — Expression AST with `Literal(Value)`, `Register(Register)`, `Binary(Box<Expr>, BinOp, Box<Expr>)`, and `Unary(UnOp, Box<Expr>)` nodes.
- **`BinOp`** — Binary operators: `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge`, `And`, `Or`, `BitAnd`, `BitOr`, `BitXor`, `Shl`, `Shr`.
- **`UnOp`** — Unary operators: `Neg`, `Not`, `BitNot`.
- **`Operand`** — `Literal(Value)` or `Register(Register)`. Separates immediate values from register references at the type level, eliminating the need for `Arc` indirection.
- **`Value`** — A tagged union: `None`, `I32(i32)`, `I128(i128)`, `U8(u8)`, `U32(u32)`, `U128(u128)`, `F32(f32)`, `F64(f64)`, `Bool(bool)`, `String(String)`. Implements `Add`/`Sub`/`Mul`/`Div` via the `op!` macro that generates type-matching arms. No longer carries register references — those live in `Operand`.
- **`Register`** — A `Copy` enum: `A1`–`A5` (discriminants only, no wrapped data). `index()` maps each to `0..4` for array access.
- **`Registers`** — Runtime register file backed by `[Value; 5]`. `read(Register) -> &Value` and `write(Register, Value)` provide O(1) array-indexed access. Replaces the old `AllRegister` with named fields.

**Key design:** Register resolution happens at the `OpCode` level via `Operand`, not inside `Value`. The VM's `resolve()` function dispatches `Operand::Literal(v)` → clone or `Operand::Register(r)` → array lookup. This eliminates the `Arc<Register>` heap allocation and nested match that existed in the old `Value::Register(Arc<Register>)` design.

**Error behavior:** `Value` arithmetic operators **panic** on type-mismatched operands (e.g., `I32(1) + String("x")` panics) — there is no graceful error propagation at runtime.

All crates use Rust edition 2024. Workspace dependencies: `clap` (CLI), `anyhow` (errors), `bincode` + `serde` (serialization), `tracing` + `tracing-subscriber` (structured logging in `ralr`). String escape processing is handled by `unescape_str()` in `reader.rs` (no external crate needed).

### `ralr-asm` (assembler — `bin/ralr-asm/`)
Reads `.ralr` source files and emits `.abin` binary files (default output: `output.abin`).

**`.ralr` instruction format:**
```
; Two syntax styles — expression assignment (preferred) and keyword (legacy):
$reg = expression;                 ; e.g. $a1 = (1 + 2) * 3;
keyword operand1 operand2 $dest;   ; e.g. add 1 2 $a1
_println|_print value;             ; e.g. _println "hello"
{ instruction; ... }               ; code block (nests, shares register context)
if cond { ... } [else { ... }]     ; conditional branch (else if desugars to nested If)
```

Each instruction must be terminated with `;`.

### Expression syntax (preferred)

Expressions use `$reg = expr;` form with full operator precedence (Pratt parser in `expr_parser.rs`):

```
$a1 = 10 + 5;            // arithmetic
$a1 = ($a2 - 3) * 2;     // parentheses override precedence
$a1 = 5 < 10;            // comparison → Bool
$a1 = true && false;     // logical
$a1 = 255 & 15;          // bitwise
$a1 = 1 << 4;            // shift
$a1 = -5;                // unary (constant-folded: -5 → I32(-5))
$a1 = !true;             // logical not
$a1 = ~0;                // bitwise not
```

Precedence (low→high): `||` → `&&` → `|` → `^` → `&` → `==` `!=` → `<` `>` `<=` `>=` → `<<` `>>` → `+` `-` → `*` `/` `%` → unary `-` `!` `~`

Operators and operands must be whitespace-separated: `1 + 2` not `1+2`.

### Code blocks

`{ ... }` groups instructions into a nested `OpCode::Block`. Blocks share the enclosing register context and support arbitrary nesting depth.

### Control flow (`if` / `else if` / `else`)

Conditional branching with expression conditions:

```
if $a1 > 0 {
    _println "positive";
}

if $a1 > 100 {
    _println "large";
} else if $a1 == 42 {
    _println "the answer";
} else {
    _println "something else";
}
```

- Condition must evaluate to `Bool` (panics otherwise)
- `else if` desugars to a nested `OpCode::If` in the else-body — no separate variant needed
- `if` statements don't require a trailing `;`
- Blocks share register context with enclosing scope

## Comments

- `//` — line comment, everything from `//` to end of line is ignored
- `/* … */` — block comment, everything between `/*` and `*/` is ignored (multi-line)
- Comment delimiters inside `"..."` strings are treated as literal text, not comments. Lines can contain multiple `;`-delimited instructions. The trailing empty segment after the final `;` is discarded. Operands for arithmetic instructions are `(Value, Value, Register)` — the destination must be a register (`$a1`–`$a5`).

- `main.rs` — Parses CLI args (`-o` for output name, multiple input files supported), reads each file fully, passes content to `reader::parse()`.
- `reader.rs` — `parse()` is the public entry point. It delegates to `parse_block()`, a recursive-descent parser that handles `{ }` blocks, `;`-delimited instructions, `//` and `/* */` comments, and string literals (including boundary characters inside strings). `parse_instruction()` dispatches individual instructions by keyword. Support functions: `to_value()` converts literal tokens (string literals are processed through `unescape_str()` for escape sequences `\n`, `\t`, `\r`, `\\`, `\"`, `\'`, `\a`, `\b`, `\f`, `\v`, `\e`, `\0`, `\xNN`, `\u{NNNN}`; bare words `true`/`false` map to `Bool`; numbers parse via `try_parse!` chain: `u8` → `u32` → `u128` → `i32` → `i128` → `f32` → `f64`). `to_register()` handles `$a1`–`$a5` → `Register::A1`–`A5`. `to_operand()` dispatches: `$`-prefixed tokens → `Operand::Register`, everything else → `Operand::Literal`. `tokenize()` splits on whitespace while keeping quoted strings intact. Unrecognized keywords are silently ignored. Expression assignments (`$a1 = ...`) are detected by matching `$`-prefixed first token with `=` as second token, then delegating to `parse_expr_str()`. The `if` keyword is intercepted at the `parse_block` level: after accumulating instruction text, if it starts with `if`, the condition is parsed via `parse_expr_str()`, then the then-block is consumed, and `parse_else_tail()` handles `else { }` / `else if ...` chains (recursive, with `else if` desugaring to a nested `OpCode::If` in the else-body).
- `expr_parser.rs` — Pratt precedence-climbing expression parser. `parse_expr_str()` tokenizes the expression string (with operator-aware splitting) then calls `parse_expression()`. Constant-folds unary `-` on unsigned literals (e.g., `-128` → `I32(-128)` instead of `Unary(Neg, U8(128))`).

### `ralr` (VM runtime — `bin/ralr/`)
Reads `.abin` binary files, deserializes them, and executes instructions against a `Registers` state.

- `main.rs` — Uses `tracing` for structured logging. Reads the file, bincode-deserializes to `Vec<OpCode>`, calls `runner()` on a fresh `Registers`.
- `runner.rs` — Synchronous execution loop. `resolve()` dispatches `Operand::Literal` (clone) vs `Operand::Register` (array index into `Registers`). Arithmetic ops resolve both operands, perform the operation, write result to destination register. `Println`/`Print` resolve and display (`Print` explicitly flushes stdout). `Block` recursively executes inner opcodes with the same register context. `Expr` evaluates the expression tree via `eval_expr()` (recursive dispatch through `BinOp`/`UnOp` operators) and writes the result to the destination register. `If` evaluates the condition via `eval_expr()`, then executes the then-body or else-body (if present) based on the `Bool` result; panics on non-Bool conditions.

## Tests

Tests live in `tests/` — a **separate Cargo workspace** (not a member of the root workspace). This lets them depend on `vm_isa`, `ralr_asm`, and `ralr` while avoiding circular dev-dependencies. Run with `cd tests && cargo test`.

Test files:
- `value.rs` — `Value` arithmetic, type-mismatch panics
- `register.rs` — `Registers` initialization and read/write, `Register` index stability
- `opcode.rs` — `OpCode` bincode roundtrip serialization, deserializing real `examples/add.abin`
- `expression.rs` — `Value` operators (`Rem`, `BitAnd`, `BitOr`, `BitXor`, `Shl`, `Shr`, `Neg`, comparisons, logical), expression parsing (precedence, parentheses, unary, constant folding), bincode roundtrip for `Expr`/`Block`, and integration tests (parse + execute)
- `block.rs` — `{ }` block parsing (empty, flat, nested, mixed), bincode roundtrip, string literals containing boundary characters (`{`, `}`, `;`), and edge cases (consecutive semicolons, missing `;` before `}`, unknown keywords)

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

- `examples/add.ralr` — Computes `((1 + 2) - 1) * 2 / 2` using old-style keyword instructions with register `$a1`.
- `examples/print.ralr` — Demonstrates `_println` and `_print` with register values, quoted string literals with escape sequences, and comments.
- `examples/block.ralr` — Nested `{ }` blocks with keyword instructions sharing register context across nesting levels.
- `examples/expr.ralr` — Comprehensive expression syntax demo: arithmetic, comparison, logical, bitwise, shift, unary operators, and complex nested expressions with precedence.
- `examples/if_else.ralr` — `if`/`else if`/`else` control flow: simple branches, chained conditions, nested `if`, and compound expression conditions.
