//! The [`Value`] type — a tagged union of all supported data types.
//! 中文：[`Value`] 类型 — 所有受支持数据类型的带标签联合体。
//! 日本語：[`Value`] 型 — サポートされるすべてのデータ型のタグ付き共用体。
//! Русский: Тип [`Value`] — размеченное объединение всех поддерживаемых типов данных.
//!
//! Value implements `Add`, `Sub`, `Mul`, `Div` for same-type numeric
//! operands. Type mismatches cause a panic (no error propagation).
//! 中文：Value 为同类型数值操作数实现了 `Add`、`Sub`、`Mul`、`Div`。类型不匹配会导致 panic（无错误传播）。
//! 日本語：Value は同じ型の数値オペランドに対して `Add`、`Sub`、`Mul`、`Div` を実装します。型の不一致はパニックを引き起こします（エラー伝播なし）。
//! Русский: Value реализует `Add`, `Sub`, `Mul`, `Div` для числовых операндов одного типа.
//! Несовпадение типов вызывает панику (без распространения ошибок).

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub},
};

/// Generates the match arms for same-type arithmetic operations.
/// 中文：为同类型算术运算生成 match 分支。
/// 日本語：同じ型の算術演算のための match アームを生成します。
/// Русский: Генерирует ветви match для арифметических операций одного типа.
///
/// For each variant in the list it produces a branch like
/// `(Value::I32(a), Value::I32(b)) => Ok(Value::I32(a + b))`.
/// 中文：对于列表中的每个变体，生成一个分支，例如 `(Value::I32(a), Value::I32(b)) => Ok(Value::I32(a + b))`。
/// 日本語：リスト内の各バリアントに対して、`(Value::I32(a), Value::I32(b)) => Ok(Value::I32(a + b))` のようなブランチを生成します。
/// Русский: Для каждого варианта в списке создаёт ветвь вида
/// `(Value::I32(a), Value::I32(b)) => Ok(Value::I32(a + b))`.
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
/// 日本語：[`Value::Bool`] を返す比較演算のための match アームを生成します。
/// Русский: Генерирует ветви match для операций сравнения, возвращающих [`Value::Bool`].
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
/// 日本語：VM が理解するすべての値型のタグ付き共用体。
/// Русский: Размеченное объединение всех типов значений, которые понимает ВМ.
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
// 日本語：算術演算子
// Русский: Арифметические операторы

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
// 日本語：ビット演算子（整数のみ）
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
// 日本語：シフト演算子（整数のみ、ラッピング）
// Русский: Операторы сдвига (только целые, с обёртыванием)

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
// 日本語：否定（符号付き整数と浮動小数のみ）
// Русский: Отрицание (только знаковые целые и числа с плавающей точкой)

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
// 日本語：比較メソッド → Value::Bool
// Русский: Методы сравнения → Value::Bool

impl Value {
    /// Equality comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：相等比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    /// 日本語：等価比較。`Value::Bool(true)` または `Value::Bool(false)` を返します。
    /// Русский: Сравнение на равенство. Возвращает `Value::Bool(true)` или `Value::Bool(false)`.
    pub fn eq_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, ==, [I32, I128, U8, U32, U128, F32, F64, Bool, String])
    }

    /// Inequality comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：不等比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    /// 日本語：非等価比較。`Value::Bool(true)` または `Value::Bool(false)` を返します。
    /// Русский: Сравнение на неравенство. Возвращает `Value::Bool(true)` или `Value::Bool(false)`.
    pub fn ne_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, !=, [I32, I128, U8, U32, U128, F32, F64, Bool, String])
    }

    /// Less-than comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：小于比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    /// 日本語：小なり比較。`Value::Bool(true)` または `Value::Bool(false)` を返します。
    /// Русский: Сравнение "меньше". Возвращает `Value::Bool(true)` или `Value::Bool(false)`.
    pub fn lt_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, <, [I32, I128, U8, U32, U128, F32, F64])
    }

    /// Less-or-equal comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：小于等于比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    /// 日本語：小なりイコール比較。`Value::Bool(true)` または `Value::Bool(false)` を返します。
    /// Русский: Сравнение "меньше или равно". Возвращает `Value::Bool(true)` или `Value::Bool(false)`.
    pub fn le_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, <=, [I32, I128, U8, U32, U128, F32, F64])
    }

    /// Greater-than comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：大于比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    /// 日本語：大なり比較。`Value::Bool(true)` または `Value::Bool(false)` を返します。
    /// Русский: Сравнение "больше". Возвращает `Value::Bool(true)` или `Value::Bool(false)`.
    pub fn gt_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, >, [I32, I128, U8, U32, U128, F32, F64])
    }

    /// Greater-or-equal comparison. Returns `Value::Bool(true)` or `Value::Bool(false)`.
    /// 中文：大于等于比较。返回 `Value::Bool(true)` 或 `Value::Bool(false)`。
    /// 日本語：大なりイコール比較。`Value::Bool(true)` または `Value::Bool(false)` を返します。
    /// Русский: Сравнение "больше или равно". Возвращает `Value::Bool(true)` или `Value::Bool(false)`.
    pub fn ge_val(&self, rhs: &Value) -> Value {
        cmp_op!(self, rhs, >=, [I32, I128, U8, U32, U128, F32, F64])
    }

    // ── Unary methods ────────────────────────────────────────────────
    // 中文：一元方法
    // 日本語：単項メソッド
    // Русский: Унарные методы

    /// Logical NOT (`!`). Panics on non-Bool.
    /// 中文：逻辑非（`!`）。对非 Bool 类型会 panic。
    /// 日本語：論理否定（`!`）。非 Bool 型ではパニックします。
    /// Русский: Логическое НЕ (`!`). Паникует на не-Bool.
    pub fn logical_not(&self) -> Value {
        match self {
            Value::Bool(b) => Value::Bool(!b),
            _ => panic!("logical NOT requires Bool operand"),
        }
    }

    /// Bitwise NOT (`~`). Panics on non-integer.
    /// 中文：按位非（`~`）。对非整数类型会 panic。
    /// 日本語：ビット反転（`~`）。非整数型ではパニックします。
    /// Русский: Побитовое НЕ (`~`). Паникует на не-целочисленном.
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
    /// 日本語：論理積（`&&`）。非短絡評価。非 Bool 型ではパニックします。
    /// Русский: Логическое И (`&&`). Без короткого замыкания. Паникует на не-Bool.
    pub fn and_val(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a && *b),
            _ => panic!("logical AND requires Bool operands"),
        }
    }

    /// Logical OR (`||`). Non-short-circuit. Panics on non-Bool.
    /// 中文：逻辑或（`||`）。非短路求值。对非 Bool 类型会 panic。
    /// 日本語：論理和（`||`）。非短絡評価。非 Bool 型ではパニックします。
    /// Русский: Логическое ИЛИ (`||`). Без короткого замыкания. Паникует на не-Bool.
    pub fn or_val(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a || *b),
            _ => panic!("logical OR requires Bool operands"),
        }
    }
}
