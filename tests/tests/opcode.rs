//! Tests for [`OpCode`] bincode serialization roundtrip.
//! 中文：[`OpCode`] bincode 序列化往返测试。
//! 日本語：[`OpCode`] bincode シリアライズ往復テスト。
//! Русский: Тесты циклической сериализации/десериализации [`OpCode`] через bincode.

use vm_isa::op_code::{OpCode, Operand};
use vm_isa::register::Register;
use vm_isa::value::Value;

/// A mix of literal and register operands should survive a serialize →
/// deserialize roundtrip with structure intact.
/// 中文：字面量和寄存器操作数的混合应在序列化 → 反序列化往返后保持结构完整。
/// 日本語：リテラルとレジスタオペランドの混合は、シリアライズ → デシリアライズ往復後も構造が保持される必要があります。
/// Русский: Смесь литеральных и регистровых операндов должна пережить цикл
/// сериализация → десериализация с сохранением структуры.
#[test]
fn serialize_deserialize_roundtrip() {
    let ops = vec![
        OpCode::Add(
            Operand::Literal(Value::U8(1)),
            Operand::Literal(Value::U8(2)),
            Register::A1,
        ),
        OpCode::Sub(
            Operand::Register(Register::A1),
            Operand::Literal(Value::U8(1)),
            Register::A1,
        ),
        OpCode::Mul(
            Operand::Literal(Value::I32(3)),
            Operand::Literal(Value::I32(4)),
            Register::A2,
        ),
        OpCode::Div(
            Operand::Literal(Value::F64(10.0)),
            Operand::Literal(Value::F64(2.0)),
            Register::A3,
        ),
    ];

    let bytes = bincode::serialize(&ops).expect("serialize failed");
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).expect("deserialize failed");

    assert_eq!(deserialized.len(), 4);
    assert!(matches!(deserialized[0], OpCode::Add(..)));
    assert!(matches!(deserialized[1], OpCode::Sub(..)));
    assert!(matches!(deserialized[2], OpCode::Mul(..)));
    assert!(matches!(deserialized[3], OpCode::Div(..)));
}

/// The `examples/add.abin` fixture (4 arithmetic instructions) must
/// deserialize correctly. This ensures the binary format stays stable.
/// 中文：`examples/add.abin` 测试夹具（4 条算术指令）必须正确反序列化。这确保二进制格式保持稳定。
/// 日本語：`examples/add.abin` フィクスチャ（4 つの算術命令）が正しくデシリアライズされる必要があります。これによりバイナリ形式の安定性が保証されます。
/// Русский: Фикстура `examples/add.abin` (4 арифметические инструкции) должна
/// десериализоваться корректно. Это гарантирует стабильность бинарного формата.
#[test]
fn deserialize_example_add_file() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/add.abin");
    let bytes = std::fs::read(path).expect("failed to read add.abin example file");
    let ops: Vec<OpCode> = bincode::deserialize(&bytes).expect("failed to deserialize add.abin");

    assert_eq!(ops.len(), 4);
    assert!(matches!(ops[0], OpCode::Add(..)));
    assert!(matches!(ops[1], OpCode::Sub(..)));
    assert!(matches!(ops[2], OpCode::Mul(..)));
    assert!(matches!(ops[3], OpCode::Div(..)));
}
