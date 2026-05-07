//! Expression parser — tokenizer and Pratt precedence-climbing parser.
//! 中文：表达式解析器 — 分词器和 Pratt 优先级爬升解析器。
//! 日本語：式パーサー — トークナイザと Pratt 優先順位登山パーサー。
//! Русский: Парсер выражений — токенизатор и парсер Пратта с подъёмом по приоритетам.

use anyhow::{Result, anyhow};
use vm_isa::{
    op_code::{BinOp, Expr, UnOp},
    value::Value,
};

use crate::reader::{to_register, to_value};

/// Tries to negate a literal value at compile time.
/// 中文：尝试在编译时对字面量取反。
/// 日本語：コンパイル時にリテラル値の否定を試みます。
/// Русский: Пытается инвертировать литеральное значение во время компиляции.
///
/// Converts unsigned/signed integers to their negated signed form so
/// that `-128` becomes `I32(-128)` rather than `Unary(Neg, U8(128))`
/// which would panic at runtime.
/// 中文：将无符号/有符号整数转换为取反后的有符号形式，使得 `-128` 变为 `I32(-128)` 而不是 `Unary(Neg, U8(128))`，后者会在运行时 panic。
/// 日本語：符号なし/符号付き整数を否定された符号付き形式に変換し、`-128` が実行時にパニックする `Unary(Neg, U8(128))` ではなく `I32(-128)` になるようにします。
/// Русский: Преобразует беззнаковые/знаковые целые в их отрицательную знаковую форму,
/// чтобы `-128` стало `I32(-128)`, а не `Unary(Neg, U8(128))`, которое вызвало бы панику во время выполнения.
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
// 日本語：式トークナイザ
// Русский: Токенизатор выражений

/// Tokenizes an expression substring, splitting on whitespace and
/// operator boundaries so that `!true`, `-5`, `(1+2)` etc. are
/// properly separated into individual tokens.
/// 中文：对表达式子串进行标记化，在空白和运算符边界处拆分，使 `!true`、`-5`、`(1+2)` 等正确分离为单独的标记。
/// 日本語：式の部分文字列をトークン化し、空白と演算子の境界で分割して、`!true`、`-5`、`(1+2)` などが適切に個別のトークンに分離されるようにします。
/// Русский: Токенизирует подстроку выражения, разделяя по пробелам и границам
/// операторов, чтобы `!true`, `-5`, `(1+2)` и т.д. корректно разделялись на отдельные токены.
fn tokenize_expr(s: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // Two-character operators (checked before single-char).
    // 中文：双字符运算符（在单字符之前检查）。
    // 日本語：2文字演算子（1文字より先にチェック）。
    // Русский: Двухсимвольные операторы (проверяются перед односимвольными).
    const MULTI_CHAR: &[&[u8]] = &[b"==", b"!=", b"<=", b">=", b"&&", b"||", b"<<", b">>"];
    // Single-character boundary tokens.
    // 中文：单字符边界标记。
    // 日本語：1文字の境界トークン。
    // Русский: Односимвольные граничные токены.
    const SINGLE_CHAR: &[u8] = b"+-*/%<>&|^!~()=";

    while i < len {
        // Skip whitespace.
        // 中文：跳过空白。
        // 日本語：空白をスキップ。
        // Русский: Пропустить пробелы.
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        // Check for two-character operators.
        // 中文：检查双字符运算符。
        // 日本語：2文字演算子をチェック。
        // Русский: Проверить двухсимвольные операторы.
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
        // 日本語：1文字の境界トークンをチェック。
        // Русский: Проверить односимвольные граничные токены.
        if SINGLE_CHAR.contains(&bytes[i]) {
            tokens.push(&s[i..i + 1]);
            i += 1;
            continue;
        }

        // Quoted string.
        // 中文：带引号的字符串。
        // 日本語：引用符付き文字列。
        // Русский: Строка в кавычках.
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
        // 日本語：識別子 / 数値 / レジスタを蓄積。
        // Русский: Накопить идентификатор / число / регистр.
        let start = i;
        while i < len && !bytes[i].is_ascii_whitespace() && !SINGLE_CHAR.contains(&bytes[i]) {
            // Stop before multi-char operators too.
            // 中文：同样在遇到双字符运算符前停止。
            // 日本語：マルチ文字演算子の前でも停止。
            // Русский: Остановиться также перед многосимвольными операторами.
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
// 日本語：式パーサー
// Русский: Парсер выражений

/// Maps an infix operator token to its [`BinOp`] and precedence level.
/// 中文：将中缀运算符标记映射到其 [`BinOp`] 和优先级。
/// 日本語：中置演算子トークンをその [`BinOp`] と優先順位にマッピングします。
/// Русский: Сопоставляет токен инфиксного оператора с его [`BinOp`] и уровнем приоритета.
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
/// 日本語：分割済みトークンからアトムまたは単項プレフィックス式を解析します。
/// Русский: Парсит атом или унарное префиксное выражение из уже разделённых токенов.
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
            // 日本語：定数畳み込み：オペランドが符号なしリテラルの場合、直接符号付き負数に変換。
            // Русский: Свёртка констант: если операнд — беззнаковый литерал,
            // преобразовать напрямую в отрицательное знаковое значение.
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
/// 日本語：Pratt / 優先順位登山式パーサー。
/// Русский: Парсер выражений Пратта / с подъёмом по приоритетам.
///
/// Parses tokens starting at `pos` until an operator with precedence
/// lower than `min_prec` is encountered. Returns the [`Expr`] and the
/// number of tokens consumed from `pos`.
/// 中文：从 `pos` 开始解析标记，直到遇到优先级低于 `min_prec` 的运算符。返回 [`Expr`] 和从 `pos` 开始消费的标记数量。
/// 日本語：`pos` からトークンを解析し、`min_prec` より低い優先順位の演算子に遭遇するまで続けます。[`Expr`] と `pos` から消費されたトークン数を返します。
/// Русский: Парсит токены начиная с `pos`, пока не встретит оператор с приоритетом
/// ниже `min_prec`. Возвращает [`Expr`] и количество потреблённых от `pos` токенов.
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
/// 日本語：生の式文字列を [`Expr`] ツリーに解析します。
/// Русский: Парсит сырую строку выражения в дерево [`Expr`].
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
