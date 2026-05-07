//! Tests for [`Value`] arithmetic operators and error behaviour.
//! 中文：[`Value`] 算术运算符和错误行为测试。
//! 日本語：[`Value`] 算術演算子とエラー動作のテスト。
//! Русский: Тесты арифметических операторов [`Value`] и поведения при ошибках.

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

/// Arithmetic operators panic when the operand types don't match.
/// 中文：当操作数类型不匹配时，算术运算符会 panic。
/// 日本語：オペランドの型が一致しない場合、算術演算子はパニックします。
/// Русский: Арифметические операторы паникуют, когда типы операндов не совпадают.
#[test]
#[should_panic]
fn type_mismatch_panics() {
    let _ = Value::I32(1) + Value::F64(2.0);
}

/// Operations involving [`Value::None`] always panic.
/// 中文：涉及 [`Value::None`] 的运算总是 panic。
/// 日本語：[`Value::None`] を含む演算は常にパニックします。
/// Русский: Операции с участием [`Value::None`] всегда вызывают панику.
#[test]
#[should_panic]
fn none_value_panics() {
    let _ = Value::None + Value::I32(1);
}
