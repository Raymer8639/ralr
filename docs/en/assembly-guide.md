# Assembly Language Guide

## Source File Format

`.ralr` files are ralr assembly source files in plain text format. Each instruction ends with a semicolon `;`. Multiple instructions can be written on a single line.

## Expressions (Recommended Syntax)

Expressions are the recommended primary way to program in ralr, supporting full operator precedence and parentheses grouping:

```
$register = expression;
```

### Basic Operations

```
$a1 = 10 + 5;
$a2 = ($a1 - 3) * 2;
$a3 = $a2 > 20;       // comparison, returns Bool
$a4 = 255 & 15;       // bitwise
$a5 = 1 << 4;         // shift
$a6 = -(10);          // unary negation
$a7 = !false;         // logical NOT
```

### Operator Overview

| Category | Operators | Example |
|----------|-----------|---------|
| Arithmetic | `+` `-` `*` `/` `%` | `$a1 = 10 + 5;` |
| Comparison | `==` `!=` `<` `>` `<=` `>=` | `$a1 = 5 < 10;` |
| Logical | `&&` `\|\|` `!` | `$a1 = true && false;` |
| Bitwise | `&` `\|` `^` `~` | `$a1 = 240 & 15;` |
| Shift | `<<` `>>` | `$a1 = 1 << 4;` |
| Unary | `-` `!` `~` | `$a1 = -5;` |

> **Note**: Operators and operands must be whitespace-separated, e.g. `1 + 2` (correct), `1+2` (error).

### Operator Precedence

Higher-precedence operators evaluate first. Use parentheses `( )` to override default precedence:

```
$a1 = 1 + 2 * 3;       // 7   (* before +)
$a1 = (1 + 2) * 3;     // 9   (parentheses override)
$a1 = 1 + 2 < 3 * 4;   // true (arithmetic before comparison)
```

Full precedence table (low to high): `||` → `&&` → `|` → `^` → `&` → `==` `!=` → `<` `>` `<=` `>=` → `<<` `>>` → `+` `-` → `*` `/` `%` → unary `-` `!` `~`

## Code Blocks

Use `{` `}` to organize multiple instructions into a code block. Blocks support nesting:

```
{
    $a1 = 10;
    _println $a1;
    {
        $a2 = $a1 + 5;
        _println $a2;
    }
}
```

Instructions inside a block share the outer register state. Code blocks serve as the foundation for control flow structures (`if`, `while`, `fn`, etc.).

## Conditional Branching (if / else if / else)

Use `if` to execute different code blocks based on a condition:

```
$a1 = 42;

// Basic if
if $a1 > 0 {
    _println "positive";
}

// if/else
if $a1 > 100 {
    _println "large";
} else {
    _println "not large";
}

// if/else if/else chain
if $a1 > 100 {
    _println "large";
} else if $a1 == 42 {
    _println "the answer";
} else {
    _println "something else";
}

// Complex condition
if $a1 > 0 && $a2 < 100 || $a3 == 1 {
    $a4 = 10;
}

// Nested if
if $a1 >= 0 {
    if $a1 == 42 {
        _println "found it";
    }
}
```

The condition expression must evaluate to `Bool` and supports all operators. `if` statements do not require a trailing semicolon. There is no limit on `else if` chains; the final `else` is optional.

## Loop Statement (while)

Use the `while` keyword to repeatedly execute a block of code based on whether a condition evaluates to true or false:
```
$a1 = 0;
while $a1 < 10 {
    $a1 = $a1 + 1;
    io writeln $a1
}
```
## Comments

ralr supports C-style comments:

```
// This is a line comment
$a1 = 5;  // end-of-line comment

/* This is a block comment
   that can span multiple lines */
$a2 = 10;

/* inline block comment */ _println $a2;
```

Comment delimiters (`//`, `/*`, `*/`) inside strings are treated as plain text and not recognized as comments:

```
_println "this // is not a comment";
```

## I/O Operations

The `io` keyword provides unified I/O. For output, use `write` (no trailing newline) or `writeln` (with trailing newline). For input, use `read` (parse as a value) or `readln` (store as a string).

### Output

```
// Recommended: io keyword
io writeln $a1;
io writeln "hello world";
io write "no newline";
io write "\n";

// Legacy keywords (backward compatible)
_println $a1;
_print "hello";
```

### Input

`io read` reads from stdin and automatically parses into the appropriate type. `io readln` reads a line and stores it as a string:

```
io read $a1;       // enter 42 → parsed as number
io readln $a2;     // enter hello → stored as string
io writeln $a1;    // print result
```

## Legacy Keyword Syntax (Backward Compatible)

The following keyword-style instructions are still available:

### Example: Basic Arithmetic

```
add 1 2 $a1;
sub $a1 1 $a1;
mul $a1 2 $a1;
div $a1 2 $a1;
```

The above computation: `((1 + 2) - 1) × 2 ÷ 2 = 2`, final result in `$a1`.

Equivalent expression syntax:

```
$a1 = 1 + 2;
$a1 = $a1 - 1;
$a1 = $a1 * 2;
$a1 = $a1 / 2;
```

## Compiling to Binary

Use `ralr-asm` to compile `.ralr` source files into `.abin` binary files:

```bash
ralr-asm input.ralr -o program.abin
```

## Running Binaries

Use `ralr` to run `.abin` binary files:

```bash
ralr program.abin
```

## Complete Workflow

```bash
# 1. Write assembly source file
cat > test.ralr << 'EOF'
$a1 = (10 + 5) * 2 - 3;
_println $a1;
$a2 = $a1 > 20;
_println $a2;
_println "done";
EOF

# 2. Compile
ralr-asm test.ralr -o test.abin

# 3. Run
ralr test.abin
```

## Complete Examples

See `examples/expr.ralr` for a comprehensive test of all operators. See `examples/io.ralr` for I/O operation demos.
