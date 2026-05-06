use anyhow::{Result, anyhow};
use std::char;
use vm_isa::{
    op_code::{OpCode, Operand},
    register::Register,
    value::Value,
};

macro_rules! try_parse {
    ($s:expr, $($ty:ty => $var:ident),* $(,)?) => {
        $(if let Ok(v) = $s.parse::<$ty>() { return Ok(Value::$var(v)); })*
    };
}

fn unescape_str(s: &str) -> Option<String> {
    let mut chars = s.chars();
    let mut out = String::with_capacity(s.len());

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next()? {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            '0' => out.push('\0'),
            'x' => {
                let hi = chars.next()?.to_digit(16)?;
                let lo = chars.next()?.to_digit(16)?;
                let byte = (hi * 16 + lo) as u8;
                out.push(byte as char);
            }
            'u' => {
                if chars.next()? != '{' {
                    return None;
                }
                let mut hex = String::new();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    if !c.is_ascii_hexdigit() {
                        return None;
                    }
                    hex.push(c);
                }
                let codepoint = u32::from_str_radix(&hex, 16).ok()?;
                out.push(char::from_u32(codepoint)?);
            }
            _ => return None,
        }
    }
    Some(out)
}

fn to_value(str: &str) -> Result<Value> {
    if let Some(inner) = str.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return unescape_str(inner)
            .map(Value::String)
            .ok_or_else(|| anyhow!("invalid escape sequence: {str}"));
    }

    match str {
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        "" => Err(anyhow!("empty token")),
        s => {
            try_parse!(s, u8 => U8, u32 => U32, u128 => U128, i32 => I32, i128 => I128, f32 => F32, f64 => F64);
            Err(anyhow!("unrecognized token: {s}"))
        }
    }
}

fn to_register(str: &str) -> Result<Register> {
    if let Some(reg) = str.strip_prefix('$') {
        match reg {
            "a1" => Ok(Register::A1),
            "a2" => Ok(Register::A2),
            "a3" => Ok(Register::A3),
            "a4" => Ok(Register::A4),
            "a5" => Ok(Register::A5),
            _ => panic!("unknown register: {str}"),
        }
    } else {
        panic!("expected register, got: {str}")
    }
}

fn to_operand(str: &str) -> Result<Operand> {
    if str.starts_with('$') {
        Ok(Operand::Register(to_register(str)?))
    } else {
        Ok(Operand::Literal(to_value(str)?))
    }
}

fn tokenize(s: &str) -> Vec<&str> {
    let s = s.trim();
    let mut tokens = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            tokens.push(&s[start..i]);
        } else {
            let start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(&s[start..i]);
        }
    }
    tokens
}

pub fn reader(line: String, cmds: &mut Vec<OpCode>) -> Result<()> {
    let mut cmd_vec: Vec<&str> = line.split(";").collect();
    cmd_vec.pop();
    for cmd in cmd_vec {
        let cmd_vecs = tokenize(cmd);

        match *cmd_vecs.first().ok_or(anyhow!("No value!"))? {
            "add" => {
                let first = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                let second = to_operand(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
                let dest = to_register(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
                cmds.push(OpCode::Add(first, second, dest));
            }

            "sub" => {
                let first = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                let second = to_operand(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
                let dest = to_register(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
                cmds.push(OpCode::Sub(first, second, dest));
            }

            "mul" => {
                let first = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                let second = to_operand(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
                let dest = to_register(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
                cmds.push(OpCode::Mul(first, second, dest));
            }
            "div" => {
                let first = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                let second = to_operand(cmd_vecs.get(2).ok_or(anyhow!("No value!"))?)?;
                let dest = to_register(cmd_vecs.get(3).ok_or(anyhow!("No value!"))?)?;
                cmds.push(OpCode::Div(first, second, dest));
            }
            "_println" => {
                let value = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                cmds.push(OpCode::Println(value));
            }
            "_print" => {
                let value = to_operand(cmd_vecs.get(1).ok_or(anyhow!("No value!"))?)?;
                cmds.push(OpCode::Print(value));
            }
            _ => (),
        }
    }
    Ok(())
}
