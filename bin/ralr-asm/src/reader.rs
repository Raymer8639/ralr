//! Parses `.ralr` source text into [`OpCode`] instructions.
//! 中文：将 `.ralr` 源文本解析为 [`OpCode`] 指令。
//!
//! Instructions are semicolon-delimited. Each instruction consists of a
//! keyword followed by whitespace-separated operands. Quoted strings are
//! kept as single tokens even when they contain spaces.
//! 中文：指令由分号分隔。每条指令由一个关键字和空格分隔的操作数组成。带引号的字符串即使包含空格也保留为单个标记。

use anyhow::{Result, anyhow};
use std::char;
use vm_isa::{
    op_code::{Expr, IoOp, OpCode, Operand},
    register::Register,
    value::Value,
    variable::Variable,
};

use crate::expr_parser::parse_expr_str;

/// Tries each numeric parse in order, returning the first success.
/// 中文：依次尝试每种数字解析，返回第一个成功的结果。
///
/// The chain is ordered from narrowest to widest type so that a number
/// like `42` becomes `U8` rather than `I32` when possible.
/// 中文：链按从窄到宽的类型排列，以便像 `42` 这样的数字在可能的情况下成为 `U8` 而不是 `I32`。
macro_rules! try_parse {
    ($s:expr, $($ty:ty => $var:ident),* $(,)?) => {
        $(if let Ok(v) = $s.parse::<$ty>() { return Ok(Value::$var(v)); })*
    };
}

/// Processes backslash escape sequences in a string literal.
/// 中文：处理字符串字面量中的反斜杠转义序列。
///
/// Supports: `\n` `\t` `\r` `\\` `\"` `\'` `\a` `\b` `\f` `\v` `\e` `\0` `\xNN` `\u{NNNN}`.
/// 中文：支持：`\n` `\t` `\r` `\\` `\"` `\'` `\a` `\b` `\f` `\v` `\e` `\0` `\xNN` `\u{NNNN}`。
///
/// Returns `None` on any unrecognized or malformed escape.
/// 中文：对于任何无法识别或格式错误的转义，返回 `None`。
fn unescape_str(s: &str) -> Option<String> {
    let mut chars = s.chars();
    let mut out = String::with_capacity(s.len());

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next()? {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            '\'' => out.push('\''),
            'a' => out.push('\x07'),
            'b' => out.push('\x08'),
            'f' => out.push('\x0C'),
            'v' => out.push('\x0B'),
            'e' => out.push('\x1B'),
            '0' => out.push('\0'),
            'x' => {
                let hi = chars.next()?.to_digit(16)?;
                let lo = chars.next()?.to_digit(16)?;
                out.push(((hi * 16 + lo) as u8) as char);
            }
            'u' => {
                // Rust-style Unicode escape: \u{XXXX}
                // 中文：Rust 风格的 Unicode 转义：\u{XXXX}
                if chars.next()? != '{' {
                    return None;
                }
                let mut hex = String::new();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    if !c.is_ascii_hexdigit() {
                        return None;
                    }
                    hex.push(c);
                }
                let codepoint = u32::from_str_radix(&hex, 16).ok()?;
                out.push(char::from_u32(codepoint)?);
            }
            _ => return None,
        }
    }
    Some(out)
}

/// Converts a token to a literal [`Value`].
/// 中文：将标记转换为字面量 [`Value`]。
///
/// Handles quoted strings (via [`unescape_str`]), booleans, and numeric
/// literals (via the [`try_parse!`] chain). Numeric inference follows the
/// chain order: U8 → U32 → U128 → I32 → I128 → F32 → F64.
/// 中文：处理带引号的字符串（通过 [`unescape_str`]）、布尔值和数字字面量（通过 [`try_parse!`] 链）。数字推断遵循链顺序：U8 → U32 → U128 → I32 → I128 → F32 → F64。
pub(crate) fn to_value(str: &str) -> Result<Value> {
    if let Some(inner) = str.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return unescape_str(inner)
            .map(Value::String)
            .ok_or_else(|| anyhow!("invalid escape sequence: {str}"));
    }

    match str {
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        "" => Err(anyhow!("empty token")),
        s => {
            try_parse!(s, u8 => U8, u32 => U32, u128 => U128, i32 => I32, i128 => I128, f32 => F32, f64 => F64);
            Err(anyhow!("unrecognized token: {s}"))
        }
    }
}

/// Converts a `$aN` token to a [`Register`] discriminant.
/// 中文：将 `$aN` 标记转换为 [`Register`] 判别式。
///
/// # Panics
///
/// Panics if the token does not start with `$` (should only be called
/// after checking the prefix) or names an unknown register.
/// 中文：如果标记不以 `$` 开头（应仅在检查前缀后调用）或命名了未知寄存器，则会 panic。
pub(crate) fn to_register(str: &str) -> Result<Register> {
    if let Some(reg) = str.strip_prefix('$') {
        match reg {
            "a1" => Ok(Register::A1),
            "a2" => Ok(Register::A2),
            "a3" => Ok(Register::A3),
            "a4" => Ok(Register::A4),
            "a5" => Ok(Register::A5),
            _ => panic!("unknown register: {str}"),
        }
    } else {
        panic!("expected register, got: {str}")
    }
}

/// Dispatches a token to [`Operand::Register`] if it starts with `$`,
/// otherwise to [`Operand::Literal`].
/// 中文：如果标记以 `$` 开头，则分派到 [`Operand::Register`]，否则分派到 [`Operand::Literal`]。
fn to_operand(str: &str) -> Result<Operand> {
    if str.starts_with('$') {
        // Register reference: $a1–$a5
        // 中文：寄存器引用：$a1–$a5
        Ok(Operand::Register(to_register(str)?))
    } else if str.starts_with('"') {
        // Quoted string literal — must parse as a valid value.
        // 中文：带引号的字符串字面量 — 必须解析为有效值。
        Ok(Operand::Literal(to_value(str)?))
    } else {
        // Bare word — try literal (number, bool) first,
        // fall back to named variable reference.
        // 中文：裸词 — 首先尝试字面量（数字、布尔值），回退到命名变量引用。
        match to_value(str) {
            Ok(v) => Ok(Operand::Literal(v)),
            Err(_) => Ok(Operand::Variable(
                str.to_string(),
                Variable {
                    is_mut: false,
                    value: Value::None,
                },
            )),
        }
    }
}

/// Splits a command string into tokens on whitespace while keeping
/// quoted strings intact.
/// 中文：在空白字符处将命令字符串拆分为标记，同时保持带引号的字符串完整。
///
/// Uses byte-level indexing, which is safe because `"` (0x22) and `\`
/// (0x5C) never appear inside UTF-8 multi-byte sequences.
/// 中文：使用字节级索引，这是安全的，因为 `"` (0x22) 和 `\` (0x5C) 永远不会出现在 UTF-8 多字节序列中。
fn tokenize(s: &str) -> Vec<&str> {
    let s = s.trim();
    let mut tokens = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                    // skip escape sequence (backslash + escaped char)
                    // 中文：跳过转义序列（反斜杠 + 转义字符）
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            tokens.push(&s[start..i]);
        } else {
            let start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(&s[start..i]);
        }
    }
    tokens
}

/// Parses a function call string and pushes the corresponding
/// [`OpCode::Call`] onto `cmds`.
/// 中文：解析函数调用字符串并将对应的 [`OpCode::Call`] 推送到 `cmds`。
///
/// The `text` must start with `call` followed by the function name and
/// parenthesised argument list, e.g. `call add(1, 2)`.
/// 中文：`text` 必须以 `call` 开头，后跟函数名和带括号的参数列表，例如 `call add(1, 2)`。
fn push_call_opcode(text: &str, dest: Operand, cmds: &mut Vec<OpCode>) -> Result<()> {
    let rest = text[4..].trim_start(); // after "call"
    let paren_pos = rest
        .find('(')
        .ok_or(anyhow!("expected '(' in function call"))?;
    let fn_name = rest[..paren_pos].trim().to_string();
    if fn_name.is_empty() {
        return Err(anyhow!("expected function name before '('"));
    }
    let after_paren = &rest[paren_pos + 1..];
    let close_paren = after_paren
        .rfind(')')
        .ok_or(anyhow!("expected ')' in function call"))?;
    let args_str = after_paren[..close_paren].trim();
    // Parse comma-separated argument expressions.
    // 中文：解析逗号分隔的参数表达式。
    let args: Vec<Expr> = if args_str.is_empty() {
        vec![]
    } else {
        // Split on top-level commas only (not inside nested parens).
        // 中文：仅按顶层逗号拆分（不在嵌套括号内拆分）。
        let mut arg_exprs = vec![];
        let mut depth = 0u32;
        let mut start = 0usize;
        for (i, ch) in args_str.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    arg_exprs.push(parse_expr_str(args_str[start..i].trim())?);
                    start = i + 1;
                }
                _ => {}
            }
        }
        arg_exprs.push(parse_expr_str(args_str[start..].trim())?);
        arg_exprs
    };
    cmds.push(OpCode::Call {
        name: fn_name,
        args,
        dest,
    });
    Ok(())
}

/// Parses a single instruction string (already trimmed, no surrounding
/// braces or semicolons) into an [`OpCode`], appending it to `cmds`.
/// 中文：将单条指令字符串（已裁剪，无外围大括号或分号）解析为 [`OpCode`]，并将其追加到 `cmds`。
///
/// Unknown keywords are silently ignored (preserving the original
/// `_ => ()` behaviour).
/// 中文：未知关键字会被静默忽略（保留原始的 `_ => ()` 行为）。
fn parse_instruction(cmd: &str, cmds: &mut Vec<OpCode>) -> Result<()> {
    let cmd_vecs = tokenize(cmd);
    if cmd_vecs.is_empty() {
        return Ok(());
    }

    match cmd_vecs[0] {
        "add" => {
            let first = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
            let second = to_operand(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
            let dest = to_register(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
            cmds.push(OpCode::Add(first, second, dest));
        }
        "sub" => {
            let first = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
            let second = to_operand(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
            let dest = to_register(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
            cmds.push(OpCode::Sub(first, second, dest));
        }
        "mul" => {
            let first = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
            let second = to_operand(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
            let dest = to_register(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
            cmds.push(OpCode::Mul(first, second, dest));
        }
        "div" => {
            let first = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
            let second = to_operand(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
            let dest = to_register(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
            cmds.push(OpCode::Div(first, second, dest));
        }
        "_println" => {
            let value = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
            cmds.push(OpCode::Println(value));
        }
        "_print" => {
            let value = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
            cmds.push(OpCode::Print(value));
        }
        "io" => {
            // Unified I/O keyword. Sub-commands: write, writeln, read, readln.
            // 中文：统一的 I/O 关键字。子命令：write、writeln、read、readln。
            let sub = cmd_vecs.get(1).ok_or(anyhow!(
                "io requires sub-command: write, writeln, read, readln"
            ))?;
            match *sub {
                "write" => {
                    // io write <operand> — output without trailing newline.
                    // 中文：io write <操作数> — 输出，末尾不换行。
                    let value = to_operand(
                        cmd_vecs
                            .get(2)
                            .ok_or(anyhow!("io write requires an operand"))?,
                    )?;
                    cmds.push(OpCode::IO(IoOp::Write(value)));
                }
                "writeln" => {
                    // io writeln <operand> — output with trailing newline.
                    // 中文：io writeln <操作数> — 输出并换行。
                    let value = to_operand(
                        cmd_vecs
                            .get(2)
                            .ok_or(anyhow!("io writeln requires an operand"))?,
                    )?;
                    cmds.push(OpCode::IO(IoOp::Writeln(value)));
                }
                "read" => {
                    // io read <$reg> — read stdin line, parse as Value, store in register.
                    // 中文：io read <$寄存器> — 从标准输入读取一行，解析为 Value，存入寄存器。
                    let reg = to_register(
                        cmd_vecs
                            .get(2)
                            .ok_or(anyhow!("io read requires a register"))?,
                    )?;
                    cmds.push(OpCode::IO(IoOp::Read(reg)));
                }
                "readln" => {
                    // io readln <$reg> — read stdin line, store as String in register.
                    // 中文：io readln <$寄存器> — 从标准输入读取一行，作为 String 存入寄存器。
                    let reg = to_register(
                        cmd_vecs
                            .get(2)
                            .ok_or(anyhow!("io readln requires a register"))?,
                    )?;
                    cmds.push(OpCode::IO(IoOp::Readln(reg)));
                }
                other => return Err(anyhow!("unknown io sub-command: {other}")),
            }
        }
        "let" => {
            // Parse `let [mut] name = expr;`
            // Use the original command string for the expression part so that
            // multi-token expressions like `1 + 2` are preserved in full.
            // 中文：使用原始命令字符串作为表达式部分，以便像 `1 + 2` 这样的多标记表达式被完整保留。
            let rest = cmd[3..].trim_start(); // after "let"
            // 中文："let" 之后的部分
            let mutable = rest.starts_with("mut")
                && (rest.len() == 3 || rest.as_bytes()[3].is_ascii_whitespace());
            let after_kw = if mutable {
                rest[3..].trim_start() // skip "mut"
            // 中文：跳过 "mut"
            } else {
                rest
            };
            let eq_pos = after_kw
                .find('=')
                .ok_or(anyhow!("expected '=' in let statement"))?;
            // 中文：let 语句中应有 '='
            let var_name = after_kw[..eq_pos].trim().to_string();
            if var_name.is_empty() {
                return Err(anyhow!("expected variable name in let statement"));
            }
            let expr_str = after_kw[eq_pos + 1..].trim();
            if expr_str.is_empty() {
                return Err(anyhow!("expected expression after '=' in let statement"));
            }
            // If the expression is a function call, push a Call opcode
            // instead of an Expr, so the return value lands in
            // SystemVarBuffer for the Variable opcode to pick up.
            // 中文：如果表达式是函数调用，则推送 Call 操作码而非 Expr，
            // 使返回值存入 SystemVarBuffer 供 Variable 操作码获取。
            if expr_str.starts_with("call ") || expr_str == "call" {
                push_call_opcode(expr_str, Operand::Register(Register::SystemVarBuffer), cmds)?;
            } else {
                let expr = parse_expr_str(expr_str)?;
                cmds.push(OpCode::Expr(
                    expr,
                    Operand::Register(Register::SystemVarBuffer),
                ));
            }
            cmds.push(OpCode::Variable(
                var_name,
                Variable {
                    is_mut: mutable,
                    value: Value::None,
                },
            ));
        }
        "call" => {
            // Function call: call name(arg1, arg2, ...) [$dest]
            // 中文：函数调用：call 名称(参数1, 参数2, ...) [$dest]
            let dest = Operand::Register(Register::SystemVarBuffer);
            push_call_opcode(cmd, dest, cmds)?;
        }
        "return" => {
            // Return from function: return expr; or return;
            // 中文：从函数返回：return 表达式; 或 return;
            let expr_str = cmd[6..].trim(); // after "return"
            if expr_str.is_empty() {
                cmds.push(OpCode::Return(Expr::Operand(Operand::Literal(Value::None))));
            } else if expr_str.starts_with("call ") || expr_str == "call" {
                // Return the result of a function call.
                // 中文：返回函数调用的结果。
                push_call_opcode(expr_str, Operand::Register(Register::SystemVarBuffer), cmds)?;
            } else {
                let expr = parse_expr_str(expr_str)?;
                cmds.push(OpCode::Return(expr));
            }
        }
        // Assignment with a function call on the RHS: $reg = call name(args)
        // 中文：右侧有函数调用的赋值：$reg = call 名称(参数)
        _ if cmd_vecs.len() >= 4 && cmd_vecs[1] == "=" && cmd_vecs[2] == "call" => {
            let dest = to_operand(cmd_vecs[0])?;
            let eq_pos = cmd.find('=').unwrap();
            let call_text = cmd[eq_pos + 1..].trim();
            push_call_opcode(call_text, dest, cmds)?;
        }
        _ if cmd_vecs.len() >= 3 && cmd_vecs[1] == "=" => {
            let dest = to_operand(cmd_vecs[0])?;
            // Re-tokenize the raw expression text with operator-aware splitting.
            let eq_pos = cmd.find('=').unwrap();
            let expr_str = cmd[eq_pos + 1..].trim();
            let expr = parse_expr_str(expr_str)?;
            cmds.push(OpCode::Expr(expr, dest));
        }
        _ => (), // Ignore unrecognized keywords silently.
                 // 中文：静默忽略无法识别的关键字。
    }
    Ok(())
}

/// Parses the `else` tail of an `if` statement: optionally `else { ... }`
/// or `else if ...`. Returns `None` if there is no else clause, or
/// `Some(body)` with the else-body opcodes. For `else if`, the body is a
/// single-element vec containing a recursive [`OpCode::If`].
/// 中文：解析 `if` 语句的 `else` 尾部：可选的 `else { ... }` 或 `else if ...`。若无 else 子句返回 `None`，否则返回 `Some(body)`。对于 `else if`，body 是包含递归 [`OpCode::If`] 的单元素 vec。
fn parse_else_tail(
    source: &str,
    pos: &mut usize,
    bytes: &[u8],
    len: usize,
) -> Result<Option<Vec<OpCode>>> {
    while *pos < len && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }

    // Check for the word "else" followed by non-alphanumeric boundary.
    // 中文：检查单词 "else" 后面是否跟非字母数字边界。
    if *pos + 4 > len
        || bytes[*pos] != b'e'
        || bytes[*pos + 1] != b'l'
        || bytes[*pos + 2] != b's'
        || bytes[*pos + 3] != b'e'
    {
        return Ok(None);
    }
    let after = *pos + 4;
    if after < len && bytes[after].is_ascii_alphanumeric() {
        // e.g. "elsewhere" — not the keyword
        // 中文：例如 "elsewhere" — 不是关键字
        return Ok(None);
    }
    // consume "else"
    // 中文：消费 "else"
    *pos += 4;

    while *pos < len && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }

    if *pos + 1 < len && bytes[*pos] == b'i' && bytes[*pos + 1] == b'f' {
        // "else if" — parse condition, block, and recurse for further tails.
        // 中文："else if" — 解析条件、块，并递归处理后续尾部。

        // consume "if"
        // 中文：消费 "if"
        *pos += 2;
        while *pos < len && bytes[*pos].is_ascii_whitespace() {
            *pos += 1;
        }
        // Parse condition expression text (until `{`).
        // 中文：解析条件表达式文本（直到 `{`）。
        let cond_start = *pos;
        while *pos < len && bytes[*pos] != b'{' {
            if bytes[*pos] == b'"' {
                *pos += 1;
                while *pos < len {
                    if bytes[*pos] == b'\\' {
                        *pos += 2;
                    } else if bytes[*pos] == b'"' {
                        *pos += 1;
                        break;
                    } else {
                        *pos += 1;
                    }
                }
            } else {
                *pos += 1;
            }
        }
        let cond_str = source[cond_start..*pos].trim();
        let cond = parse_expr_str(cond_str)?;

        if *pos >= len || bytes[*pos] != b'{' {
            return Err(anyhow!("expected '{{' after else if condition"));
        }
        *pos += 1;
        let mut then_body = vec![];
        parse_block(source, pos, &mut then_body)?;

        let else_body = parse_else_tail(source, pos, bytes, len)?;
        Ok(Some(vec![OpCode::If(cond, then_body, else_body)]))
    } else if *pos < len && bytes[*pos] == b'{' {
        *pos += 1;
        let mut body = vec![];
        parse_block(source, pos, &mut body)?;
        Ok(Some(body))
    } else {
        Err(anyhow!("expected '{{' or 'if' after else"))
    }
}

/// Recursive-descent parser for `{ ... }` blocks.
/// 中文：`{ ... }` 块的递归下降解析器。
///
/// Scans `source` from `*pos`, appending [`OpCode`]s to `cmds`.  On `{` it
/// recurses to collect a nested [`OpCode::Block`]; on `}` it returns to the
/// caller.  Instructions inside the block are delimited by `;` as usual.
/// 中文：从 `*pos` 扫描 `source`，将 [`OpCode`] 追加到 `cmds`。遇到 `{` 时递归收集嵌套的 [`OpCode::Block`]；遇到 `}` 时返回调用者。块内指令照常由 `;` 分隔。
fn parse_block(source: &str, pos: &mut usize, cmds: &mut Vec<OpCode>) -> Result<()> {
    let bytes = source.as_bytes();
    let len = bytes.len();

    loop {
        // Skip whitespace (including newlines).
        // 中文：跳过空白字符（包括换行符）。
        while *pos < len && bytes[*pos].is_ascii_whitespace() {
            *pos += 1;
        }
        if *pos >= len {
            break;
        }

        // Line comment `//` — skip to end of line.
        // 中文：行注释 `//` — 跳过直到行尾。
        if *pos + 1 < len && bytes[*pos] == b'/' && bytes[*pos + 1] == b'/' {
            while *pos < len && bytes[*pos] != b'\n' {
                *pos += 1;
            }
            continue;
        }
        // Block comment `/* … */` — skip to closing `*/`.
        // 中文：块注释 `/* … */` — 跳过直到 `*/`。
        if *pos + 1 < len && bytes[*pos] == b'/' && bytes[*pos + 1] == b'*' {
            *pos += 2;
            while *pos + 1 < len {
                if bytes[*pos] == b'*' && bytes[*pos + 1] == b'/' {
                    *pos += 2;
                    break;
                }
                *pos += 1;
            }
            continue;
        }

        match bytes[*pos] {
            b'{' => {
                *pos += 1;
                let mut inner = vec![];
                parse_block(source, pos, &mut inner)?;
                cmds.push(OpCode::Block(inner));
            }
            b'}' => {
                *pos += 1;
                return Ok(());
            }
            b';' => {
                *pos += 1;
                // empty instruction — skip
                // 中文：空指令 — 跳过
            }
            _ => {
                // Accumulate instruction text until a boundary token.
                let start = *pos;
                while *pos < len {
                    match bytes[*pos] {
                        b';' | b'{' | b'}' => break,
                        b'"' => {
                            *pos += 1;
                            while *pos < len {
                                if bytes[*pos] == b'\\' {
                                    *pos += 2;
                                } else if bytes[*pos] == b'"' {
                                    *pos += 1;
                                    break;
                                } else {
                                    *pos += 1;
                                }
                            }
                        }
                        _ => *pos += 1,
                    }
                }
                let text = source[start..*pos].trim();
                if text.is_empty() {
                    if *pos < len && bytes[*pos] == b';' {
                        *pos += 1;
                    }
                    continue;
                }
                if text.starts_with("fn ") || text == "fn" {
                    // `fn name(params) { ... }`
                    // 中文：`fn 名称(参数) { ... }`
                    let rest = if text.len() > 2 {
                        text[2..].trim()
                    } else {
                        return Err(anyhow!("fn requires a name"));
                    };
                    let paren_pos = rest
                        .find('(')
                        .ok_or(anyhow!("expected '(' after function name"))?;
                    let fn_name = rest[..paren_pos].trim().to_string();
                    if fn_name.is_empty() {
                        return Err(anyhow!("expected function name before '('"));
                    }
                    let after_paren = &rest[paren_pos + 1..];
                    let close_paren = after_paren
                        .find(')')
                        .ok_or(anyhow!("expected ')' after parameters"))?;
                    let params_str = after_paren[..close_paren].trim();
                    let params: Vec<String> = if params_str.is_empty() {
                        vec![]
                    } else {
                        params_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    };
                    // Consume `{` for the function body.
                    while *pos < len && bytes[*pos].is_ascii_whitespace() {
                        *pos += 1;
                    }
                    // The text accumulator stopped at `{` because `{` is a boundary.
                    // Advance past the `{`.
                    if *pos < len && bytes[*pos] == b'{' {
                        *pos += 1;
                    } else {
                        return Err(anyhow!("expected '{{' after function parameters"));
                    }
                    let mut body = vec![];
                    parse_block(source, pos, &mut body)?;
                    cmds.push(OpCode::FnDef {
                        name: fn_name,
                        params,
                        body,
                    });
                    // Consume optional trailing `;` after the `}`.
                    while *pos < len && bytes[*pos].is_ascii_whitespace() {
                        *pos += 1;
                    }
                    if *pos < len && bytes[*pos] == b';' {
                        *pos += 1;
                    }
                }
                if text.starts_with("while ") || text == "while" {
                    // `while` condition { ... }
                    let cond_str = if text.len() > 5 {
                        text[5..].trim()
                    } else {
                        return Err(anyhow!("while statement requires a condition"));
                    };
                    let cond = parse_expr_str(cond_str)?;
                    if *pos >= len || bytes[*pos] != b'{' {
                        return Err(anyhow!("expected '{{' after while condition"));
                    }
                    *pos += 1;
                    let mut body = vec![];
                    parse_block(source, pos, &mut body)?;

                    cmds.push(OpCode::While(cond, body));
                    // Consume optional trailing `;` after the `}`.
                    while *pos < len && bytes[*pos].is_ascii_whitespace() {
                        *pos += 1;
                    }
                    if *pos < len && bytes[*pos] == b';' {
                        *pos += 1;
                    }
                }
                if text.starts_with("if ") || text == "if" {
                    // `if` condition { ... } [else { ... } | else if ...]
                    let cond_str = if text.len() > 2 {
                        text[2..].trim()
                    } else {
                        return Err(anyhow!("if statement requires a condition"));
                    };
                    let cond = parse_expr_str(cond_str)?;

                    if *pos >= len || bytes[*pos] != b'{' {
                        return Err(anyhow!("expected '{{' after if condition"));
                    }
                    *pos += 1;
                    let mut then_body = vec![];
                    parse_block(source, pos, &mut then_body)?;

                    let else_body = parse_else_tail(source, pos, bytes, len)?;
                    cmds.push(OpCode::If(cond, then_body, else_body));

                    // Consume optional trailing `;` after the `}`.
                    while *pos < len && bytes[*pos].is_ascii_whitespace() {
                        *pos += 1;
                    }
                    if *pos < len && bytes[*pos] == b';' {
                        *pos += 1;
                    }
                } else {
                    parse_instruction(text, cmds)?;
                    // Consume the trailing `;` if that was the boundary.
                    if *pos < len && bytes[*pos] == b';' {
                        *pos += 1;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Parses a complete `.ralr` source string into a [`Vec<OpCode>`].
/// 中文：将完整的 `.ralr` 源字符串解析为 [`Vec<OpCode>`]。
///
/// This is the public entry point for the assembler's parser.
/// 中文：这是汇编器解析器的公共入口点。
pub fn parse(source: &str) -> Result<Vec<OpCode>> {
    let mut cmds = vec![];
    parse_block(source, &mut 0, &mut cmds)?;
    Ok(cmds)
}
