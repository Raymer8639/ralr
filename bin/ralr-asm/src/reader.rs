use std::sync::Arc;

use anyhow::{Result, anyhow};
use vm_isa::{op_code::OpCode, register::Register, value::Value};

pub fn to_value(str: &str) -> Result<Value> {
    if str.chars().next().ok_or(anyhow!("?"))? == '$' {
        let str: String = str.chars().skip(1).map(|c| c.to_string()).collect();
        match str.as_str() {
            "a1" => Ok(Value::Register(Arc::new(Register::A1(Value::None)))),
            "a2" => Ok(Value::Register(Arc::new(Register::A2(Value::None)))),
            "a3" => Ok(Value::Register(Arc::new(Register::A3(Value::None)))),
            "a4" => Ok(Value::Register(Arc::new(Register::A4(Value::None)))),
            "a5" => Ok(Value::Register(Arc::new(Register::A5(Value::None)))),
            _ => {
                panic!();
            }
        }
    } else {
        if str == "true" {
            return Ok(Value::Bool(true));
        }
        if str == "false" {
            return Ok(Value::Bool(false));
        }
        if let Ok(v) = str.parse::<u8>() {
            return Ok(Value::U8(v));
        }
        if let Ok(v) = str.parse::<u32>() {
            return Ok(Value::U32(v));
        }
        if let Ok(v) = str.parse::<u128>() {
            return Ok(Value::U128(v));
        }
        if let Ok(v) = str.parse::<i32>() {
            return Ok(Value::I32(v));
        }
        if let Ok(v) = str.parse::<i128>() {
            return Ok(Value::I128(v));
        }
        if let Ok(v) = str.parse::<f32>() {
            return Ok(Value::F32(v));
        }
        if let Ok(v) = str.parse::<f64>() {
            return Ok(Value::F64(v));
        }
        Ok(Value::String(str.to_string()))
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
            _ => (),
        }
    }
    Ok(())
}
