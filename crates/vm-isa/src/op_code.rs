use serde::{Deserialize, Serialize};

use crate::{register::Register, value::Value};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Operand {
    Literal(Value),
    Register(Register),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OpCode {
    Add(Operand, Operand, Register),
    Sub(Operand, Operand, Register),
    Mul(Operand, Operand, Register),
    Div(Operand, Operand, Register),
    Println(Operand),
    Print(Operand),
}
