use crate::value::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AllRegister {
    pub a1: Register,
    pub a2: Register,
    pub a3: Register,
    pub a4: Register,
    pub a5: Register,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Register {
    A1(Value),
    A2(Value),
    A3(Value),
    A4(Value),
    A5(Value),
}
impl Default for AllRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl AllRegister {
    pub fn new() -> Self {
        Self {
            a1: Register::A1(Value::None),
            a2: Register::A2(Value::None),
            a3: Register::A3(Value::None),
            a4: Register::A4(Value::None),
            a5: Register::A5(Value::None),
        }
    }
}
