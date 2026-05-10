# ralr Documentation Index

ralr is a simple register-based virtual machine (VM) and assembler written in Rust.

## Document List

- [Installation Guide](install.md) — How to compile, install, and uninstall ralr
- [Keyword Reference](keywords.md) — Expression syntax, operator tables, code blocks, registers, and value type reference
- [Assembly Language Guide](assembly-guide.md) — How to write `.ralr` assembly source files with complete expression and legacy syntax examples

## Language Feature Overview

- **Expression syntax**: `$reg = expr;` — 24 operators with standard precedence and parentheses grouping
- **Code blocks**: `{ ... }` — Nested instruction sequences, foundation for control flow structures
- **5 general-purpose registers**: `$a1`–`$a5`
- **10 value types**: `None`, `I32`, `I128`, `U8`, `U32`, `U128`, `F32`, `F64`, `Bool`, `String`
- **I/O operations**: `io write`/`writeln` for output, `io read`/`readln` for input
- **Control flow**: `if`/`else if`/`else` conditional branching
- **Legacy keyword instructions**: `add`/`sub`/`mul`/`div`, `_print`/`_println` (backward compatible)
