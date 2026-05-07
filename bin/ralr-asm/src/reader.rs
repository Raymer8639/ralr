//! Parses `.ralr` source text into [`OpCode`] instructions.
//! 中文：将 `.ralr` 源文本解析为 [`OpCode`] 指令。
//! 日本語：`.ralr` ソーステキストを [`OpCode`] 命令に解析します。
//! Русский: Парсит исходный текст `.ralr` в инструкции [`OpCode`].
//!
//! Instructions are semicolon-delimited. Each instruction consists of a
//! keyword followed by whitespace-separated operands. Quoted strings are
//! kept as single tokens even when they contain spaces.
//! 中文：指令由分号分隔。每条指令由一个关键字和空格分隔的操作数组成。带引号的字符串即使包含空格也保留为单个标记。
//! 日本語：命令はセミコロンで区切られます。各命令はキーワードとスペース区切りのオペランドで構成されます。引用符で囲まれた文字列はスペースを含んでいても単一のトークンとして保持されます。
//! Русский: Инструкции разделены точкой с запятой. Каждая инструкция состоит из
//! ключевого слова и операндов, разделённых пробелами. Строки в кавычках сохраняются
//! как единые токены, даже если содержат пробелы.

use anyhow::{Result, anyhow};
use std::char;
use vm_isa::{
    op_code::{OpCode, Operand},
    register::Register,
    value::Value,
};

use crate::expr_parser::parse_expr_str;

/// Tries each numeric parse in order, returning the first success.
/// 中文：依次尝试每种数字解析，返回第一个成功的结果。
/// 日本語：各数値解析を順に試行し、最初に成功したものを返します。
/// Русский: Пробует каждый числовой парсинг по порядку, возвращая первый успешный.
///
/// The chain is ordered from narrowest to widest type so that a number
/// like `42` becomes `U8` rather than `I32` when possible.
/// 中文：链按从窄到宽的类型排列，以便像 `42` 这样的数字在可能的情况下成为 `U8` 而不是 `I32`。
/// 日本語：チェーンは狭い型から広い型の順に並んでいるため、`42` のような数値は可能であれば `I32` ではなく `U8` になります。
/// Русский: Цепочка упорядочена от самого узкого к самому широкому типу, так что
/// число вроде `42` становится `U8`, а не `I32`, когда это возможно.
macro_rules! try_parse {
    ($s:expr, $($ty:ty => $var:ident),* $(,)?) => {
        $(if let Ok(v) = $s.parse::<$ty>() { return Ok(Value::$var(v)); })*
    };
}

/// Processes backslash escape sequences in a string literal.
/// 中文：处理字符串字面量中的反斜杠转义序列。
/// 日本語：文字列リテラル内のバックスラッシュエスケープシーケンスを処理します。
/// Русский: Обрабатывает escape-последовательности с обратной косой чертой в строковом литерале.
///
/// Supports: `\n` `\t` `\r` `\\` `\"` `\0` `\xNN` `\u{NNNN}`.
/// 中文：支持：`\n` `\t` `\r` `\\` `\"` `\0` `\xNN` `\u{NNNN}`。
/// 日本語：対応：`\n` `\t` `\r` `\\` `\"` `\0` `\xNN` `\u{NNNN}`。
/// Русский: Поддерживает: `\n` `\t` `\r` `\\` `\"` `\0` `\xNN` `\u{NNNN}`.
///
/// Returns `None` on any unrecognized or malformed escape.
/// 中文：对于任何无法识别或格式错误的转义，返回 `None`。
/// 日本語：認識できないまたは不正な形式のエスケープでは `None` を返します。
/// Русский: Возвращает `None` при любой нераспознанной или некорректной escape-последовательности.
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
            '0' => out.push('\0'),
            'x' => {
                let hi = chars.next()?.to_digit(16)?;
                let lo = chars.next()?.to_digit(16)?;
                out.push(((hi * 16 + lo) as u8) as char);
            }
            'u' => {
                // Rust-style Unicode escape: \u{XXXX}
                // 中文：Rust 风格的 Unicode 转义：\u{XXXX}
                // 日本語：Rust スタイルの Unicode エスケープ：\u{XXXX}
                // Русский: Unicode-escape в стиле Rust: \u{XXXX}
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
/// 日本語：トークンをリテラル [`Value`] に変換します。
/// Русский: Преобразует токен в литеральное [`Value`].
///
/// Handles quoted strings (via [`unescape_str`]), booleans, and numeric
/// literals (via the [`try_parse!`] chain). Numeric inference follows the
/// chain order: U8 → U32 → U128 → I32 → I128 → F32 → F64.
/// 中文：处理带引号的字符串（通过 [`unescape_str`]）、布尔值和数字字面量（通过 [`try_parse!`] 链）。数字推断遵循链顺序：U8 → U32 → U128 → I32 → I128 → F32 → F64。
/// 日本語：引用符付き文字列（[`unescape_str`] 経由）、ブール値、数値リテラル（[`try_parse!`] チェーン経由）を処理します。数値推論はチェーン順：U8 → U32 → U128 → I32 → I128 → F32 → F64 に従います。
/// Русский: Обрабатывает строки в кавычках (через [`unescape_str`]), булевы значения и числовые
/// литералы (через цепочку [`try_parse!`]). Числовой вывод следует порядку цепочки:
/// U8 → U32 → U128 → I32 → I128 → F32 → F64.
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
/// 日本語：`$aN` トークンを [`Register`] 判別式に変換します。
/// Русский: Преобразует токен `$aN` в дискриминант [`Register`].
///
/// # Panics
///
/// Panics if the token does not start with `$` (should only be called
/// after checking the prefix) or names an unknown register.
/// 中文：如果标记不以 `$` 开头（应仅在检查前缀后调用）或命名了未知寄存器，则会 panic。
/// 日本語：トークンが `$` で始まらない場合（プレフィックスを確認した後にのみ呼び出す必要があります）または未知のレジスタ名の場合にパニックします。
/// Русский: Паникует, если токен не начинается с `$` (должен вызываться только после
/// проверки префикса) или называет неизвестный регистр.
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
/// 日本語：トークンが `$` で始まる場合は [`Operand::Register`] に、それ以外は [`Operand::Literal`] にディスパッチします。
/// Русский: Направляет токен в [`Operand::Register`], если он начинается с `$`,
/// иначе в [`Operand::Literal`].
fn to_operand(str: &str) -> Result<Operand> {
    if str.starts_with('$') {
        Ok(Operand::Register(to_register(str)?))
    } else {
        Ok(Operand::Literal(to_value(str)?))
    }
}

/// Splits a command string into tokens on whitespace while keeping
/// quoted strings intact.
/// 中文：在空白字符处将命令字符串拆分为标记，同时保持带引号的字符串完整。
/// 日本語：空白でコマンド文字列をトークンに分割しつつ、引用符付き文字列はそのまま保持します。
/// Русский: Разбивает командную строку на токены по пробелам, сохраняя строки в кавычках нетронутыми.
///
/// Uses byte-level indexing, which is safe because `"` (0x22) and `\`
/// (0x5C) never appear inside UTF-8 multi-byte sequences.
/// 中文：使用字节级索引，这是安全的，因为 `"` (0x22) 和 `\` (0x5C) 永远不会出现在 UTF-8 多字节序列中。
/// 日本語：バイトレベルのインデックスを使用しますが、`"` (0x22) と `\` (0x5C) は UTF-8 マルチバイトシーケンス内には決して現れないため安全です。
/// Русский: Использует индексацию на уровне байтов, что безопасно, так как `"` (0x22) и `\`
/// (0x5C) никогда не появляются внутри многобайтовых последовательностей UTF-8.
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
                    i += 2; // skip escape sequence (backslash + escaped char)
                            // 中文：跳过转义序列（反斜杠 + 转义字符）
                            // 日本語：エスケープシーケンスをスキップ（バックスラッシュ + エスケープ文字）
                            // Русский: пропустить escape-последовательность (обратная косая + экранированный символ)
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

/// Parses a single instruction string (already trimmed, no surrounding
/// braces or semicolons) into an [`OpCode`], appending it to `cmds`.
/// 中文：将单条指令字符串（已裁剪，无外围大括号或分号）解析为 [`OpCode`]，并将其追加到 `cmds`。
/// 日本語：単一の命令文字列（すでにトリミング済みで、囲み括弧やセミコロンなし）を [`OpCode`] に解析し、`cmds` に追加します。
/// Русский: Парсит одну строку инструкции (уже обрезанную, без окружающих фигурных скобок
/// или точек с запятой) в [`OpCode`], добавляя в `cmds`.
///
/// Unknown keywords are silently ignored (preserving the original
/// `_ => ()` behaviour).
/// 中文：未知关键字会被静默忽略（保留原始的 `_ => ()` 行为）。
/// 日本語：未知のキーワードは暗黙的に無視されます（元の `_ => ()` の動作を維持）。
/// Русский: Неизвестные ключевые слова молча игнорируются (сохраняя исходное поведение `_ => ()`).
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
        _ if cmd_vecs.len() >= 3
            && cmd_vecs[0].starts_with('$')
            && cmd_vecs[1] == "=" =>
        {
            let dest = to_register(cmd_vecs[0])?;
            // Re-tokenize the raw expression text with operator-aware splitting.
            let eq_pos = cmd.find('=').unwrap();
            let expr_str = cmd[eq_pos + 1..].trim();
            let expr = parse_expr_str(expr_str)?;
            cmds.push(OpCode::Expr(expr, dest));
        }
        _ => (), // Ignore unrecognized keywords silently.
                 // 中文：静默忽略无法识别的关键字。
                 // 日本語：認識できないキーワードは暗黙的に無視。
                 // Русский: Молча игнорировать нераспознанные ключевые слова.
    }
    Ok(())
}

/// Recursive-descent parser for `{ ... }` blocks.
/// 中文：`{ ... }` 块的递归下降解析器。
/// 日本語：`{ ... }` ブロックの再帰下降パーサー。
/// Русский: Рекурсивный нисходящий парсер для блоков `{ ... }`.
///
/// Scans `source` from `*pos`, appending [`OpCode`]s to `cmds`.  On `{` it
/// recurses to collect a nested [`OpCode::Block`]; on `}` it returns to the
/// caller.  Instructions inside the block are delimited by `;` as usual.
/// 中文：从 `*pos` 扫描 `source`，将 [`OpCode`] 追加到 `cmds`。遇到 `{` 时递归收集嵌套的 [`OpCode::Block`]；遇到 `}` 时返回调用者。块内指令照常由 `;` 分隔。
/// 日本語：`*pos` から `source` をスキャンし、[`OpCode`] を `cmds` に追加します。`{` では再帰してネストされた [`OpCode::Block`] を収集し、`}` では呼び出し元に戻ります。ブロック内の命令は通常通り `;` で区切られます。
/// Русский: Сканирует `source` начиная с `*pos`, добавляя [`OpCode`] в `cmds`. При `{`
/// рекурсивно собирает вложенный [`OpCode::Block`]; при `}` возвращается к вызывающей стороне.
/// Инструкции внутри блока разделены `;` как обычно.
fn parse_block(source: &str, pos: &mut usize, cmds: &mut Vec<OpCode>) -> Result<()> {
    let bytes = source.as_bytes();
    let len = bytes.len();

    loop {
        // Skip whitespace (including newlines).
        // 中文：跳过空白字符（包括换行符）。
        // 日本語：空白（改行含む）をスキップします。
        // Русский: Пропустить пробельные символы (включая переводы строк).
        while *pos < len && bytes[*pos].is_ascii_whitespace() {
            *pos += 1;
        }
        if *pos >= len {
            break;
        }

        // Line comment `//` — skip to end of line.
        // 中文：行注释 `//` — 跳过直到行尾。
        // 日本語：行コメント `//` — 行末までスキップ。
        // Русский: Строчный комментарий `//` — пропустить до конца строки.
        if *pos + 1 < len && bytes[*pos] == b'/' && bytes[*pos + 1] == b'/' {
            while *pos < len && bytes[*pos] != b'\n' {
                *pos += 1;
            }
            continue;
        }
        // Block comment `/* … */` — skip to closing `*/`.
        // 中文：块注释 `/* … */` — 跳过直到 `*/`。
        // 日本語：ブロックコメント `/* … */` — `*/` までスキップ。
        // Русский: Блочный комментарий `/* … */` — пропустить до `*/`.
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
                // 日本語：空の命令 — スキップ
                // Русский: пустая инструкция — пропустить
            }
            _ => {
                // Accumulate instruction text until a boundary token.
                // 中文：累积指令文本直到遇到边界标记。
                // 日本語：境界トークンまで命令テキストを蓄積します。
                // Русский: Накопить текст инструкции до граничного токена.
                let start = *pos;
                while *pos < len {
                    match bytes[*pos] {
                        b';' | b'{' | b'}' => break,
                        b'"' => {
                            // Skip the whole string literal so that
                            // boundary characters inside it are kept.
                            // 中文：跳过整个字符串字面量，以便保留其中的边界字符。
                            // 日本語：文字列リテラル全体をスキップして、内部の境界文字が保持されるようにします。
                            // Русский: Пропустить весь строковый литерал, чтобы
                            // граничные символы внутри него сохранились.
                            *pos += 1; // opening quote
                                       // 中文：开始引号
                                       // 日本語：開始引用符
                                       // Русский: открывающая кавычка
                            while *pos < len {
                                if bytes[*pos] == b'\\' {
                                    *pos += 2; // skip escape sequence
                                                // 中文：跳过转义序列
                                                // 日本語：エスケープシーケンスをスキップ
                                                // Русский: пропустить escape-последовательность
                                } else if bytes[*pos] == b'"' {
                                    *pos += 1; // closing quote
                                                // 中文：结束引号
                                                // 日本語：終了引用符
                                                // Русский: закрывающая кавычка
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
                if !text.is_empty() {
                    parse_instruction(text, cmds)?;
                }
                // Consume the trailing `;` if that was the boundary.
                // 中文：如果边界是尾随的 `;`，则消费它。
                // 日本語：境界が末尾の `;` だった場合、それを消費します。
                // Русский: Поглотить завершающую `;`, если она была границей.
                if *pos < len && bytes[*pos] == b';' {
                    *pos += 1;
                }
            }
        }
    }
    Ok(())
}

/// Parses a complete `.ralr` source string into a [`Vec<OpCode>`].
/// 中文：将完整的 `.ralr` 源字符串解析为 [`Vec<OpCode>`]。
/// 日本語：完全な `.ralr` ソース文字列を [`Vec<OpCode>`] に解析します。
/// Русский: Парсит полную строку исходного кода `.ralr` в [`Vec<OpCode>`].
///
/// This is the public entry point for the assembler's parser.
/// 中文：这是汇编器解析器的公共入口点。
/// 日本語：これはアセンブラのパーサーの公開エントリポイントです。
/// Русский: Это публичная точка входа для парсера ассемблера.
pub fn parse(source: &str) -> Result<Vec<OpCode>> {
    let mut cmds = vec![];
    parse_block(source, &mut 0, &mut cmds)?;
    Ok(cmds)
}
