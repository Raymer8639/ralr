# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Register-based VM with 5 general-purpose registers
- Assembler (`ralr-asm`) with `.ralr` → `.abin` compilation
- Expression parser with full operator precedence (Pratt parser)
- Control flow: `if`/`else if`/`else` with expression conditions
- `while` loops with expression conditions
- Nested `{ }` code blocks
- Variable system: `let`/`let mut` with assignment
- Function definitions, calls, parameters, and return values
- Unified I/O: `io write`/`writeln`/`read`/`readln`
- 10-value type system: `None`, `I32`, `I128`, `U8`, `U32`, `U128`, `F32`, `F64`, `Bool`, `String`
- Binary format via `bincode` serialization
- Multi-language documentation (中文, English, 日本語, Русский)
- Release profile with LTO, single codegen unit, and symbol stripping
- `Registers::take()` for zero-clone value extraction
- `resolve_ref()` for no-clone I/O output paths
