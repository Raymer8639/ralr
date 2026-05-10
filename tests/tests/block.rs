//! Tests for `{ ... }` block parsing, serialization, and nesting.
//! 中文：`{ ... }` 块解析、序列化和嵌套测试。
//! 日本語：`{ ... }` ブロックの解析、シリアライズ、ネストのテスト。
//! Русский: Тесты парсинга, сериализации и вложенности блоков `{ ... }`.

use ralr_asm::reader;
use vm_isa::op_code::{OpCode, Operand};
use vm_isa::register::Register;
use vm_isa::value::Value;

// ── Parser tests ──────────────────────────────────────────────────
// 中文：解析器测试
// 日本語：パーサーテスト
// Русский: Тесты парсера

#[test]
fn parse_empty_block() {
    let ops = reader::parse("{}").unwrap();
    assert_eq!(ops.len(), 1);
    assert!(matches!(&ops[0], OpCode::Block(v) if v.is_empty()));
}

#[test]
fn parse_empty_block_with_whitespace() {
    let ops = reader::parse("{  }").unwrap();
    assert_eq!(ops.len(), 1);
    assert!(matches!(&ops[0], OpCode::Block(v) if v.is_empty()));
}

#[test]
fn parse_block_with_one_instruction() {
    let ops = reader::parse("{ add 1 2 $a1; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(v) => {
            assert_eq!(v.len(), 1);
            assert!(matches!(v[0], OpCode::Add(..)));
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn parse_block_with_multiple_instructions() {
    let ops = reader::parse("{ add 1 2 $a1; sub $a1 1 $a2; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(v) => {
            assert_eq!(v.len(), 2);
            assert!(matches!(v[0], OpCode::Add(..)));
            assert!(matches!(v[1], OpCode::Sub(..)));
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn parse_nested_block() {
    let ops = reader::parse("{ add 1 2 $a1; { sub $a1 1 $a2; } }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(outer) => {
            assert_eq!(outer.len(), 2);
            assert!(matches!(outer[0], OpCode::Add(..)));
            assert!(matches!(&outer[1], OpCode::Block(inner) if inner.len() == 1));
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn parse_deeply_nested() {
    let ops = reader::parse("{{{ add 1 2 $a1; }}}").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(l1) => {
            assert_eq!(l1.len(), 1);
            if let OpCode::Block(l2) = &l1[0] {
                assert_eq!(l2.len(), 1);
                if let OpCode::Block(l3) = &l2[0] {
                    assert_eq!(l3.len(), 1);
                    assert!(matches!(l3[0], OpCode::Add(..)));
                } else {
                    panic!("expected level-3 block");
                }
            } else {
                panic!("expected level-2 block");
            }
        }
        _ => panic!("expected level-1 block"),
    }
}

#[test]
fn parse_mixed_flat_and_blocks() {
    let ops = reader::parse("add 0 0 $a1; { add 1 2 $a2; } sub $a2 1 $a3;").unwrap();
    assert_eq!(ops.len(), 3);
    assert!(matches!(ops[0], OpCode::Add(..)));
    if let OpCode::Block(v) = &ops[1] {
        assert_eq!(v.len(), 1);
    } else {
        panic!("expected block");
    }
    assert!(matches!(ops[2], OpCode::Sub(..)));
}

#[test]
fn parse_string_containing_brace_in_block() {
    // The { inside the string must NOT open a nested block.
    // 中文：字符串内的 { 不能打开嵌套块。
    // 日本語：文字列内の { はネストブロックを開いてはいけません。
    // Русский: { внутри строки НЕ должен открывать вложенный блок.
    let ops = reader::parse("{ _println \"hello { world\"; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(v) => {
            assert_eq!(v.len(), 1);
            if let OpCode::Println(Operand::Literal(Value::String(s))) = &v[0] {
                assert_eq!(s, "hello { world");
            } else {
                panic!("expected Println with string literal");
            }
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn parse_string_containing_close_brace_in_block() {
    let ops = reader::parse("{ _println \"hello } world\"; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(v) => {
            assert_eq!(v.len(), 1);
            if let OpCode::Println(Operand::Literal(Value::String(s))) = &v[0] {
                assert_eq!(s, "hello } world");
            } else {
                panic!("expected Println with string literal");
            }
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn parse_string_containing_semicolon_in_block() {
    // The ; inside the string must NOT terminate the instruction.
    // 中文：字符串内的 ; 不能终止指令。
    // 日本語：文字列内の ; は命令を終了してはいけません。
    // Русский: ; внутри строки НЕ должен завершать инструкцию.
    let ops = reader::parse("{ _println \"semi;colon\"; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(v) => {
            assert_eq!(v.len(), 1);
            if let OpCode::Println(Operand::Literal(Value::String(s))) = &v[0] {
                assert_eq!(s, "semi;colon");
            } else {
                panic!("expected Println with string literal");
            }
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn parse_consecutive_semicolons() {
    let ops = reader::parse("{ ;;; add 1 2 $a1; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(v) => {
            assert_eq!(v.len(), 1);
            assert!(matches!(v[0], OpCode::Add(..)));
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn parse_multiline_block() {
    let ops = reader::parse("{\n  add 1 2 $a1;\n  _println $a1;\n}\n").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(v) => {
            assert_eq!(v.len(), 2);
            assert!(matches!(v[0], OpCode::Add(..)));
            assert!(matches!(v[1], OpCode::Println(..)));
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn parse_instruction_without_semicolon_before_close_brace() {
    // Lenient: instruction without ; before } still parses.
    // 中文：宽松模式：} 前没有 ; 的指令仍然会解析。
    // 日本語：寛容モード：} の前に ; がない命令も解析されます。
    // Русский: Лояльный режим: инструкция без ; перед } всё равно парсится.
    let ops = reader::parse("{ add 1 2 $a1 }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(v) => {
            assert_eq!(v.len(), 1);
            assert!(matches!(v[0], OpCode::Add(..)));
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn parse_unknown_keyword_in_block_silently_ignored() {
    let ops = reader::parse("{ foobar 1 2 $a1; add 1 2 $a1; }").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::Block(v) => {
            assert_eq!(v.len(), 1); // only `add`, not `foobar`
                                    // 中文：只有 `add`，没有 `foobar`
                                    // 日本語：`add` のみ、`foobar` はなし
                                    // Русский: только `add`, не `foobar`
            assert!(matches!(v[0], OpCode::Add(..)));
        }
        _ => panic!("expected block"),
    }
}

// ── Bincode roundtrip tests ────────────────────────────────────────
// 中文：Bincode 往返测试
// 日本語：Bincode 往復テスト
// Русский: Тесты циклической сериализации bincode

#[test]
fn block_roundtrip() {
    let ops = vec![OpCode::Block(vec![
        OpCode::Add(
            Operand::Literal(Value::U8(1)),
            Operand::Literal(Value::U8(2)),
            Register::A1,
        ),
    ])];

    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert!(matches!(&deserialized[0], OpCode::Block(v) if v.len() == 1));
}

#[test]
fn nested_block_roundtrip() {
    let ops = vec![OpCode::Block(vec![
        OpCode::Add(
            Operand::Literal(Value::U8(1)),
            Operand::Literal(Value::U8(2)),
            Register::A1,
        ),
        OpCode::Block(vec![
            OpCode::Sub(
                Operand::Register(Register::A1),
                Operand::Literal(Value::U8(1)),
                Register::A1,
            ),
        ]),
    ])];

    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    match &deserialized[0] {
        OpCode::Block(outer) => {
            assert_eq!(outer.len(), 2);
            assert!(matches!(outer[0], OpCode::Add(..)));
            assert!(matches!(&outer[1], OpCode::Block(inner) if inner.len() == 1));
        }
        _ => panic!("expected block"),
    }
}

#[test]
fn empty_block_roundtrip() {
    let ops = vec![OpCode::Block(vec![])];
    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert!(matches!(&deserialized[0], OpCode::Block(v) if v.is_empty()));
}

#[test]
fn mixed_flat_and_block_roundtrip() {
    let ops = vec![
        OpCode::Add(
            Operand::Literal(Value::U8(0)),
            Operand::Literal(Value::U8(1)),
            Register::A1,
        ),
        OpCode::Block(vec![
            OpCode::Sub(
                Operand::Register(Register::A1),
                Operand::Literal(Value::U8(1)),
                Register::A1,
            ),
        ]),
        OpCode::Println(Operand::Register(Register::A1)),
    ];

    let bytes = bincode::serialize(&ops).unwrap();
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(deserialized.len(), 3);
    assert!(matches!(deserialized[0], OpCode::Add(..)));
    assert!(matches!(deserialized[1], OpCode::Block(..)));
    assert!(matches!(deserialized[2], OpCode::Println(..)));
}
