use std::sync::Arc;

use anyhow::{Result, anyhow};
use vm_isa::{op_code::OpCode, register::Register, value::Value};

macro_rules! try_parse {
    ($s:expr, $($ty:ty => $var:ident),* $(,)?) => {
        $(if let Ok(v) = $s.parse::<$ty>() { return Ok(Value::$var(v)); })*
    };
}

pub fn to_value(str: &str) -> Result<Value> {
    if let Some(reg) = str.strip_prefix('$') {
        return match reg {
            "a1" => Ok(Value::Register(Arc::new(Register::A1(Value::None)))),
            "a2" => Ok(Value::Register(Arc::new(Register::A2(Value::None)))),
            "a3" => Ok(Value::Register(Arc::new(Register::A3(Value::None)))),
            "a4" => Ok(Value::Register(Arc::new(Register::A4(Value::None)))),
            "a5" => Ok(Value::Register(Arc::new(Register::A5(Value::None)))),
            _ => panic!("unknown register: {str}"),
        };
    }

    if let Some(inner) = str.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return unescape::unescape(inner)
            .map(Value::String)
            .ok_or_else(|| anyhow!("invalid escape sequence: {str}"));
    }

    match str {
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        "" => Err(anyhow!("?")),
        s => {
            try_parse!(s, u8 => U8, u32 => U32, u128 => U128, i32 => I32, i128 => I128, f32 => F32, f64 => F64);
            Err(anyhow!("unrecognized token: {s}"))
        }
    }
}

pub fn reader(line: String, cmds: &mut Vec<OpCode>) -> Result<()> {
    let mut cmd_vec: Vec<&str> = line.split(";").collect();
    cmd_vec.pop();
    for cmd in cmd_vec {
        let cmd_vecs: Vec<&str> = cmd.trim_start().split(" ").collect();

        match *cmd_vecs.first().ok_or(anyhow!("No value!"))? {
            "add" => {
                let first = to_value(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                let second = to_value(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
                let output = to_value(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
                let output = match output {
                    Value::Register(v) => (*v).clone(),
                    _ => {
                        panic!()
                    }
                };
                cmds.push(OpCode::Add(first, second, output));
            }

            "sub" => {
                let first = to_value(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                let second = to_value(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
                let output = to_value(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
                let output = match output {
                    Value::Register(v) => (*v).clone(),
                    _ => {
                        panic!()
                    }
                };
                cmds.push(OpCode::Sub(first, second, output));
            }

            "mul" => {
                let first = to_value(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                let second = to_value(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
                let output = to_value(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
                let output = match output {
                    Value::Register(v) => (*v).clone(),
                    _ => {
                        panic!()
                    }
                };
                cmds.push(OpCode::Mul(first, second, output));
            }
            "div" => {
                let first = to_value(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                let second = to_value(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
                let output = to_value(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
                let output = match output {
                    Value::Register(v) => (*v).clone(),
                    _ => {
                        panic!()
                    }
                };
                cmds.push(OpCode::Div(first, second, output));
            }
            "_println" => {
                let value = to_value(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                cmds.push(OpCode::Println(value));
            }
            "_print" => {
                let value = to_value(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                cmds.push(OpCode::Print(value));
            }
            _ => (),
        }
    }
    Ok(())
}
