//! Tests for expression parsing, evaluation, and serialization.
//! 中文：表达式解析、求值和序列化测试。
//! 日本語：式の解析、評価、シリアライズのテスト。
//! Русский: Тесты парсинга, вычисления и сериализации выражений.

use ralr_asm::reader;
use vm_isa::op_code::{BinOp, Expr, OpCode, UnOp};
use vm_isa::register::{Register, Registers};
use vm_isa::value::Value;

// ── Value operation tests ────────────────────────────────────────────
// 中文：Value 运算测试
// 日本語：Value 演算テスト
// Русский: Тесты операций Value

#[test]
fn rem_i32() {
    let v = Value::I32(10) % Value::I32(3);
    assert_eq!(v, Value::I32(1));
}

#[test]
fn bitand_u8() {
    let v = Value::U8(0xFF) & Value::U8(0x0F);
    assert_eq!(v, Value::U8(0x0F));
}

#[test]
fn bitor_u32() {
    let v = Value::U32(0xF0) | Value::U32(0x0F);
    assert_eq!(v, Value::U32(0xFF));
}

#[test]
fn bitxor_i32() {
    let v = Value::I32(0b1100) ^ Value::I32(0b1010);
    assert_eq!(v, Value::I32(0b0110));
}

#[test]
fn shl_u8() {
    let v = Value::U8(1) << Value::U8(3);
    assert_eq!(v, Value::U8(8));
}

#[test]
fn shr_i32() {
    let v = Value::I32(16) >> Value::I32(2);
    assert_eq!(v, Value::I32(4));
}

#[test]
fn shl_wrapping() {
    // wrapping_shl masks the shift amount: 8 & 7 = 0, so 1 << 0 = 1
    let v = Value::U8(1) << Value::U8(8);
    assert_eq!(v, Value::U8(1));
    // Shift by 7 (max for u8) = 128
    let v = Value::U8(1) << Value::U8(7);
    assert_eq!(v, Value::U8(128));
}

#[test]
fn neg_i32() {
    let v = -Value::I32(5);
    assert_eq!(v, Value::I32(-5));
}

#[test]
fn neg_f64() {
    let v = -Value::F64(3.14);
    assert!(matches!(v, Value::F64(x) if (x + 3.14).abs() < f64::EPSILON));
}

#[test]
#[should_panic]
fn neg_unsigned_panics() {
    let _ = -Value::U8(1);
}

#[test]
fn eq_val_true() {
    let v = Value::I32(5).eq_val(&Value::I32(5));
    assert_eq!(v, Value::Bool(true));
}

#[test]
fn eq_val_false() {
    let v = Value::I32(5).eq_val(&Value::I32(3));
    assert_eq!(v, Value::Bool(false));
}

#[test]
fn ne_val() {
    assert_eq!(Value::U8(1).ne_val(&Value::U8(2)), Value::Bool(true));
    assert_eq!(Value::U8(1).ne_val(&Value::U8(1)), Value::Bool(false));
}

#[test]
fn lt_val() {
    assert_eq!(Value::I32(1).lt_val(&Value::I32(2)), Value::Bool(true));
    assert_eq!(Value::I32(2).lt_val(&Value::I32(1)), Value::Bool(false));
}

#[test]
fn le_val_equal() {
    assert_eq!(Value::F64(1.0).le_val(&Value::F64(1.0)), Value::Bool(true));
}

#[test]
fn gt_val() {
    assert_eq!(Value::U32(10).gt_val(&Value::U32(5)), Value::Bool(true));
}

#[test]
fn ge_val() {
    assert_eq!(Value::I128(5).ge_val(&Value::I128(5)), Value::Bool(true));
    assert_eq!(Value::I128(4).ge_val(&Value::I128(5)), Value::Bool(false));
}

#[test]
fn logical_not_true() {
    assert_eq!(Value::Bool(true).logical_not(), Value::Bool(false));
}

#[test]
fn logical_not_false() {
    assert_eq!(Value::Bool(false).logical_not(), Value::Bool(true));
}

#[test]
#[should_panic]
fn logical_not_on_int_panics() {
    Value::I32(1).logical_not();
}

#[test]
fn bitwise_not() {
    assert_eq!(Value::U8(0x0F).bitwise_not(), Value::U8(0xF0));
    assert_eq!(Value::I32(0).bitwise_not(), Value::I32(-1));
}

#[test]
#[should_panic]
fn bitwise_not_on_bool_panics() {
    Value::Bool(true).bitwise_not();
}

#[test]
fn and_val() {
    assert_eq!(
        Value::Bool(true).and_val(&Value::Bool(false)),
        Value::Bool(false)
    );
    assert_eq!(
        Value::Bool(true).and_val(&Value::Bool(true)),
        Value::Bool(true)
    );
}

#[test]
fn or_val() {
    assert_eq!(
        Value::Bool(false).or_val(&Value::Bool(false)),
        Value::Bool(false)
    );
    assert_eq!(
        Value::Bool(true).or_val(&Value::Bool(false)),
        Value::Bool(true)
    );
}

#[test]
#[should_panic]
fn and_val_on_int_panics() {
    Value::I32(1).and_val(&Value::I32(2));
}

// ── Parser tests ──────────────────────────────────────────────────
// 中文：解析器测试
// 日本語：パーサーテスト
// Русский: Тесты парсера

#[test]
fn parse_simple_literal_assignment() {
    let ops = reader::parse("$a1 = 42;").unwrap();
    assert_eq!(ops.len(), 1);
    assert!(matches!(&ops[0], OpCode::Expr(Expr::Literal(Value::U8(42)), Register::A1)));
}

#[test]
fn parse_register_assignment() {
    let ops = reader::parse("$a2 = $a1;").unwrap();
    assert_eq!(ops.len(), 1);
    assert!(matches!(&ops[0], OpCode::Expr(Expr::Register(Register::A1), Register::A2)));
}

#[test]
fn parse_binary_with_precedence() {
    // 1 + 2 * 3 should parse as Add(1, Mul(2, 3))
    let ops = reader::parse("$a1 = 1 + 2 * 3;").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Expr(expr, Register::A1) => match expr {
            Expr::Binary(lhs, BinOp::Add, rhs) => {
                assert!(matches!(**lhs, Expr::Literal(Value::U8(1))));
                if let Expr::Binary(ll, BinOp::Mul, rr) = &**rhs {
                    assert!(matches!(**ll, Expr::Literal(Value::U8(2))));
                    assert!(matches!(**rr, Expr::Literal(Value::U8(3))));
                } else {
                    panic!("expected Mul");
                }
            }
            _ => panic!("expected Add"),
        },
        _ => panic!("expected Expr"),
    }
}

#[test]
fn parse_parentheses_override_precedence() {
    // (1 + 2) * 3 should parse as Mul(Add(1, 2), 3)
    let ops = reader::parse("$a1 = ( 1 + 2 ) * 3;").unwrap();
    match &ops[0] {
        OpCode::Expr(expr, _) => match expr {
            Expr::Binary(lhs, BinOp::Mul, rhs) => {
                assert!(matches!(**rhs, Expr::Literal(Value::U8(3))));
                assert!(matches!(**lhs, Expr::Binary(_, BinOp::Add, _)));
            }
            _ => panic!("expected Mul"),
        },
        _ => panic!("expected Expr"),
    }
}

#[test]
fn parse_unary_negation() {
    // -5 is constant-folded to Literal(I32(-5))
    let ops = reader::parse("$a1 = -5;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(v), _) => {
            assert_eq!(v, &Value::I32(-5));
        }
        _ => panic!("expected folded literal"),
    }
}

#[test]
fn parse_double_unary() {
    // --5 is constant-folded: inner -5 → I32(-5), outer -(−5) → I32(5)
    let ops = reader::parse("$a1 = --5;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(v), _) => {
            assert_eq!(v, &Value::I32(5));
        }
        _ => panic!("expected folded literal"),
    }
}

#[test]
fn parse_logical_not() {
    let ops = reader::parse("$a1 = !true;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Unary(UnOp::Not, inner), _) => {
            assert!(matches!(**inner, Expr::Literal(Value::Bool(true))));
        }
        _ => panic!("expected Unary Not"),
    }
}

#[test]
fn parse_bitwise_not() {
    let ops = reader::parse("$a1 = ~0;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Unary(UnOp::BitNot, inner), _) => {
            assert!(matches!(**inner, Expr::Literal(Value::U8(0))));
        }
        _ => panic!("expected Unary BitNot"),
    }
}

#[test]
fn parse_comparison_precedence() {
    // a + b < c * d  should be  Lt(Add(a,b), Mul(c,d))
    let ops = reader::parse("$a1 = 1 + 2 < 3 * 4;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Binary(lhs, BinOp::Lt, rhs), _) => {
            assert!(matches!(**lhs, Expr::Binary(_, BinOp::Add, _)));
            assert!(matches!(**rhs, Expr::Binary(_, BinOp::Mul, _)));
        }
        _ => panic!("expected Lt"),
    }
}

#[test]
fn parse_bitwise_precedence() {
    // 1 & 2 | 3  should be  BitOr(BitAnd(1,2), 3)
    let ops = reader::parse("$a1 = 1 & 2 | 3;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Binary(lhs, BinOp::BitOr, rhs), _) => {
            assert!(matches!(**lhs, Expr::Binary(_, BinOp::BitAnd, _)));
            assert!(matches!(**rhs, Expr::Literal(Value::U8(3))));
        }
        _ => panic!("expected BitOr"),
    }
}

#[test]
fn parse_logical_and_or() {
    let ops = reader::parse("$a1 = true && false || true;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Binary(lhs, BinOp::Or, _), _) => {
            assert!(matches!(**lhs, Expr::Binary(_, BinOp::And, _)));
        }
        _ => panic!("expected Or"),
    }
}

#[test]
fn parse_complex_nested() {
    let ops = reader::parse("$a1 = ($a2 + 5) * ($a3 - 2) > 10 && $a4 == 1;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Binary(lhs, BinOp::And, rhs), _) => {
            assert!(matches!(**lhs, Expr::Binary(_, BinOp::Gt, _)));
            assert!(matches!(**rhs, Expr::Binary(_, BinOp::Eq, _)));
        }
        _ => panic!("expected And"),
    }
}

#[test]
fn parse_old_keywords_still_work() {
    let ops = reader::parse("add 1 2 $a1;").unwrap();
    assert_eq!(ops.len(), 1);
    assert!(matches!(ops[0], OpCode::Add(..)));
}

// ── Bincode roundtrip tests ───────────────────────────────────────
// 中文：Bincode 往返测试
// 日本語：Bincode 往復テスト
// Русский: Тесты циклической сериализации bincode

#[test]
fn expr_roundtrip() {
    let ops = vec![OpCode::Expr(
        Expr::Binary(
            Box::new(Expr::Literal(Value::U8(1))),
            BinOp::Add,
            Box::new(Expr::Literal(Value::U8(2))),
        ),
        Register::A1,
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert!(matches!(&deserialized[0], OpCode::Expr(_, Register::A1)));
}

#[test]
fn expr_with_unary_roundtrip() {
    let ops = vec![OpCode::Expr(
        Expr::Unary(UnOp::Neg, Box::new(Expr::Literal(Value::I32(5)))),
        Register::A2,
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert!(matches!(&deserialized[0], OpCode::Expr(_, Register::A2)));
}

#[test]
fn expr_with_register_roundtrip() {
    let ops = vec![OpCode::Expr(
        Expr::Binary(
            Box::new(Expr::Register(Register::A1)),
            BinOp::Add,
            Box::new(Expr::Literal(Value::U8(3))),
        ),
        Register::A3,
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert!(matches!(&deserialized[0], OpCode::Expr(_, Register::A3)));
}

#[test]
fn nested_expr_roundtrip() {
    let ops = vec![OpCode::Block(vec![
        OpCode::Expr(
            Expr::Literal(Value::U8(10)),
            Register::A1,
        ),
        OpCode::Expr(
            Expr::Binary(
                Box::new(Expr::Register(Register::A1)),
                BinOp::Mul,
                Box::new(Expr::Literal(Value::U8(2))),
            ),
            Register::A2,
        ),
    ])];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert!(matches!(&deserialized[0], OpCode::Block(v) if v.len() == 2));
}

// ── Integration: parse then execute ───────────────────────────────
// 中文：集成：解析然后执行
// 日本語：統合：解析して実行
// Русский: Интеграция: парсинг и выполнение

/// Helper: parse source, execute against fresh registers, return registers.
/// 中文：辅助函数：解析源代码，在全新寄存器上执行，返回寄存器。
/// 日本語：ヘルパー：ソースを解析し、新しいレジスタで実行し、レジスタを返します。
/// Русский: Помощник: парсит исходник, выполняет на свежих регистрах, возвращает регистры.
fn run(source: &str) -> Registers {
    let ops = reader::parse(source).unwrap();
    let mut regs = Registers::new();
    ralr::runner::runner(&ops, &mut regs).unwrap();
    regs
}

#[test]
fn eval_simple_arithmetic() {
    let regs = run("$a1 = 1 + 2 * 3;");
    assert_eq!(regs.read(Register::A1), &Value::U8(7));
}

#[test]
fn eval_register_chaining() {
    let regs = run("$a1 = 10; $a2 = $a1 + 5;");
    assert_eq!(regs.read(Register::A1), &Value::U8(10));
    assert_eq!(regs.read(Register::A2), &Value::U8(15));
}

#[test]
fn eval_parentheses() {
    let regs = run("$a1 = (1 + 2) * 3;");
    assert_eq!(regs.read(Register::A1), &Value::U8(9));
}

#[test]
fn eval_comparison_true() {
    let regs = run("$a1 = 5 < 10;");
    assert_eq!(regs.read(Register::A1), &Value::Bool(true));
}

#[test]
fn eval_comparison_false() {
    let regs = run("$a1 = 5 > 10;");
    assert_eq!(regs.read(Register::A1), &Value::Bool(false));
}

#[test]
fn eval_eq() {
    let regs = run("$a1 = 42 == 42;");
    assert_eq!(regs.read(Register::A1), &Value::Bool(true));
}

#[test]
fn eval_bitwise() {
    let regs = run("$a1 = 255 & 15;");
    assert_eq!(regs.read(Register::A1), &Value::U8(15));
}

#[test]
fn eval_shift() {
    let regs = run("$a1 = 1 << 3;");
    assert_eq!(regs.read(Register::A1), &Value::U8(8));
}

#[test]
fn eval_negation() {
    let regs = run("$a1 = -10;");
    assert_eq!(regs.read(Register::A1), &Value::I32(-10));
}

#[test]
fn eval_logical_and() {
    let regs = run("$a1 = true && false;");
    assert_eq!(regs.read(Register::A1), &Value::Bool(false));
}

#[test]
fn eval_logical_or() {
    let regs = run("$a1 = false || true;");
    assert_eq!(regs.read(Register::A1), &Value::Bool(true));
}

#[test]
fn eval_logical_not() {
    let regs = run("$a1 = !true;");
    assert_eq!(regs.read(Register::A1), &Value::Bool(false));
}

#[test]
fn eval_bitwise_not() {
    let regs = run("$a1 = ~0;");
    assert_eq!(regs.read(Register::A1), &Value::U8(0xFF));
}

#[test]
fn eval_expression_in_block() {
    let regs = run("{ $a1 = (1 + 2) * 3; }");
    assert_eq!(regs.read(Register::A1), &Value::U8(9));
}

#[test]
fn eval_mixed_old_and_new() {
    let regs = run("$a1 = 10; add $a1 5 $a2;");
    assert_eq!(regs.read(Register::A1), &Value::U8(10));
    assert_eq!(regs.read(Register::A2), &Value::U8(15));
}

// ── Escape sequence tests ────────────────────────────────────────────

#[test]
fn parse_string_with_newline_escape() {
    let ops = reader::parse(r#"$a1 = "a\nb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\nb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_tab_escape() {
    let ops = reader::parse(r#"$a1 = "a\tb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\tb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_carriage_return_escape() {
    let ops = reader::parse(r#"$a1 = "a\rb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\rb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_backslash_escape() {
    let ops = reader::parse(r#"$a1 = "a\\b";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\\b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_double_quote_escape() {
    let ops = reader::parse(r#"$a1 = "a\"b";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\"b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_single_quote_escape() {
    let ops = reader::parse(r#"$a1 = "a\'b";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a'b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_bell_escape() {
    let ops = reader::parse(r#"$a1 = "a\ab";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\x07b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_backspace_escape() {
    let ops = reader::parse(r#"$a1 = "a\bb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\x08b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_form_feed_escape() {
    let ops = reader::parse(r#"$a1 = "a\fb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\x0Cb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_vertical_tab_escape() {
    let ops = reader::parse(r#"$a1 = "a\vb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\x0Bb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_escape_char_escape() {
    let ops = reader::parse(r#"$a1 = "a\eb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\x1Bb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_null_escape() {
    let ops = reader::parse(r#"$a1 = "a\0b";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => assert_eq!(s, "a\0b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_multiple_escapes() {
    let ops = reader::parse(r#"$a1 = "\t\r\n\0\x41\u{4e2d}";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Literal(Value::String(s)), _) => {
            assert_eq!(s, "\t\r\n\0\x41\u{4e2d}");
        }
        _ => panic!("expected string literal"),
    }
}

#[test]
fn unescape_invalid_escape_errors() {
    let result = reader::parse(r#"$a1 = "\w";"#);
    assert!(result.is_err());
}

// ── If/else tests ────────────────────────────────────────────────────

#[test]
fn parse_if_true_statement() {
    let ops = reader::parse("if true { $a1 = 1; }").unwrap();
    assert_eq!(ops.len(), 1);
    assert!(matches!(&ops[0], OpCode::If(_, _, None)));
}

#[test]
fn parse_if_else_statement() {
    let ops = reader::parse("if true { $a1 = 1; } else { $a1 = 2; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::If(_, then, else_b) => {
            assert_eq!(then.len(), 1);
            assert!(else_b.is_some());
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn parse_if_else_if_chain() {
    let ops = reader::parse(
        "if $a1 == 1 { $a2 = 10; } else if $a1 == 2 { $a2 = 20; } else { $a2 = 30; }",
    )
    .unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::If(_, then, Some(else_b)) => {
            assert_eq!(then.len(), 1);
            // else body is a single If (the "else if" desugaring)
            assert_eq!(else_b.len(), 1);
            assert!(matches!(&else_b[0], OpCode::If(_, _, _)));
        }
        _ => panic!("expected If with else"),
    }
}

#[test]
fn parse_if_with_complex_condition() {
    let ops =
        reader::parse("if $a1 > 0 && $a2 < 100 || $a3 == 1 { $a4 = 42; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::If(expr, _, _) => {
            assert!(matches!(expr, Expr::Binary(_, _, _)));
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn parse_if_with_parenthesized_condition() {
    let ops = reader::parse("if ($a1 > 0) { $a2 = 1; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::If(expr, _, _) => {
            assert!(matches!(expr, Expr::Binary(_, _, _)));
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn if_condition_requires_brace() {
    let result = reader::parse("if true $a1 = 1;");
    assert!(result.is_err());
}

#[test]
fn if_condition_requires_condition() {
    let result = reader::parse("if { $a1 = 1; }");
    assert!(result.is_err());
}

#[test]
fn if_missing_else_brace_errors() {
    let result = reader::parse("if true { $a1 = 1; } else $a1 = 2;");
    assert!(result.is_err());
}

// ── If/else execution tests ──────────────────────────────────────────

#[test]
fn eval_if_true_executes_then() {
    let regs = run("if true { $a1 = 42; }");
    assert_eq!(regs.read(Register::A1), &Value::U8(42));
}

#[test]
fn eval_if_false_skips_then() {
    let regs = run("$a1 = 0; if false { $a1 = 99; }");
    assert_eq!(regs.read(Register::A1), &Value::U8(0));
}

#[test]
fn eval_if_else_false_branch() {
    let regs = run("if false { $a1 = 1; } else { $a1 = 2; }");
    assert_eq!(regs.read(Register::A1), &Value::U8(2));
}

#[test]
fn eval_if_else_true_branch() {
    let regs = run("if true { $a1 = 1; } else { $a1 = 2; }");
    assert_eq!(regs.read(Register::A1), &Value::U8(1));
}

#[test]
fn eval_if_else_if_chain_first() {
    let regs = run(
        "if 1 == 1 { $a1 = 1; } else if 1 == 2 { $a1 = 2; } else { $a1 = 3; }",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(1));
}

#[test]
fn eval_if_else_if_chain_second() {
    let regs = run(
        "if 1 == 2 { $a1 = 1; } else if 1 == 1 { $a1 = 2; } else { $a1 = 3; }",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(2));
}

#[test]
fn eval_if_else_if_chain_else() {
    let regs = run(
        "if 1 == 2 { $a1 = 1; } else if 1 == 3 { $a1 = 2; } else { $a1 = 3; }",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(3));
}

#[test]
fn eval_if_else_if_no_final_else() {
    let regs = run("$a1 = 0; if false { $a1 = 1; } else if false { $a1 = 2; }");
    assert_eq!(regs.read(Register::A1), &Value::U8(0));
}

#[test]
fn eval_nested_if() {
    let regs = run("if true { if true { $a1 = 99; } }");
    assert_eq!(regs.read(Register::A1), &Value::U8(99));
}

#[test]
fn eval_if_inside_block() {
    let regs = run("{ if true { $a1 = 7; } }");
    assert_eq!(regs.read(Register::A1), &Value::U8(7));
}

#[test]
fn eval_if_with_expression_condition() {
    let regs = run("$a1 = 10; $a2 = 5; if $a1 > $a2 { $a3 = 1; } else { $a3 = 0; }");
    assert_eq!(regs.read(Register::A3), &Value::U8(1));
}

#[test]
#[should_panic]
fn eval_if_non_bool_condition_panics() {
    run("if 42 { $a1 = 1; }");
}

// ── If/else bincode roundtrip ────────────────────────────────────────

#[test]
fn if_else_roundtrip() {
    let ops = vec![OpCode::If(
        Expr::Literal(Value::Bool(true)),
        vec![OpCode::Expr(
            Expr::Literal(Value::U8(1)),
            Register::A1,
        )],
        Some(vec![OpCode::Expr(
            Expr::Literal(Value::U8(2)),
            Register::A1,
        )]),
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::If(cond, then, else_b) => {
            assert_eq!(cond, &Expr::Literal(Value::Bool(true)));
            assert_eq!(then.len(), 1);
            assert!(else_b.is_some());
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn if_without_else_roundtrip() {
    let ops = vec![OpCode::If(
        Expr::Literal(Value::Bool(false)),
        vec![],
        None,
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::If(_, then, else_b) => {
            assert!(then.is_empty());
            assert!(else_b.is_none());
        }
        _ => panic!("expected If"),
    }
}
