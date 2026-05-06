use std::sync::Arc;
use vm_isa::op_code::OpCode;
use vm_isa::register::Register;
use vm_isa::value::Value;

#[test]
fn serialize_deserialize_roundtrip() {
    let ops = vec![
        OpCode::Add(Value::U8(1), Value::U8(2), Register::A1(Value::None)),
        OpCode::Sub(
            Value::Register(Arc::new(Register::A1(Value::None))),
            Value::U8(1),
            Register::A1(Value::None),
        ),
        OpCode::Mul(Value::I32(3), Value::I32(4), Register::A2(Value::None)),
        OpCode::Div(Value::F64(10.0), Value::F64(2.0), Register::A3(Value::None)),
    ];

    let bytes = bincode::serialize(&ops).expect("serialize 失败");
    let deserialized: Vec<OpCode> = bincode::deserialize(&bytes).expect("deserialize 失败");

    assert_eq!(deserialized.len(), 4);

    // 逐条对比反序列化结果
    assert!(matches!(deserialized[0], OpCode::Add(..)));
    assert!(matches!(deserialized[1], OpCode::Sub(..)));
    assert!(matches!(deserialized[2], OpCode::Mul(..)));
    assert!(matches!(deserialized[3], OpCode::Div(..)));
}

#[test]
fn deserialize_example_add_file() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/add.abin");
    let bytes = std::fs::read(path).expect("无法读取 add.abin 示例文件");
    let ops: Vec<OpCode> =
        bincode::deserialize(&bytes).expect("反序列化 add.abin 失败");

    // add.abin 对应 add.ralr，包含 4 条指令
    assert_eq!(ops.len(), 4);
    assert!(matches!(ops[0], OpCode::Add(..)));
    assert!(matches!(ops[1], OpCode::Sub(..)));
    assert!(matches!(ops[2], OpCode::Mul(..)));
    assert!(matches!(ops[3], OpCode::Div(..)));
}
