use serde::{Deserialize, Serialize};

use crate::{register::Register, value::Value};

#[derive(Serialize, Deserialize, Debug)]
pub enum OpCode {
    Add(Value, Value, Register),
    Sub(Value, Value, Register),
    Mul(Value, Value, Register),
    Div(Value, Value, Register),
}
