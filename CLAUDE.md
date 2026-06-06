# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development rules

The following conventions apply to every change made in this repository:

1. **Every feature → test + docs.** Before committing a new feature or bug fix, add or update the corresponding tests under `tests/tests/` and update relevant documentation (`CLAUDE.md`, docs under `docs/`, or `README.md`).
2. **Every release → version bump.** When publishing a release, update the version in all `Cargo.toml` files, update `CHANGELOG.md` with the release notes, and tag the commit (`git tag vX.Y.Z`).
3. **Commit granularity.** Commit each logical change separately with a descriptive message. Do not batch unrelated changes into one commit.
4. **Never push directly to `main`.** Always push to a feature branch (e.g., `feature/description`), then tell the user so they can open a pull request to merge into `main`.

## Build & Run

```bash
cargo build                    # Dev build all workspace crates
cargo build --release          # Release build (LTO, single codegen unit, stripped, panic=abort)
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

### `vm_isa` (shared library — `vm-isa/`)
The instruction set architecture. Defines the core types that both the assembler and VM depend on:

- **`OpCode`** — Arithmetic (`Add`, `Sub`, `Mul`, `Div`) each take `(Operand, Operand, Register)`. I/O via `IO(IoOp)` with `Write`/`Writeln`/`Read`/`Readln` sub-variants (legacy `Println`/`Print` still exist). Control flow: `Block(Vec<OpCode>)` for nested scopes, `Expr(Expr, Operand)` for expression tree evaluation, `If(Expr, Vec<OpCode>, Option<Vec<OpCode>>)` for conditional branching, `While(Expr, Vec<OpCode>)` for while loops. Variable declarations: `Variable(String, Variable)`. Functions: `FnDef { name, params, body }` for definitions, `Call { name, args, dest }` for calls, `Return(Expr)` for return values. OOP: `ClassDef { name, def }` for class definitions, `New { class, args, dest }` for instantiation, `MethodCall { recv, method, args, dest }` for method dispatch, `SetField { base, path, value }` for field assignment.
- **`Expr`** (`vm-isa/src/expr.rs`) — Expression AST with `Operand(Operand)`, `Binary`, `Unary`, and `Field(Box<Expr>, String)` nodes. Leaf nodes (`Operand`) can be literals, register references, or named variable references. `Field` nodes are object field accesses (`obj.field`); chains like `a.b.c` nest `Field` nodes. 18 binary operators (`BinOp`) and 3 unary operators (`UnOp`) covering arithmetic, comparison, logical, and bitwise operations. `eval_expr()` evaluates against both the register file and a variable hashmap.
- **`Operand`** — `Literal(Value)`, `Register(Register)`, or `Variable(String, Variable)`. Separates immediates, register references, and named variable references at the type level.
- **`Value`** — A tagged union of 11 variants: `None`, signed/unsigned ints (`I32`, `I128`, `U8`, `U32`, `U128`), floats (`F32`, `F64`), `Bool`, `String`, and `Object(Box<Instance>)` (a user-defined object). Arithmetic is implemented via the `op!` macro that generates type-matching arms; `Object` is not a numeric/comparable variant, so arithmetic or comparison on objects panics.
- **`Variable`** (`vm-isa/src/variable.rs`) — `Variable { is_mut: bool, value: Value }`. Named, mutable-or-immutable value storage that lives in a hashmap parallel to the register file.
- **`FnDef`** (`vm-isa/src/function.rs`) — `FnDef { params: Vec<String>, body: Vec<OpCode> }`. Stored representation of a user-defined function, registered at definition time and replayed on each call.
- **`ClassDef`** / **`Instance`** (`vm-isa/src/class.rs`) — `ClassDef { fields: Vec<(String, Expr)>, methods: BTreeMap<String, FnDef> }` is the stored class blueprint: ordered field initializers (evaluated at construction) plus a method table (keyed by name; `BTreeMap` rather than `AHashMap` because the latter lacks `Deserialize` without its serde feature). `Instance { class: String, fields: BTreeMap<String, Value> }` is a live object. By convention a method's first parameter is `self`, bound to the receiver. A method named `init` is the constructor, invoked by `new`.
- **`Register`** — A `Copy` enum (`A1`–`A5`, `SystemVarBuffer`) with `index()` mapping to array slots `0..5`. `SystemVarBuffer` is an internal buffer used during variable declaration to transfer expression results into the variable hashmap.
- **`Registers`** — Runtime register file backed by `[Value; 6]` with O(1) `read`/`write`/`take`. `take()` uses `mem::replace` with `Value::None` to avoid cloning when the old value is no longer needed (e.g., `Variable` insertion, `Call` return-value capture).

**Key design:** Resolution happens at the `OpCode` level via `Operand`, not inside `Value`. The VM's `resolve()` function dispatches `Operand::Literal(v)` → clone, `Operand::Register(r)` → array lookup, or `Operand::Variable(_, v)` → clone from the variable's current value. Variables live in an `AHashMap<String, Variable>` that is threaded through `runner()` and `eval_expr()`, separate from the fixed-size register file. Functions and classes live in parallel `AHashMap<String, FnDef>` / `AHashMap<String, ClassDef>` tables, also threaded through `runner()`.

**Object semantics:** Objects use reference-like mutation semantics. `let`/`let mut` controls whether the *variable binding* can be reassigned, but field mutation (`obj.field = ...`, or a method mutating `self`) is always permitted and persists. Method calls bind the receiver to a mutable `self`, run the body, then write the (possibly mutated) `self` back into the receiver variable.

**Error behavior:** `Value` arithmetic operators **panic** on type-mismatched operands (e.g., `I32(1) + String("x")` panics) — there is no graceful error propagation at runtime.

All crates use Rust edition 2024. Workspace dependencies: `clap` (CLI), `anyhow` (errors), `bincode` + `serde` (serialization), `tracing` + `tracing-subscriber` (structured logging in `ralr`), `ahash` (fast hasher for variable map). String escape processing is handled by `unescape_str()` in `reader.rs` (no external crate needed).

### `ralr-asm` (assembler — `ralr-asm/`)
Reads `.ralr` source files and emits `.abin` binary files (default output: `output.abin`).

**`.ralr` instruction format:**
```
; Two syntax styles — expression assignment (preferred) and keyword (legacy):
$reg = expression;                 ; e.g. $a1 = (1 + 2) * 3;
keyword operand1 operand2 $dest;   ; e.g. add 1 2 $a1
_println|_print value;             ; e.g. _println "hello"
io write|writeln value;            ; e.g. io writeln "hello"
io read|readln $reg;               ; e.g. io read $a1
{ instruction; ... }               ; code block (nests, shares register context)
if cond { ... } [else { ... }]     ; conditional branch (else if desugars to nested If)
while cond { ... }                 ; while loop (condition must be Bool)
let name = expr;                   ; immutable variable declaration
let mut name = expr;               ; mutable variable declaration
name = expr;                       ; variable assignment (mut only)
fn name(params) { ... }            ; function definition
$reg = call name(args);            ; function call with return value capture
return expr;                       ; return from function
class Name { ... }                 ; class definition (fields via let, methods via fn)
$reg = new Name(args);             ; object instantiation (runs init method if present)
obj.field                          ; field read (in expressions)
obj.field = expr;                  ; field assignment
obj.method(args);                  ; method call (result discarded)
$reg = obj.method(args);           ; method call with return value capture
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

### While loops (`while`)

```
while $a1 < 10 {
    $a1 = $a1 + 1;
}
```

- Condition must evaluate to `Bool` (panics otherwise)
- Body repeats until condition becomes `Bool(false)`
- Variables declared inside the loop body persist after the loop

### Variables (`let` / `let mut`)

Named, dynamically-allocated values stored in a hashmap parallel to the register file:

```
let x = 42;
let mut y = 10;
y = y + 1;
io writeln y;
```

- `let` declares an immutable variable; `let mut` allows reassignment
- Assignment to an immutable variable is a runtime error (panics)
- Variables are accessible in expressions by name (e.g., `$a1 = x + y;`)
- The `=` sign after a non-keyword, non-register token triggers variable/expression assignment parsing
- Internally: `let x = expr;` evaluates `expr` into `Register::SystemVarBuffer`, then `OpCode::Variable` moves the value into the hashmap

### Classes (`class` / `new`)

Object-oriented syntax layered on top of the existing variable/function machinery:

```
class Point {
    let x = 0;             // field with default initializer
    let y = 0;

    fn init(self, x, y) {  // constructor — first param is always `self`
        self.x = x;
        self.y = y;
    }

    fn sum(self) {         // method
        return self.x + self.y;
    }

    fn shift(self, dx, dy) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}

let mut p = new Point(3, 4);  // construct (calls init with 3, 4)
io writeln p.x;               // field read in an io operand
$a1 = p.sum();                // method call capturing return value
p.shift(10, 20);              // method call mutating the receiver
p.x = 100;                    // direct field assignment
io writeln p;                 // prints: Point { x: 100, y: 24 }
```

- A class body may contain **only** field declarations (`let name = expr;`) and method definitions (`fn name(self, ...) { ... }`). The `let` lowering (`Expr` into `SystemVarBuffer` + `Variable`) is reconstructed by `build_class_def()` into `(field_name, initializer)` pairs.
- Methods take `self` as their explicit first parameter. Calling `obj.method(a, b)` binds `obj` to `self` and `a, b` to the remaining parameters (so `params.len() == 1 + args.len()`).
- `new Name(args)` evaluates each field initializer in declaration order, then — if a method named `init` exists — runs it as a constructor; otherwise a non-empty argument list is an error.
- The **receiver of a method call must be a simple variable name** (no nested field path). For a deeper receiver, bind it to an intermediate variable first.
- Field access (`obj.field`) works wherever expressions are parsed: `$reg = ...`, `let`, conditions, function/method arguments, and `io write`/`writeln`/`_print`/`_println` operands (lowered via `io_operand()`). It does **not** work as a bare keyword-instruction operand elsewhere.
- Field *assignment* RHS is a plain expression — to store a `new`/`call`/method-call result into a field, assign it to an intermediate variable first.
- Objects print as `ClassName { field: value, ... }` (fields sorted, since `Instance.fields` is a `BTreeMap`).

### I/O operations (`io` keyword)

The `io` keyword unifies all I/O under sub-commands:

```
io write "hello";        // print without trailing newline
io writeln "world";      // print with trailing newline
io read $a1;             // read stdin, parse as Value, store in register
io readln $a2;           // read stdin line, store as String in register
```

- `write` / `writeln` accept any operand (literal or register)
- `read` parses input using the same numeric-inference chain as the assembler (`u8` → `u32` → `u128` → `i32` → `i128` → `f32` → `f64` → `Bool` → `String` fallback)
- `readln` always stores the input line as a `String` (trailing newline stripped)
- The legacy `_print` / `_println` keywords continue to work

- `main.rs` — Parses CLI args (`-o` for output name, multiple input files supported), reads each file fully, passes content to `reader::parse()`.
- `reader.rs` — `parse()` is the public entry point. It delegates to `parse_block()`, a recursive-descent parser that handles `{ }` blocks, `;`-delimited instructions, `//` and `/* */` comments, and string literals (including boundary characters inside strings). `parse_instruction()` dispatches individual instructions by keyword. Support functions: `to_value()` converts literal tokens (string literals are processed through `unescape_str()` for escape sequences `\n`, `\t`, `\r`, `\\`, `\"`, `\'`, `\a`, `\b`, `\f`, `\v`, `\e`, `\0`, `\xNN`, `\u{NNNN}`; bare words `true`/`false` map to `Bool`; numbers parse via `try_parse!` chain: `u8` → `u32` → `u128` → `i32` → `i128` → `f32` → `f64`). `to_register()` handles `$a1`–`$a5` → `Register::A1`–`A5`. `to_operand()` dispatches: `$`-prefixed tokens → `Operand::Register`, unrecognized bare words → `Operand::Variable`, everything else → `Operand::Literal`. `tokenize()` splits on whitespace while keeping quoted strings intact. Unrecognized keywords are silently ignored. Expression assignments (`$a1 = ...`) are detected by matching `$`-prefixed first token with `=` as second token, then delegating to `parse_expr_str()`. Variable assignments (`name = ...`) are detected by non-keyword, non-register first token with `=`. The `if` and `while` keywords are intercepted at the `parse_block` level: after accumulating instruction text, control-flow parsing extracts the condition via `parse_expr_str()`, consumes the body block, and for `if` handles `else { }` / `else if ...` chains via `parse_else_tail()`. The `io` keyword dispatches to `write`/`writeln`/`read`/`readln` sub-commands (`write`/`writeln` operands go through `io_operand()`, which lowers a field-access token to an `Expr` into `SystemVarBuffer`). The `let`/`let mut` keywords parse a variable name, skip to `=`, then route the initializer through `push_rhs_opcode()` (which dispatches `call`/`new`/method-call/plain-expression into `SystemVarBuffer`) and emit `OpCode::Variable`. The `class` keyword is intercepted at the `parse_block` level (like `fn`/`while`/`if`): it parses the class name, recursively parses the body block, then `build_class_def()` post-processes the body opcodes into `(field, initializer)` pairs and a method table. OOP statements are detected in `parse_instruction()` via guard arms: a dotted LHS with `=` → `SetField`; RHS `new`/method-call → `New`/`MethodCall`; a bare `recv.method(...)` (no `=`) → `MethodCall` with the result discarded. Helpers `push_new_opcode()`, `push_method_call_opcode()`, and `parse_arg_list()` (shared with `push_call_opcode()`) build the opcodes.
- `expr_parser.rs` — Pratt precedence-climbing expression parser. `parse_expr_str()` tokenizes the expression string (with operator-aware splitting) then calls `parse_expression()`. Constant-folds unary `-` on unsigned literals (e.g., `-128` → `I32(-128)` instead of `Unary(Neg, U8(128))`). A bare-word token containing `.` that does not parse as a numeric literal (so `3.14` stays a float) becomes a `Field`-access chain (`base.f1.f2` → nested `Expr::Field`).

## Comments

- `//` — line comment, everything from `//` to end of line is ignored
- `/* … */` — block comment, everything between `/*` and `*/` is ignored (multi-line)
- Comment delimiters inside `"..."` strings are treated as literal text, not comments.

### `ralr` (VM runtime — `ralr/`)
Reads `.abin` binary files, deserializes them, and executes instructions against a `Registers` state.

- `main.rs` — Uses `tracing` for structured logging. Reads the file, bincode-deserializes to `Vec<OpCode>`, creates a fresh `Registers` and `AHashMap<String, Variable>`, calls `runner()`.
- `runner.rs` — Synchronous execution loop. Two operand-resolution helpers: `resolve()` returns an owned `Value` (cloning for `Register`/`Variable` lookups — used by arithmetic ops that consume operands); `resolve_ref()` returns a shared `&Value` without cloning (used by I/O output paths where only `Display` is needed). Arithmetic ops resolve both operands via `resolve()`, perform the operation, write result to destination register. `IO(IoOp::Write)`/`Writeln` resolve via `resolve_ref()` and display (`Write` flushes stdout); `Read`/`Readln` read from stdin. `Block` recursively executes inner opcodes with the same register/variable/function context. `Expr` evaluates the expression tree via `eval_expr()` (recursive dispatch through `BinOp`/`UnOp` operators, with variable lookup at leaf nodes) and writes the result to the destination operand. `If` evaluates the condition via `eval_expr()`, then executes the then-body or else-body (if present) based on the `Bool` result; panics on non-Bool conditions. `While` re-evaluates the condition each iteration; panics on non-Bool. `Variable` inserts a named value into the hashmap using `SystemVarBuffer` as the source (via `regs.take()` to avoid cloning). `FnDef` registers a function in the function table. `Call` evaluates arguments, saves shadowed variables, binds parameters, executes the function body, reads the return value from `SystemVarBuffer` (via `regs.take()`), restores shadowed variables, and writes the result to the destination. `Return` evaluates an expression and writes the result to `SystemVarBuffer`. `ClassDef` registers a class in the class table. `New` evaluates field initializers in order, builds an `Instance`, runs the `init` method (if any) via the `call_method()` helper, and writes the object to `dest`. `MethodCall` clones the receiver object out of its variable, looks up the method on its class, runs `call_method()`, writes the (mutated) receiver back into the variable, and writes the return value to `dest`. `SetField` evaluates the value, then navigates the field path from the base variable and writes the final field. The function and class tables (`&mut AHashMap<String, FnDef>` / `&mut AHashMap<String, ClassDef>`) are threaded through all recursive `runner` calls. Shared helpers: `write_dest()` writes a value to a register or mutable-variable destination; `call_method()` binds `self` + params, runs the body, and returns the mutated receiver (the method's return value stays in `SystemVarBuffer`).

## Tests

Tests live in `tests/` — a **separate Cargo workspace** (not a member of the root workspace). This lets them depend on `vm_isa`, `ralr_asm`, and `ralr` while avoiding circular dev-dependencies. The test runner entry point is `tests/src/lib.rs` (module declarations only); actual test cases are at `tests/tests/`. Run with `cd tests && cargo test`.

Test files:
- `tests/tests/value.rs` — `Value` arithmetic, type-mismatch panics
- `tests/tests/register.rs` — `Registers` initialization and read/write, `Register` index stability
- `tests/tests/opcode.rs` — `OpCode` bincode roundtrip serialization, deserializing real `examples/add.abin`
- `tests/tests/expression.rs` — `Value` operators, expression parsing (precedence, parentheses, unary, constant folding), bincode roundtrip, and integration tests (parse + execute)
- `tests/tests/block.rs` — `{ }` block parsing, bincode roundtrip, string literals containing boundary characters, and edge cases
- `tests/tests/oop.rs` — class parsing (`ClassDef`/`New`/`MethodCall`/`SetField`), field-access expressions, `Object` bincode roundtrip and `Display`, and integration tests (construct, read/assign fields, method return values, mutation persistence, no-`init` defaults, factory methods)

> **Warning:** The CI workflow (`.github/workflows/rust.yml`) runs `cargo test` from the root workspace, which does **not** execute the integration tests in `tests/`. Run them manually with `cd tests && cargo test` — CI only validates that workspace crates compile.

## Data flow

```
.ralr (text) ──[ralr-asm]──→ .abin (bincode) ──[ralr]──→ register state
```

## Documentation

Docs are available in 4 languages under `docs/`:

| Language | Directory |
|----------|-----------|
| 中文 (Chinese) | `docs/zh/` |
| English | `docs/en/` |
| 日本語 (Japanese) | `docs/ja/` |
| Русский (Russian) | `docs/ru/` |

Each language contains: `index.md`, `keywords.md`, `assembly-guide.md`, `install.md`.
Start at `docs/INDEX.md` for the language selector.

## Examples

- `examples/add.ralr` — Computes `((1 + 2) - 1) * 2 / 2` using old-style keyword instructions with register `$a1`.
- `examples/print.ralr` — Demonstrates `_println` and `_print` with register values, quoted string literals with escape sequences, and comments.
- `examples/block.ralr` — Nested `{ }` blocks with keyword instructions sharing register context across nesting levels.
- `examples/expr.ralr` — Comprehensive expression syntax demo: arithmetic, comparison, logical, bitwise, shift, unary operators, and complex nested expressions with precedence.
- `examples/if_else.ralr` — `if`/`else if`/`else` control flow: simple branches, chained conditions, nested `if`, and compound expression conditions.
- `examples/io.ralr` — `io write`/`writeln` output with literals, registers, and bools. Legacy `_print`/`_println` compatibility.
- `examples/var.ralr` — Variable declaration (`let`/`let mut`), variable assignment, and expression evaluation with named variables.
- `examples/fn.ralr` — Function definitions (`fn`), calls (`call`), return values, parameter scoping, closures over outer variables, and conditional logic inside functions.
- `examples/oop.ralr` — Class definitions (`class`) with fields and methods, construction (`new`) with an `init` constructor, field reads/assignment, method calls with return values and in-place mutation, and a class without `init`.
