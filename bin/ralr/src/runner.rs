use std::io::{self, Write};

use anyhow::Result;
use vm_isa::{
    op_code::{OpCode, Operand},
    register::Registers,
    value::Value,
};

fn resolve(operand: &Operand, regs: &Registers) -> Value {
    match operand {
        Operand::Literal(v) => v.clone(),
        Operand::Register(r) => regs.read(*r).clone(),
    }
}

pub fn runner(cmds: Vec<OpCode>, mut regs: Registers) -> Result<()> {
    for cmd in cmds {
        match cmd {
            OpCode::Add(a, b, dest) => {
                let result = resolve(&a, &regs) + resolve(&b, &regs);
                regs.write(dest, result);
            }
            OpCode::Sub(a, b, dest) => {
                let result = resolve(&a, &regs) - resolve(&b, &regs);
                regs.write(dest, result);
            }
            OpCode::Mul(a, b, dest) => {
                let result = resolve(&a, &regs) * resolve(&b, &regs);
                regs.write(dest, result);
            }
            OpCode::Div(a, b, dest) => {
                let result = resolve(&a, &regs) / resolve(&b, &regs);
                regs.write(dest, result);
            }
            OpCode::Println(val) => {
                println!("{}", resolve(&val, &regs));
            }
            OpCode::Print(val) => {
                print!("{}", resolve(&val, &regs));
                io::stdout().flush().unwrap();
            }
        }
    }
    Ok(())
}
