//! The [`Value`] type — a tagged union of all supported data types.
//! 中文：[`Value`] 类型 — 所有受支持数据类型的带标签联合体。
//!
//! Value implements `Add`, `Sub`, `Mul`, `Div` for same-type numeric
//! operands. Type mismatches cause a panic (no error propagation).
//! 中文：Value 为同类型数值操作数实现了 `Add`、`Sub`、`Mul`、`Div`。类型不匹配会导致 panic（无错误传播）。

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub},
};

/// Generates the match arms for same-type arithmetic operations.
/// 中文：为同类型算术运算生成 match 分支。
///
/// For each variant in the list it produces a branch like
/// `(Value::I32(a), Value::I32(b)) => Ok(Value::I32(a + b))`.
/// 中文：对于列表中的每个变体，生成一个分支，例如 `(Value::I32(a), Value::I32(b)) => Ok(Value::I32(a + b))`。
macro_rules! op {
    ($self: expr, $rhs: expr, $op: tt, [$($variant: ident),*]) => {
        match ($self, $rhs) {
            $(
                (Value::$variant(a), Value::$variant(b)) => Ok(Value::$variant(a $op b)),
            )*
            (_, _) => Err(anyhow!("cannot perform operation")),
        }
    };
}

/// Generates match arms for comparison operations that return [`Value::Bool`].
/// 中文：为返回 [`Value::Bool`] 的比较运算生成 match 分支。
macro_rules! cmp_op {
    ($self:expr, $rhs:expr, $op:tt, [$($variant:ident),*]) => {
        match ($self, $rhs) {
            $(
                (Value::$variant(a), Value::$variant(b)) => Value::Bool(a $op b),
            )*
            _ => panic!("type mismatch in comparison"),
        }
    };
}

/// Tagged union of every value type the VM understands.
/// 中文：虚拟机理解的所有值类型的带标签联合体。
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Value {
    None,
    I32(i32),
    I128(i128),
    U8(u8),
    U32(u32),
    U128(u128),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::I32(v) => write!(f, "{v}"),
            Value::I128(v) => write!(f, "{v}"),
            Value::U8(v) => write!(f, "{v}"),
            Value::U32(v) => write!(f, "{v}"),
            Value::U128(v) => write!(f, "{v}"),
            Value::F32(v) => write!(f, "{v}"),
            Value::F64(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
        }
    }
}

// ── Arithmetic operators ────────────────────────────────────────────
// 中文：算术运算符

impl Add for Value {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match op!(self, rhs, +, [I32, I128, U8, U32, U128, F32, F64]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match op!(self, rhs, -, [I32, I128, U8, U32, U128, F32, F64]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match op!(self, rhs, *, [I32, I128, U8, U32, U128, F32, F64]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match op!(self, rhs, /, [I32, I128, U8, U32, U128, F32, F64]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

impl Rem for Value {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        match op!(self, rhs, %, [I32, I128, U8, U32, U128, F32, F64]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

// ── Bitwise operators (integers only) ───────────────────────────────
// 中文：位运算符（仅整数）
// Русский: Побитовые операторы (только целые)

impl BitAnd for Value {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        match op!(self, rhs, &, [I32, I128, U8, U32, U128]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

impl BitOr for Value {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        match op!(self, rhs, |, [I32, I128, U8, U32, U128]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

impl BitXor for Value {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        match op!(self, rhs, ^, [I32, I128, U8, U32, U128]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

// ── Shift operators (integers only, wrapping) ───────────────────────
// 中文：移位运算符（仅整数，环绕）

impl Shl for Value {
    type Output = Self;
    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a.wrapping_shl(b as u32)),
            (Value::I128(a), Value::I128(b)) => Value::I128(a.wrapping_shl(b as u32)),
            (Value::U8(a), Value::U8(b)) => Value::U8(a.wrapping_shl(b as u32)),
            (Value::U32(a), Value::U32(b)) => Value::U32(a.wrapping_shl(b)),
            (Value::U128(a), Value::U128(b)) => Value::U128(a.wrapping_shl(b as u32)),
            _ => panic!("type mismatch in shift left"),
        }
    }
}

impl Shr for Value {
    type Output = Self;
    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a.wrapping_shr(b as u32)),
            (Value::I128(a), Value::I128(b)) => Value::I128(a.wrapping_shr(b as u32)),
            (Value::U8(a), Value::U8(b)) => Value::U8(a.wrapping_shr(b as u32)),
            (Value::U32(a), Value::U32(b)) => Value::U32(a.wrapping_shr(b)),
            (Value::U128(a), Value::U128(b)) => Value::U128(a.wrapping_shr(b as u32)),
            _ => panic!("type mismatch in shift right"),
        }
    }
}

// ── Negation (signed integers and floats only) ──────────────────────
// 中文：取反（仅带符号整数和浮点数）

impl Neg for Value {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Value::I32(v) => Value::I32(-v),
            Value::I128(v) => Value::I128(-v),
            Value::F32(v) => Value::F32(-v),
            Value::F64(v) => Value::F64(-v),
            _ => panic!("negation requires signed integer or float operand"),
        }
    }
}

// ── Comparison methods → Value::Bool ────────────────────────────────
// 中文：比较方法 → Value::Bool

impl Value {
    /// Equality comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：相等比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    pub fn eq_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, ==, [I32, I128, U8, U32, U128, F32, F64, Bool, String])
    }

    /// Inequality comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：不等比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    pub fn ne_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, !=, [I32, I128, U8, U32, U128, F32, F64, Bool, String])
    }

    /// Less-than comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：小于比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    pub fn lt_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, <, [I32, I128, U8, U32, U128, F32, F64])
    }

    /// Less-or-equal comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：小于等于比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    pub fn le_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, <=, [I32, I128, U8, U32, U128, F32, F64])
    }

    /// Greater-than comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：大于比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    pub fn gt_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, >, [I32, I128, U8, U32, U128, F32, F64])
    }

    /// Greater-or-equal comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：大于等于比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    pub fn ge_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, >=, [I32, I128, U8, U32, U128, F32, F64])
    }

    // ── Unary methods ────────────────────────────────────────────────
    // 中文：一元方法

    /// Logical NOT (`!`). Panics on non-Bool.
    /// 中文：逻辑非（`!`）。对非 Bool 类型会 panic。
    pub fn logical_not(&self) -> Value {
        match self {
            Value::Bool(b) => Value::Bool(!b),
            _ => panic!("logical NOT requires Bool operand"),
        }
    }

    /// Bitwise NOT (`~`). Panics on non-integer.
    /// 中文：按位非（`~`）。对非整数类型会 panic。
    pub fn bitwise_not(&self) -> Value {
        match self {
            Value::I32(v) => Value::I32(!v),
            Value::I128(v) => Value::I128(!v),
            Value::U8(v) => Value::U8(!v),
            Value::U32(v) => Value::U32(!v),
            Value::U128(v) => Value::U128(!v),
            _ => panic!("bitwise NOT requires integer operand"),
        }
    }

    /// Logical AND (`&&`). Non-short-circuit. Panics on non-Bool.
    /// 中文：逻辑与（`&&`）。非短路求值。对非 Bool 类型会 panic。
    pub fn and_val(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a && *b),
            _ => panic!("logical AND requires Bool operands"),
        }
    }

    /// Logical OR (`||`). Non-short-circuit. Panics on non-Bool.
    /// 中文：逻辑或（`||`）。非短路求值。对非 Bool 类型会 panic。
    pub fn or_val(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a || *b),
            _ => panic!("logical OR requires Bool operands"),
        }
    }
}
