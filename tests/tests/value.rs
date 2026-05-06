use vm_isa::value::Value;

#[test]
fn add_i32() {
    let result = Value::I32(1) + Value::I32(2);
    assert!(matches!(result, Value::I32(3)));
}

#[test]
fn add_f64() {
    let result = Value::F64(1.5) + Value::F64(2.5);
    assert!(matches!(result, Value::F64(v) if (v - 4.0).abs() < f64::EPSILON));
}

#[test]
fn add_u8() {
    let result = Value::U8(10) + Value::U8(20);
    assert!(matches!(result, Value::U8(30)));
}

#[test]
fn sub_i32() {
    let result = Value::I32(5) - Value::I32(3);
    assert!(matches!(result, Value::I32(2)));
}

#[test]
fn sub_f32() {
    let result = Value::F32(5.0) - Value::F32(2.0);
    assert!(matches!(result, Value::F32(3.0)));
}

#[test]
fn mul_i32() {
    let result = Value::I32(3) * Value::I32(4);
    assert!(matches!(result, Value::I32(12)));
}

#[test]
fn mul_f64() {
    let result = Value::F64(2.5) * Value::F64(2.0);
    assert!(matches!(result, Value::F64(v) if (v - 5.0).abs() < f64::EPSILON));
}

#[test]
fn div_i32() {
    let result = Value::I32(10) / Value::I32(2);
    assert!(matches!(result, Value::I32(5)));
}

#[test]
fn div_f64() {
    let result = Value::F64(7.5) / Value::F64(2.5);
    assert!(matches!(result, Value::F64(v) if (v - 3.0).abs() < f64::EPSILON));
}

#[test]
fn add_u128() {
    let result = Value::U128(100) + Value::U128(200);
    assert!(matches!(result, Value::U128(300)));
}

#[test]
fn sub_i128() {
    let result = Value::I128(-10) - Value::I128(5);
    assert!(matches!(result, Value::I128(-15)));
}

#[test]
#[should_panic]
fn type_mismatch_panics() {
    let _ = Value::I32(1) + Value::F64(2.0);
}

#[test]
#[should_panic]
fn none_value_panics() {
    let _ = Value::None + Value::I32(1);
}
