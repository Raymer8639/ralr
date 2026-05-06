use vm_isa::{register::Register, value::Value};

#[test]
fn all_register_new_initializes_all_to_none() {
    let ar = vm_isa::register::AllRegister::new();
    assert!(matches!(ar.a1, Register::A1(Value::None)));
    assert!(matches!(ar.a2, Register::A2(Value::None)));
    assert!(matches!(ar.a3, Register::A3(Value::None)));
    assert!(matches!(ar.a4, Register::A4(Value::None)));
    assert!(matches!(ar.a5, Register::A5(Value::None)));
}

#[test]
fn all_register_default_equals_new() {
    let new_ar = vm_isa::register::AllRegister::new();
    let default_ar = vm_isa::register::AllRegister::default();
    // 每个字段的类型和值一致
    assert!(
        matches!((&new_ar.a1, &default_ar.a1), (Register::A1(Value::None), Register::A1(Value::None)))
    );
    assert!(
        matches!((&new_ar.a2, &default_ar.a2), (Register::A2(Value::None), Register::A2(Value::None)))
    );
    assert!(
        matches!((&new_ar.a3, &default_ar.a3), (Register::A3(Value::None), Register::A3(Value::None)))
    );
    assert!(
        matches!((&new_ar.a4, &default_ar.a4), (Register::A4(Value::None), Register::A4(Value::None)))
    );
    assert!(
        matches!((&new_ar.a5, &default_ar.a5), (Register::A5(Value::None), Register::A5(Value::None)))
    );
}
