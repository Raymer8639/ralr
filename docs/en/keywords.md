# Keyword Reference

## Expression Assignment

Expressions are the recommended primary way to program in ralr:

```
$register = expression;
```

Expressions support full operator precedence and parentheses grouping.

### Operator Precedence (Low to High)

| Precedence | Operators | Associativity | Description |
|-----------|-----------|---------------|-------------|
| 2 | `\|\|` | Left | Logical OR |
| 3 | `&&` | Left | Logical AND |
| 4 | `\|` | Left | Bitwise OR |
| 5 | `^` | Left | Bitwise XOR |
| 6 | `&` | Left | Bitwise AND |
| 7 | `==` `!=` | Left | Equal / Not equal |
| 8 | `<` `>` `<=` `>=` | Left | Less / Greater / Less or equal / Greater or equal |
| 9 | `<<` `>>` | Left | Left shift / Right shift |
| 10 | `+` `-` | Left | Addition / Subtraction |
| 11 | `*` `/` `%` | Left | Multiplication / Division / Remainder |
| 12 | `-` `!` `~` | — | Unary negation / Logical NOT / Bitwise NOT |

> **Note**: Operators and operands must be whitespace-separated, e.g. `1 + 2` (correct), `1+2` (error).

### Operator Categories

**Arithmetic operators**: `+` `-` `*` `/` `%`

Operate on integers (I32/I128/U8/U32/U128) and floats (F32/F64). `%` is the remainder operator. Operand types must match.

**Comparison operators**: `==` `!=` `<` `>` `<=` `>=`

Operate on numeric types and strings, returning `Bool` (`true` or `false`). Operand types must match.

**Logical operators**: `&&` `||` `!`

Operate only on `Bool` type. `&&` and `||` use non-short-circuit evaluation (both sides are always evaluated).

**Bitwise operators**: `&` `|` `^` `~`

Operate only on integer types (I32/I128/U8/U32/U128).

**Shift operators**: `<<` `>>`

Operate only on integer types, using wrapping shifts (`wrapping_shl`/`wrapping_shr`). The shift amount is masked.

**Unary operators**: `-` (negation), `!` (logical NOT), `~` (bitwise NOT)

- `-`: Operates on signed integers and floats. If the operand is an unsigned literal, the assembler constant-folds it to a signed negative at compile time (e.g. `-128` → `I32(-128)`).
- `!`: Operates only on `Bool`.
- `~`: Operates only on integer types.

### Parentheses

Use `(` `)` to group sub-expressions and override default precedence:

```
$a1 = (1 + 2) * 3;    // 9, not 7
```

## Conditional Branching (if / else if / else)

Use `if` to execute code blocks based on a condition:

```
if condition_expression {
    instructions;
}
```

### Basic Syntax

```
if $a1 > 0 {
    _println "positive";
}
```

The condition expression must evaluate to `Bool`. If `true`, the following code block executes; otherwise it is skipped.

### else Branch

```
if $a1 > 0 {
    _println "positive";
} else {
    _println "zero or negative";
}
```

### else if Chains

```
if $a1 > 0 {
    _println "positive";
} else if $a1 == 0 {
    _println "zero";
} else {
    _println "negative";
}
```

`else if` can be chained multiple times; the final `else` is optional. Any operator (comparison, logical, arithmetic, etc.) can be used in the condition expression:

```
if $a1 > 0 && $a2 < 100 {
    $a3 = 1;
} else if $a1 == 0 || $a2 == 0 {
    $a3 = 0;
}
```

`if` statements can be arbitrarily nested. `if` statements do not require a trailing semicolon.

## Loop Statement (while)

### Basic Syntax
```
while condition {
   // do_something 
}
```
The `condition` must be of type `bool`. When it evaluates to `true`, the code block will continue to execute until it becomes `false`.

## Code Blocks

Use `{` `}` to organize multiple instructions into a code block. Blocks can be nested and serve as the foundation for control flow structures (`if`, `while`, `fn`, etc.).

```
{
    instruction1;
    instruction2;
    {
        instruction3;   // nested block
    }
    instruction4;
}
```

Instructions inside a block share the outer register state. Empty blocks `{ }` are valid.

## Instructions (Legacy Keyword Syntax)

The following legacy keywords remain available for backward compatibility.

### Arithmetic Instructions

Format: `instruction operand1 operand2 $dest_register`

| Instruction | Description | Example |
|------------|-------------|---------|
| `add` | Addition | `add 1 2 $a1` |
| `sub` | Subtraction | `sub $a1 1 $a1` |
| `mul` | Multiplication | `mul $a1 3 $a1` |
| `div` | Division | `div $a1 2 $a2` |

Operand1 and operand2 can be immediates (literals) or registers. The destination must be a register.

### Print Instructions

Accept one operand (immediate or register) and write it to stdout:

| Instruction | Description | Example |
|------------|-------------|---------|
| `_println` | Print with newline | `_println $a1` / `_println "text"` |
| `_print` | Print without newline | `_print "hello"` / `_print "\n"` |

## I/O Operations (`io` Keyword)

The `io` keyword unifies all I/O operations through sub-commands:

```
io write operand;       // output (no trailing newline)
io writeln operand;     // output (with trailing newline)
io read $register;      // read from stdin, parse as Value, store in register
io readln $register;    // read line from stdin, store as String in register
```

| Sub-command | Parameter | Description |
|------------|-----------|-------------|
| `write` | Operand (literal or register) | Write to stdout without trailing newline |
| `writeln` | Operand (literal or register) | Write to stdout with trailing newline |
| `read` | Destination register | Read a line from stdin, parse using type inference chain (U8→U32→U128→I32→I128→F32→F64→Bool→String fallback) |
| `readln` | Destination register | Read a line from stdin, strip trailing newline, always store as String |

Examples:

```
io writeln "hello world";
io write "answer: ";
io writeln 42;
io read $a1;            // user enters 123 → $a1 = U8(123)
io readln $a2;          // user enters hello → $a2 = String("hello")
```

> The legacy `_print` / `_println` keywords still work and remain backward compatible.

## Registers

ralr provides 5 general-purpose registers, referenced in assembly code with the `$` prefix:

| Register | Syntax |
|----------|--------|
| A1 | `$a1` |
| A2 | `$a2` |
| A3 | `$a3` |
| A4 | `$a4` |
| A5 | `$a5` |

At program start, all registers are initialized to `None`.

## Value Types

The assembler automatically infers value types from literals with the following resolution priority:

| Type | Example | Description |
|------|---------|-------------|
| `Bool` | `true`, `false` | Boolean value |
| `U8` | Integers in range `0`–`255` | 8-bit unsigned integer |
| `U32` | Integers exceeding `u8` range but within `u32` range | 32-bit unsigned integer |
| `U128` | Integers exceeding `u32` range but within `u128` range | 128-bit unsigned integer |
| `I32` | Negative numbers within `i32` range | 32-bit signed integer |
| `I128` | Negative numbers within `i128` range | 128-bit signed integer |
| `F32` | Numbers with a decimal point | 32-bit float |
| `F64` | Numbers with a decimal point (default) | 64-bit float |
| `String` | Double-quoted strings, e.g. `"hello"`, supporting all ASCII escape sequences | String |

### Escape Sequences

String literals support the following escape sequences:

| Sequence | Meaning |
|----------|---------|
| `\n` | Newline |
| `\t` | Tab |
| `\\` | Backslash `\` |
| `\"` | Double quote `"` |
| `\r` | Carriage return |
| `\'` | Single quote `'` |
| `\a` | Bell (0x07) |
| `\b` | Backspace (0x08) |
| `\f` | Form Feed (0x0C) |
| `\v` | Vertical Tab (0x0B) |
| `\e` | Escape (0x1B) |
| `\0` | Null character |
| `\xNN` | Hex byte (e.g. `\x20` = space) |
| `\u{NNNN}` | Unicode codepoint (e.g. `\u{4e2d}` = 中) |

Example: `"hello\nworld"` prints `hello` and `world` on separate lines.

> **Note**: Strings must be wrapped in double quotes `"` (e.g. `"hello"`). Unquoted words are no longer recognized as strings and will trigger a parse error. The type inference order means `255` is parsed as `U8` while `256` is parsed as `U32`.

## Syntax Delimiters

- **Semicolon `;`**: Instruction separator. Multiple instructions can be written on a single line separated by semicolons. Every instruction must end with `;`.
- **Braces `{` `}`**: Code block delimiters. Used to organize multiple instructions into a nested block.

## Comments

| Syntax | Description |
|--------|-------------|
| `//` | Line comment. Everything from `//` to end of line is ignored. |
| `/* … */` | Block comment. Everything between `/*` and `*/` is ignored, supports multiple lines. |

Comment delimiters inside string literals are treated as ordinary characters and are not recognized as comments.
