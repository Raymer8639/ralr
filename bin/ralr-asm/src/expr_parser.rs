//! Expression parser — tokenizer and Pratt precedence-climbing parser.
//! 中文：表达式解析器 — 分词器和 Pratt 优先级爬升解析器。

use anyhow::{Result, anyhow};
use vm_isa::{
    op_code::{BinOp, Expr, UnOp},
    value::Value,
};

use crate::reader::{to_register, to_value};

/// Tries to negate a literal value at compile time.
/// 中文：尝试在编译时对字面量取反。
///
/// Converts unsigned/signed integers to their negated signed form so
/// that `-128` becomes `I32(-128)` rather than `Unary(Neg, U8(128))`
/// which would panic at runtime.
/// 中文：将无符号/有符号整数转换为取反后的有符号形式，使得 `-128` 变为 `I32(-128)` 而不是 `Unary(Neg, U8(128))`，后者会在运行时 panic。
fn try_negate_value(v: &Value) -> Option<Value> {
    match v {
        Value::U8(n) => Some(Value::I32(-(*n as i32))),
        Value::U32(n) => Some(Value::I128(-(*n as i128))),
        Value::U128(n) => Some(Value::I128(-(*n as i128))),
        Value::I32(n) => Some(Value::I32(-n)),
        Value::I128(n) => Some(Value::I128(-n)),
        Value::F32(n) => Some(Value::F32(-n)),
        Value::F64(n) => Some(Value::F64(-n)),
        _ => None,
    }
}

// ── Expression tokenizer ─────────────────────────────────────────────
// 中文：表达式分词器

/// Tokenizes an expression substring, splitting on whitespace and
/// operator boundaries so that `!true`, `-5`, `(1+2)` etc. are
/// properly separated into individual tokens.
/// 中文：对表达式子串进行标记化，在空白和运算符边界处拆分，使 `!true`、`-5`、`(1+2)` 等正确分离为单独的标记。
fn tokenize_expr(s: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // Two-character operators (checked before single-char).
    // 中文：双字符运算符（在单字符之前检查）。
    const MULTI_CHAR: &[&[u8]] = &[b"==", b"!=", b"<=", b">=", b"&&", b"||", b"<<", b">>"];
    // Single-character boundary tokens.
    // 中文：单字符边界标记。
    const SINGLE_CHAR: &[u8] = b"+-*/%<>&|^!~()=";

    while i < len {
        // Skip whitespace.
        // 中文：跳过空白。
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        // Check for two-character operators.
        // 中文：检查双字符运算符。
        let mut matched_multi = false;
        if i + 1 < len {
            for op in MULTI_CHAR {
                if bytes[i] == op[0] && bytes[i + 1] == op[1] {
                    tokens.push(&s[i..i + 2]);
                    i += 2;
                    matched_multi = true;
                    break;
                }
            }
        }
        if matched_multi {
            continue;
        }

        // Check for single-character boundary tokens.
        // 中文：检查单字符边界标记。
        if SINGLE_CHAR.contains(&bytes[i]) {
            tokens.push(&s[i..i + 1]);
            i += 1;
            continue;
        }

        // Quoted string.
        // 中文：带引号的字符串。
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < len {
                if bytes[i] == b'\\' {
                    i += 2;
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            tokens.push(&s[start..i]);
            continue;
        }

        // Accumulate identifier / number / register.
        // 中文：累积标识符 / 数字 / 寄存器。
        let start = i;
        while i < len && !bytes[i].is_ascii_whitespace() && !SINGLE_CHAR.contains(&bytes[i]) {
            // Stop before multi-char operators too.
            // 中文：同样在遇到双字符运算符前停止。
            if i + 1 < len {
                let mut is_op = false;
                for op in MULTI_CHAR {
                    if bytes[i] == op[0] && bytes[i + 1] == op[1] {
                        is_op = true;
                        break;
                    }
                }
                if is_op {
                    break;
                }
            }
            i += 1;
        }
        if i > start {
            tokens.push(&s[start..i]);
        }
    }
    tokens
}

// ── Expression parser ────────────────────────────────────────────────
// 中文：表达式解析器

/// Maps an infix operator token to its [`BinOp`] and precedence level.
/// 中文：将中缀运算符标记映射到其 [`BinOp`] 和优先级。
fn infix_binding_power(op: &str) -> Option<(BinOp, u8)> {
    match op {
        "||" => Some((BinOp::Or, 2)),
        "&&" => Some((BinOp::And, 3)),
        "|" => Some((BinOp::BitOr, 4)),
        "^" => Some((BinOp::BitXor, 5)),
        "&" => Some((BinOp::BitAnd, 6)),
        "==" => Some((BinOp::Eq, 7)),
        "!=" => Some((BinOp::Ne, 7)),
        "<" => Some((BinOp::Lt, 8)),
        ">" => Some((BinOp::Gt, 8)),
        "<=" => Some((BinOp::Le, 8)),
        ">=" => Some((BinOp::Ge, 8)),
        "<<" => Some((BinOp::Shl, 9)),
        ">>" => Some((BinOp::Shr, 9)),
        "+" => Some((BinOp::Add, 10)),
        "-" => Some((BinOp::Sub, 10)),
        "*" => Some((BinOp::Mul, 11)),
        "/" => Some((BinOp::Div, 11)),
        "%" => Some((BinOp::Rem, 11)),
        _ => None,
    }
}

/// Parses an atom or unary prefix expression from already-split tokens.
/// 中文：从已拆分的标记中解析原子或一元前缀表达式。
fn parse_prefix(tokens: &[&str], pos: usize) -> Result<(Expr, usize)> {
    if pos >= tokens.len() {
        return Err(anyhow!("unexpected end of expression"));
    }
    let tok = tokens[pos];
    match tok {
        "(" => {
            let (inner, consumed) = parse_expression(tokens, pos + 1, 0)?;
            let next = pos + 1 + consumed;
            if next >= tokens.len() || tokens[next] != ")" {
                return Err(anyhow!("expected closing ')'"));
            }
            Ok((inner, consumed + 2))
        }
        "-" => {
            let (inner, consumed) = parse_expression(tokens, pos + 1, 12)?;
            // Constant-fold: if the operand is an unsigned literal,
            // convert it directly to a negative signed value so that
            // `-128` becomes `I32(-128)` instead of panicking on Neg<U8>.
            // 中文：常量折叠：如果操作数是无符号字面量，直接转换为有符号负数。
            if let Expr::Literal(ref v) = inner
                && let Some(folded) = try_negate_value(v)
            {
                return Ok((Expr::Literal(folded), consumed + 1));
            }
            Ok((Expr::Unary(UnOp::Neg, Box::new(inner)), consumed + 1))
        }
        "!" => {
            let (inner, consumed) = parse_expression(tokens, pos + 1, 12)?;
            Ok((Expr::Unary(UnOp::Not, Box::new(inner)), consumed + 1))
        }
        "~" => {
            let (inner, consumed) = parse_expression(tokens, pos + 1, 12)?;
            Ok((Expr::Unary(UnOp::BitNot, Box::new(inner)), consumed + 1))
        }
        s if s.starts_with('$') => Ok((Expr::Register(to_register(s)?), 1)),
        _ => Ok((Expr::Literal(to_value(tok)?), 1)),
    }
}

/// Pratt / precedence-climbing expression parser.
/// 中文：Pratt / 优先级爬升表达式解析器。
///
/// Parses tokens starting at `pos` until an operator with precedence
/// lower than `min_prec` is encountered. Returns the [`Expr`] and the
/// number of tokens consumed from `pos`.
/// 中文：从 `pos` 开始解析标记，直到遇到优先级低于 `min_prec` 的运算符。返回 [`Expr`] 和从 `pos` 开始消费的标记数量。
fn parse_expression(tokens: &[&str], pos: usize, min_prec: u8) -> Result<(Expr, usize)> {
    let (mut lhs, mut consumed) = parse_prefix(tokens, pos)?;

    loop {
        let next = pos + consumed;
        if next >= tokens.len() {
            break;
        }
        let (op, prec) = match infix_binding_power(tokens[next]) {
            Some(v) => v,
            None => break,
        };
        if prec < min_prec {
            break;
        }
        let (rhs, rhs_consumed) = parse_expression(tokens, next + 1, prec + 1)?;
        lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
        consumed += 1 + rhs_consumed;
    }

    Ok((lhs, consumed))
}

/// Parses a raw expression string into an [`Expr`] tree.
/// 中文：将原始表达式字符串解析为 [`Expr`] 树。
pub fn parse_expr_str(s: &str) -> Result<Expr> {
    let tokens = tokenize_expr(s);
    if tokens.is_empty() {
        return Err(anyhow!("empty expression"));
    }
    let (expr, consumed) = parse_expression(&tokens, 0, 0)?;
    if consumed != tokens.len() {
        return Err(anyhow!(
            "unexpected token in expression: '{}'",
            tokens[consumed]
        ));
    }
    Ok(expr)
}
