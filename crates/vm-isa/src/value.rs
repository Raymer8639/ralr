use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
