use crate::register::Register;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};

macro_rules! op {
    ($self: expr, $rhs: expr, $op: tt, [$($variant: ident),*]) => { // add!(变量1, 变量2, [所有要支持 $op 运算的类型])
        match ($self, $rhs) { // 遍历两个变量，找到对应的两个相同类型，然后做运算
            $(
                (Value::$variant(a), Value::$variant(b)) => Ok(Value::$variant(a $op b)), // 这里的 $()* 会将输入的所有类型依次按照这个形式展开
            )*
            (_, _) => Err(anyhow!("无法进行运算",)),
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
    Register(Arc<Register>),
}

macro_rules! update_register {
    ($val1: expr, $val2: expr, [$($register: ident),*]) => {
        {
            let val1_new = match $val1 {
                Value::Register(value_arc) => match (*value_arc).clone() {
                    $(
                        Register::$register(value) => value,
                    )*
                },
                _ => $val1,
            };
            let val2_new = match $val2 {
                Value::Register(value_arc) => match (*value_arc).clone() {
                    $(
                        Register::$register(value) => value,
                    )*
                },
                _ => $val2,
            };
            (val1_new, val2_new)
        }
    };
}

impl Add for Value {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let (new_self, new_rhs) = update_register!(self, rhs, [A1, A2, A3, A4, A5]);
        match op!(new_self, new_rhs, +, [I32, I128, U8, U32, U128, F32, F64]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}
impl Sub for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let (new_self, new_rhs) = update_register!(self, rhs, [A1, A2, A3, A4, A5]);

        match op!(new_self, new_rhs, -, [I32, I128, U8, U32, U128, F32, F64]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}
impl Mul for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let (new_self, new_rhs) = update_register!(self, rhs, [A1, A2, A3, A4, A5]);

        match op!(new_self, new_rhs, *, [I32, I128, U8, U32, U128, F32, F64]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}
impl Div for Value {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let (new_self, new_rhs) = update_register!(self, rhs, [A1, A2, A3, A4, A5]);
        match op!(new_self, new_rhs, /, [I32, I128, U8, U32, U128, F32, F64]) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}
