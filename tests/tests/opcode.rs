use vm_isa::op_code::{OpCode, Operand};
use vm_isa::register::Register;
use vm_isa::value::Value;

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
