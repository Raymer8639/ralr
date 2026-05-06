use crate::value::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Register {
    A1,
    A2,
    A3,
    A4,
    A5,
}

impl Register {
    pub fn index(self) -> usize {
        match self {
            Register::A1 => 0,
            Register::A2 => 1,
            Register::A3 => 2,
            Register::A4 => 3,
            Register::A5 => 4,
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    inner: [Value; 5],
}

impl Registers {
    pub fn new() -> Self {
        Self {
            inner: [
                Value::None,
                Value::None,
                Value::None,
                Value::None,
                Value::None,
            ],
        }
    }

    pub fn read(&self, reg: Register) -> &Value {
        &self.inner[reg.index()]
    }

    pub fn write(&mut self, reg: Register, value: Value) {
        self.inner[reg.index()] = value;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}
