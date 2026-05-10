//! Tests for expression parsing, evaluation, and serialization.
//! 中文：表达式解析、求值和序列化测试。
//! 日本語：式の解析、評価、シリアライズのテスト。
//! Русский: Тесты парсинга, вычисления и сериализации выражений.

use ralr_asm::reader;
use vm_isa::op_code::{BinOp, Expr, IoOp, OpCode, Operand, UnOp};
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
    assert!(matches!(&ops[0], OpCode::Expr(Expr::Operand(Operand::Literal(Value::U8(42))), Operand::Register(Register::A1))));
}

#[test]
fn parse_register_assignment() {
    let ops = reader::parse("$a2 = $a1;").unwrap();
    assert_eq!(ops.len(), 1);
    assert!(matches!(&ops[0], OpCode::Expr(Expr::Operand(Operand::Register(Register::A1)), Operand::Register(Register::A2))));
}

#[test]
fn parse_binary_with_precedence() {
    // 1 + 2 * 3 should parse as Add(1, Mul(2, 3))
    let ops = reader::parse("$a1 = 1 + 2 * 3;").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Expr(expr, Operand::Register(Register::A1)) => match expr {
            Expr::Binary(lhs, BinOp::Add, rhs) => {
                assert!(matches!(**lhs, Expr::Operand(Operand::Literal(Value::U8(1)))));
                if let Expr::Binary(ll, BinOp::Mul, rr) = &**rhs {
                    assert!(matches!(**ll, Expr::Operand(Operand::Literal(Value::U8(2)))));
                    assert!(matches!(**rr, Expr::Operand(Operand::Literal(Value::U8(3)))));
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
                assert!(matches!(**rhs, Expr::Operand(Operand::Literal(Value::U8(3)))));
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
        OpCode::Expr(Expr::Operand(Operand::Literal(v)), _) => {
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
        OpCode::Expr(Expr::Operand(Operand::Literal(v)), _) => {
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
            assert!(matches!(**inner, Expr::Operand(Operand::Literal(Value::Bool(true)))));
        }
        _ => panic!("expected Unary Not"),
    }
}

#[test]
fn parse_bitwise_not() {
    let ops = reader::parse("$a1 = ~0;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Unary(UnOp::BitNot, inner), _) => {
            assert!(matches!(**inner, Expr::Operand(Operand::Literal(Value::U8(0)))));
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
            assert!(matches!(**rhs, Expr::Operand(Operand::Literal(Value::U8(3)))));
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
            Box::new(Expr::Operand(Operand::Literal(Value::U8(1)))),
            BinOp::Add,
            Box::new(Expr::Operand(Operand::Literal(Value::U8(2)))),
        ),
        Operand::Register(Register::A1),
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert!(matches!(&deserialized[0], OpCode::Expr(_, Operand::Register(Register::A1))));
}

#[test]
fn expr_with_unary_roundtrip() {
    let ops = vec![OpCode::Expr(
        Expr::Unary(UnOp::Neg, Box::new(Expr::Operand(Operand::Literal(Value::I32(5))))),
        Operand::Register(Register::A2),
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert!(matches!(&deserialized[0], OpCode::Expr(_, Operand::Register(Register::A2))));
}

#[test]
fn expr_with_register_roundtrip() {
    let ops = vec![OpCode::Expr(
        Expr::Binary(
            Box::new(Expr::Operand(Operand::Register(Register::A1))),
            BinOp::Add,
            Box::new(Expr::Operand(Operand::Literal(Value::U8(3)))),
        ),
        Operand::Register(Register::A3),
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert!(matches!(&deserialized[0], OpCode::Expr(_, Operand::Register(Register::A3))));
}

#[test]
fn nested_expr_roundtrip() {
    let ops = vec![OpCode::Block(vec![
        OpCode::Expr(
            Expr::Operand(Operand::Literal(Value::U8(10))),
            Operand::Register(Register::A1),
        ),
        OpCode::Expr(
            Expr::Binary(
                Box::new(Expr::Operand(Operand::Register(Register::A1))),
                BinOp::Mul,
                Box::new(Expr::Operand(Operand::Literal(Value::U8(2)))),
            ),
            Operand::Register(Register::A2),
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
    let mut variables: ahash::AHashMap<String, vm_isa::variable::Variable> = ahash::AHashMap::new();
    let mut functions: ahash::AHashMap<String, vm_isa::function::FnDef> = ahash::AHashMap::new();
    ralr::runner::runner(&ops, &mut regs, &mut variables, &mut functions).unwrap();
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
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\nb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_tab_escape() {
    let ops = reader::parse(r#"$a1 = "a\tb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\tb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_carriage_return_escape() {
    let ops = reader::parse(r#"$a1 = "a\rb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\rb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_backslash_escape() {
    let ops = reader::parse(r#"$a1 = "a\\b";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\\b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_double_quote_escape() {
    let ops = reader::parse(r#"$a1 = "a\"b";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\"b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_single_quote_escape() {
    let ops = reader::parse(r#"$a1 = "a\'b";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a'b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_bell_escape() {
    let ops = reader::parse(r#"$a1 = "a\ab";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\x07b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_backspace_escape() {
    let ops = reader::parse(r#"$a1 = "a\bb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\x08b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_form_feed_escape() {
    let ops = reader::parse(r#"$a1 = "a\fb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\x0Cb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_vertical_tab_escape() {
    let ops = reader::parse(r#"$a1 = "a\vb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\x0Bb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_escape_char_escape() {
    let ops = reader::parse(r#"$a1 = "a\eb";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\x1Bb"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_null_escape() {
    let ops = reader::parse(r#"$a1 = "a\0b";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => assert_eq!(s, "a\0b"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn parse_string_with_multiple_escapes() {
    let ops = reader::parse(r#"$a1 = "\t\r\n\0\x41\u{4e2d}";"#).unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Operand(Operand::Literal(Value::String(s))), _) => {
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
        Expr::Operand(Operand::Literal(Value::Bool(true))),
        vec![OpCode::Expr(
            Expr::Operand(Operand::Literal(Value::U8(1))),
            Operand::Register(Register::A1),
        )],
        Some(vec![OpCode::Expr(
            Expr::Operand(Operand::Literal(Value::U8(2))),
            Operand::Register(Register::A1),
        )]),
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::If(cond, then, else_b) => {
            assert_eq!(cond, &Expr::Operand(Operand::Literal(Value::Bool(true))));
            assert_eq!(then.len(), 1);
            assert!(else_b.is_some());
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn if_without_else_roundtrip() {
    let ops = vec![OpCode::If(
        Expr::Operand(Operand::Literal(Value::Bool(false))),
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

// ── IO operation tests ────────────────────────────────────────────────

#[test]
fn parse_io_write_literal() {
    let ops = reader::parse("io write \"hello\";").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::IO(IoOp::Write(Operand::Literal(Value::String(s)))) => {
            assert_eq!(s, "hello");
        }
        other => panic!("expected IO(Write(Literal(String))), got {other:?}"),
    }
}

#[test]
fn parse_io_writeln_literal() {
    let ops = reader::parse("io writeln \"world\";").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::IO(IoOp::Writeln(Operand::Literal(Value::String(s)))) => {
            assert_eq!(s, "world");
        }
        other => panic!("expected IO(Writeln(Literal(String))), got {other:?}"),
    }
}

#[test]
fn parse_io_write_number() {
    let ops = reader::parse("io write 42;").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::IO(IoOp::Write(Operand::Literal(Value::U8(42)))) => {}
        other => panic!("expected IO(Write(Literal(U8(42)))), got {other:?}"),
    }
}

#[test]
fn parse_io_write_register() {
    let ops = reader::parse("io write $a1;").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::IO(IoOp::Write(Operand::Register(Register::A1))) => {}
        other => panic!("expected IO(Write(Register(A1))), got {other:?}"),
    }
}

#[test]
fn parse_io_read_register() {
    let ops = reader::parse("io read $a3;").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::IO(IoOp::Read(Register::A3)) => {}
        other => panic!("expected IO(Read(A3)), got {other:?}"),
    }
}

#[test]
fn parse_io_readln_register() {
    let ops = reader::parse("io readln $a5;").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::IO(IoOp::Readln(Register::A5)) => {}
        other => panic!("expected IO(Readln(A5)), got {other:?}"),
    }
}

#[test]
fn parse_io_unknown_subcommand() {
    let result = reader::parse("io foobar 42;");
    assert!(result.is_err());
}

#[test]
fn parse_io_write_missing_operand() {
    let result = reader::parse("io write;");
    assert!(result.is_err());
}

#[test]
fn parse_io_read_missing_register() {
    let result = reader::parse("io read;");
    assert!(result.is_err());
}

#[test]
fn io_write_roundtrip() {
    let ops = vec![OpCode::IO(IoOp::Write(Operand::Literal(Value::String(
        "test".into(),
    ))))];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::IO(IoOp::Write(Operand::Literal(Value::String(s)))) => {
            assert_eq!(s, "test");
        }
        other => panic!("expected IO(Write), got {other:?}"),
    }
}

#[test]
fn io_writeln_roundtrip() {
    let ops = vec![OpCode::IO(IoOp::Writeln(Operand::Literal(Value::U8(99))))];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::IO(IoOp::Writeln(Operand::Literal(Value::U8(99)))) => {}
        other => panic!("expected IO(Writeln), got {other:?}"),
    }
}

#[test]
fn io_read_roundtrip() {
    let ops = vec![OpCode::IO(IoOp::Read(Register::A2))];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::IO(IoOp::Read(Register::A2)) => {}
        other => panic!("expected IO(Read), got {other:?}"),
    }
}

#[test]
fn io_readln_roundtrip() {
    let ops = vec![OpCode::IO(IoOp::Readln(Register::A4))];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::IO(IoOp::Readln(Register::A4)) => {}
        other => panic!("expected IO(Readln), got {other:?}"),
    }
}

#[test]
fn io_write_executes_without_panic() {
    let regs = run("io write \"ok\";");
    // Verify no side effects on registers from write.
    assert_eq!(regs.read(Register::A1), &Value::None);
}

#[test]
fn io_writeln_executes_without_panic() {
    let regs = run("io writeln 42;");
    assert_eq!(regs.read(Register::A1), &Value::None);
}

#[test]
fn io_write_with_register_value() {
    let regs = run("$a1 = 100; io write $a1;");
    assert_eq!(regs.read(Register::A1), &Value::U8(100));
}

// ── Variable tests ───────────────────────────────────────────────────
// 中文：变量测试
// Русский: Тесты переменных

/// Helper: parse source, execute against fresh registers and variable hashmap,
/// return the variable hashmap.
/// 中文：辅助函数：解析源代码，在全新寄存器和变量哈希表上执行，返回变量哈希表。
fn run_with_vars(source: &str) -> (Registers, ahash::AHashMap<String, vm_isa::variable::Variable>) {
    let ops = reader::parse(source).unwrap();
    let mut regs = Registers::new();
    let mut variables: ahash::AHashMap<String, vm_isa::variable::Variable> = ahash::AHashMap::new();
    let mut functions: ahash::AHashMap<String, vm_isa::function::FnDef> = ahash::AHashMap::new();
    ralr::runner::runner(&ops, &mut regs, &mut variables, &mut functions).unwrap();
    (regs, variables)
}

#[test]
fn parse_let_immutable_variable() {
    let ops = reader::parse("let x = 42;").unwrap();
    assert_eq!(ops.len(), 2);
    // First opcode: Expr writing to SystemVarBuffer
    assert!(matches!(&ops[0], OpCode::Expr(_, Operand::Register(Register::SystemVarBuffer))));
    // Second opcode: Variable declaration with is_mut = false
    match &ops[1] {
        OpCode::Variable(name, var) => {
            assert_eq!(name, "x");
            assert!(!var.is_mut);
        }
        other => panic!("expected Variable, got {other:?}"),
    }
}

#[test]
fn parse_let_mut_mutable_variable() {
    let ops = reader::parse("let mut y = 10;").unwrap();
    assert_eq!(ops.len(), 2);
    assert!(matches!(&ops[0], OpCode::Expr(_, Operand::Register(Register::SystemVarBuffer))));
    match &ops[1] {
        OpCode::Variable(name, var) => {
            assert_eq!(name, "y");
            assert!(var.is_mut);
        }
        other => panic!("expected Variable, got {other:?}"),
    }
}

#[test]
fn eval_let_variable() {
    let (_, vars) = run_with_vars("let x = 100;");
    let var = vars.get("x").expect("variable x should exist");
    assert_eq!(var.value, Value::U8(100));
    assert!(!var.is_mut);
}

#[test]
fn eval_let_mut_variable() {
    let (_, vars) = run_with_vars("let mut y = 200;");
    let var = vars.get("y").expect("variable y should exist");
    assert_eq!(var.value, Value::U8(200));
    assert!(var.is_mut);
}

#[test]
fn eval_variable_mutation() {
    let (_, vars) = run_with_vars("let mut count = 0; count = count + 1;");
    let var = vars.get("count").expect("variable count should exist");
    assert_eq!(var.value, Value::U8(1));
}

#[test]
fn eval_variable_in_expression() {
    let (regs, _) = run_with_vars("let a = 10; let b = 20; $a1 = a + b;");
    assert_eq!(regs.read(Register::A1), &Value::U8(30));
}

#[test]
#[should_panic(expected = "cannot assign to immutable variable")]
fn immutable_variable_assignment_panics() {
    run_with_vars("let x = 1; x = 2;");
}

#[test]
#[should_panic(expected = "Cannot find the variable")]
fn undefined_variable_in_expression_panics() {
    run_with_vars("$a1 = undefined_var;");
}

#[test]
fn variable_bincode_roundtrip() {
    use vm_isa::variable::Variable;
    let ops = vec![
        OpCode::Expr(
            Expr::Operand(Operand::Literal(Value::U8(42))),
            Operand::Register(Register::SystemVarBuffer),
        ),
        OpCode::Variable("myvar".into(), Variable {
            is_mut: false,
            value: Value::None,
        }),
    ];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 2);
    assert!(matches!(&deserialized[0], OpCode::Expr(_, Operand::Register(Register::SystemVarBuffer))));
    match &deserialized[1] {
        OpCode::Variable(name, var) => {
            assert_eq!(name, "myvar");
            assert!(!var.is_mut);
        }
        other => panic!("expected Variable, got {other:?}"),
    }
}

#[test]
fn let_missing_equals_errors() {
    let result = reader::parse("let x 42;");
    assert!(result.is_err());
}

#[test]
fn let_missing_expression_errors() {
    let result = reader::parse("let x =;");
    assert!(result.is_err());
}

#[test]
fn let_missing_variable_name_errors() {
    let result = reader::parse("let = 42;");
    assert!(result.is_err());
}

// ── While loop tests ──────────────────────────────────────────────────
// 中文：While 循环测试
// Русский: Тесты циклов while

#[test]
fn parse_while_loop() {
    let ops = reader::parse("while true { $a1 = 1; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::While(cond, body) => {
            assert_eq!(cond, &Expr::Operand(Operand::Literal(Value::Bool(true))));
            assert_eq!(body.len(), 1);
        }
        other => panic!("expected While, got {other:?}"),
    }
}

#[test]
fn parse_while_with_expression_condition() {
    let ops = reader::parse("while $a1 < 10 { $a1 = $a1 + 1; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::While(cond, _) => {
            assert!(matches!(cond, Expr::Binary(_, BinOp::Lt, _)));
        }
        other => panic!("expected While, got {other:?}"),
    }
}

#[test]
fn parse_while_requires_brace() {
    let result = reader::parse("while true $a1 = 1;");
    assert!(result.is_err());
}

#[test]
fn parse_while_requires_condition() {
    let result = reader::parse("while { }");
    assert!(result.is_err());
}

#[test]
fn eval_while_variable_counter() {
    // A while loop that increments a mutable variable from 0 to 5.
    // 中文：一个 while 循环，将可变变量从 0 递增到 5。
    let (_, vars) = run_with_vars(
        "let mut i = 0; while i < 5 { i = i + 1; }",
    );
    let var = vars.get("i").expect("variable i should exist");
    assert_eq!(var.value, Value::U8(5));
}

#[test]
fn while_bincode_roundtrip() {
    let ops = vec![OpCode::While(
        Expr::Operand(Operand::Literal(Value::Bool(true))),
        vec![OpCode::Add(
            Operand::Literal(Value::U8(1)),
            Operand::Literal(Value::U8(1)),
            Register::A1,
        )],
    )];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::While(_, body) => assert_eq!(body.len(), 1),
        other => panic!("expected While, got {other:?}"),
    }
}

#[test]
fn while_with_variable_condition() {
    // While loop using a variable in the condition expression.
    // 中文：在条件表达式中使用变量的 while 循环。
    let (regs, _) = run_with_vars(
        "let cond = true; $a1 = 0; while cond { $a1 = 1; let cond = false; }",
    );
    // The loop body executes once because cond starts as true,
    // then becomes false so the loop exits on the next iteration.
    assert_eq!(regs.read(Register::A1), &Value::U8(1));
}

#[test]
#[should_panic(expected = "Bool")]
fn while_non_bool_condition_panics() {
    run_with_vars("while 42 { }");
}

#[test]
fn io_writeln_with_variable() {
    // io writeln should resolve variable references via the hashmap.
    // 中文：io writeln 应通过哈希表解析变量引用。
    let (_, vars) = run_with_vars("let msg = 99; io writeln msg;");
    let var = vars.get("msg").expect("variable msg should exist");
    assert_eq!(var.value, Value::U8(99));
}

// ── Function tests ────────────────────────────────────────────────────
// 中文：函数测试
// Русский: Тесты функций

#[test]
fn parse_fn_definition() {
    let ops = reader::parse("fn add(a, b) { return a + b; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::FnDef { name, params, body } => {
            assert_eq!(name, "add");
            assert_eq!(params, &["a", "b"]);
            assert!(!body.is_empty());
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

#[test]
fn parse_fn_no_params() {
    let ops = reader::parse("fn answer() { return 42; }").unwrap();
    match &ops[0] {
        OpCode::FnDef { name, params, body } => {
            assert_eq!(name, "answer");
            assert!(params.is_empty());
            assert_eq!(body.len(), 1); // Return
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

#[test]
fn parse_fn_multiple_params() {
    let ops = reader::parse("fn sum(a, b, c, d) { return a + b + c + d; }").unwrap();
    match &ops[0] {
        OpCode::FnDef { params, .. } => {
            assert_eq!(params, &["a", "b", "c", "d"]);
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

#[test]
fn parse_call_standalone() {
    let ops = reader::parse("fn f() { return 1; } call f();").unwrap();
    assert_eq!(ops.len(), 2);
    assert!(matches!(&ops[0], OpCode::FnDef { .. }));
    assert!(matches!(&ops[1], OpCode::Call { .. }));
}

#[test]
fn parse_call_with_assign() {
    let ops = reader::parse("fn f() { return 1; } $a1 = call f();").unwrap();
    match &ops[1] {
        OpCode::Call { name, dest, .. } => {
            assert_eq!(name, "f");
            assert_eq!(dest, &Operand::Register(Register::A1));
        }
        other => panic!("expected Call, got {other:?}"),
    }
}

#[test]
fn parse_call_with_args() {
    let ops = reader::parse("fn add(a, b) { return a + b; } $a1 = call add(1, 2);").unwrap();
    match &ops[1] {
        OpCode::Call { name, args, .. } => {
            assert_eq!(name, "add");
            assert_eq!(args.len(), 2);
        }
        other => panic!("expected Call, got {other:?}"),
    }
}

#[test]
fn parse_return_expr() {
    let ops = reader::parse("fn f() { return 1 + 2; }").unwrap();
    match &ops[0] {
        OpCode::FnDef { body, .. } => {
            assert!(matches!(&body[0], OpCode::Return(_)));
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

#[test]
fn parse_return_empty() {
    let ops = reader::parse("fn f() { return; }").unwrap();
    match &ops[0] {
        OpCode::FnDef { body, .. } => {
            assert!(matches!(&body[0], OpCode::Return(_)));
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

#[test]
fn parse_return_call() {
    let ops = reader::parse("fn f() { fn g() { return 1; } return call g(); }").unwrap();
    match &ops[0] {
        OpCode::FnDef { body, .. } => {
            assert_eq!(body.len(), 2); // FnDef for g, Call for g
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

#[test]
fn parse_let_with_call() {
    let ops = reader::parse("fn f() { return 1; } let x = call f();").unwrap();
    // FnDef, Call, Variable
    assert_eq!(ops.len(), 3);
    assert!(matches!(&ops[0], OpCode::FnDef { .. }));
    assert!(matches!(&ops[1], OpCode::Call { .. }));
    assert!(matches!(&ops[2], OpCode::Variable(..)));
}

/// Helper: parse source with functions, execute, return registers and vars.
/// 中文：辅助函数：解析带函数的源代码，执行，返回寄存器和变量。
fn run_with_fns(
    source: &str,
) -> (
    Registers,
    ahash::AHashMap<String, vm_isa::variable::Variable>,
) {
    let ops = reader::parse(source).unwrap();
    let mut regs = Registers::new();
    let mut variables: ahash::AHashMap<String, vm_isa::variable::Variable> = ahash::AHashMap::new();
    let mut functions: ahash::AHashMap<String, vm_isa::function::FnDef> = ahash::AHashMap::new();
    ralr::runner::runner(&ops, &mut regs, &mut variables, &mut functions).unwrap();
    (regs, variables)
}

#[test]
fn eval_simple_fn_call() {
    let (regs, _) = run_with_fns("fn add(a, b) { return a + b; } $a1 = call add(5, 7);");
    assert_eq!(regs.read(Register::A1), &Value::U8(12));
}

#[test]
fn eval_fn_call_no_args() {
    let (regs, _) = run_with_fns("fn answer() { return 42; } $a1 = call answer();");
    assert_eq!(regs.read(Register::A1), &Value::U8(42));
}

#[test]
fn eval_fn_call_with_variable_arg() {
    let (regs, _) = run_with_fns(
        "fn double(n) { return n + n; } let x = 5; $a1 = call double(x);",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(10));
}

#[test]
fn eval_fn_calling_fn() {
    let (regs, _) = run_with_fns(
        "fn double(n) { return n + n; } fn quadruple(n) { let d = call double(n); return call double(d); } $a1 = call quadruple(3);",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(12));
}

#[test]
fn eval_fn_using_outer_variable() {
    // Functions close over outer-scope variables.
    // 中文：函数闭包引用外部作用域的变量。
    let (regs, _) = run_with_fns(
        "let factor = 3; fn scale(n) { return n * factor; } $a1 = call scale(4);",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(12));
}

#[test]
fn eval_fn_with_if() {
    let (regs, _) = run_with_fns(
        "fn max(a, b) { if a > b { return a; } return b; } $a1 = call max(3, 7);",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(7));
}

#[test]
fn eval_fn_param_shadowing() {
    // Parameter names shadow outer variables of the same name.
    // 中文：参数名遮蔽同名的外部变量。
    let (regs, _) = run_with_fns(
        "let x = 100; fn f(x) { return x + 1; } $a1 = call f(5);",
    );
    // Inside f, x = 5 (the argument), not 100 (the outer variable).
    assert_eq!(regs.read(Register::A1), &Value::U8(6));
}

#[test]
fn eval_fn_standalone_call() {
    // Call without explicit dest — result goes to SystemVarBuffer,
    // then can be used via let/register.
    // 中文：无显式目标的调用 — 结果存入 SystemVarBuffer。
    let (regs, _) = run_with_fns(
        "fn answer() { return 42; } call answer(); let x = call answer(); $a1 = x;",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(42));
}

#[test]
#[should_panic(expected = "function not found")]
fn call_undefined_fn_panics() {
    run_with_fns("$a1 = call missing(1, 2);");
}

#[test]
#[should_panic(expected = "expects 2 arguments, got 1")]
fn call_wrong_arg_count_panics() {
    run_with_fns("fn f(a, b) { return a + b; } $a1 = call f(1);");
}

#[test]
fn fn_bincode_roundtrip() {
    let ops = vec![OpCode::FnDef {
        name: "add".into(),
        params: vec!["a".into(), "b".into()],
        body: vec![OpCode::Return(Expr::Binary(
            Box::new(Expr::Operand(Operand::Variable("a".into(), vm_isa::variable::Variable { is_mut: false, value: Value::None }))),
            BinOp::Add,
            Box::new(Expr::Operand(Operand::Variable("b".into(), vm_isa::variable::Variable { is_mut: false, value: Value::None }))),
        ))],
    }];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::FnDef { name, params, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params, &["a", "b"]);
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

#[test]
fn call_bincode_roundtrip() {
    let ops = vec![OpCode::Call {
        name: "f".into(),
        args: vec![Expr::Operand(Operand::Literal(Value::U8(1)))],
        dest: Operand::Register(Register::A1),
    }];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    match &deserialized[0] {
        OpCode::Call { name, args, dest } => {
            assert_eq!(name, "f");
            assert_eq!(args.len(), 1);
            assert_eq!(dest, &Operand::Register(Register::A1));
        }
        other => panic!("expected Call, got {other:?}"),
    }
}

#[test]
fn return_bincode_roundtrip() {
    let ops = vec![OpCode::Return(Expr::Operand(Operand::Literal(
        Value::U8(42),
    )))];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    match &deserialized[0] {
        OpCode::Return(_) => {}
        other => panic!("expected Return, got {other:?}"),
    }
}

#[test]
fn fn_missing_name_errors() {
    let result = reader::parse("fn () { return 1; }");
    assert!(result.is_err());
}

#[test]
fn fn_missing_paren_errors() {
    let result = reader::parse("fn f { return 1; }");
    assert!(result.is_err());
}

#[test]
fn fn_missing_brace_errors() {
    let result = reader::parse("fn f() return 1;");
    assert!(result.is_err());
}

#[test]
fn call_missing_paren_errors() {
    let result = reader::parse("call f;");
    assert!(result.is_err());
}
