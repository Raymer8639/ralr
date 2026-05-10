//! Synchronous instruction execution loop.
//! 中文：同步指令执行循环。
//!
//! Resolves operands (literals or register references) against the
//! register file and dispatches each opcode.
//! 中文：根据寄存器文件解析操作数（字面量或寄存器引用）并分派每条操作码。

use std::io::{self, Write};

use anyhow::Result;
use vm_isa::{
    op_code::{BinOp, Expr, IoOp, OpCode, Operand, UnOp},
    register::Registers,
    value::Value,
};

/// Resolves an operand to a concrete value.
/// 中文：将操作数解析为具体值。
///
/// Literals are cloned; register references are looked up from `regs`.
/// 中文：字面量被克隆；寄存器引用从 `regs` 中查找。
fn resolve(operand: &Operand, regs: &Registers) -> Value {
    match operand {
        Operand::Literal(v) => v.clone(),
        Operand::Register(r) => regs.read(*r).clone(),
    }
}

/// Recursively evaluates an expression tree against the register file.
/// 中文：根据寄存器文件递归计算表达式树。
///
/// Register references are resolved from the current register state. All
/// arithmetic, comparison, logical, bitwise, and shift operators are
/// dispatched here.
/// 中文：寄存器引用从当前寄存器状态解析。所有算术、比较、逻辑、位和移位运算符都在此处分派。
fn eval_expr(expr: &Expr, regs: &Registers) -> Value {
    match expr {
        Expr::Literal(v) => v.clone(),
        Expr::Register(r) => regs.read(*r).clone(),
        Expr::Binary(lhs, op, rhs) => {
            let lhs_val = eval_expr(lhs, regs);
            let rhs_val = eval_expr(rhs, regs);
            match op {
                BinOp::Add => lhs_val + rhs_val,
                BinOp::Sub => lhs_val - rhs_val,
                BinOp::Mul => lhs_val * rhs_val,
                BinOp::Div => lhs_val / rhs_val,
                BinOp::Rem => lhs_val % rhs_val,
                BinOp::Eq => lhs_val.eq_val(&rhs_val),
                BinOp::Ne => lhs_val.ne_val(&rhs_val),
                BinOp::Lt => lhs_val.lt_val(&rhs_val),
                BinOp::Le => lhs_val.le_val(&rhs_val),
                BinOp::Gt => lhs_val.gt_val(&rhs_val),
                BinOp::Ge => lhs_val.ge_val(&rhs_val),
                BinOp::And => lhs_val.and_val(&rhs_val),
                BinOp::Or => lhs_val.or_val(&rhs_val),
                BinOp::BitAnd => lhs_val & rhs_val,
                BinOp::BitOr => lhs_val | rhs_val,
                BinOp::BitXor => lhs_val ^ rhs_val,
                BinOp::Shl => lhs_val << rhs_val,
                BinOp::Shr => lhs_val >> rhs_val,
            }
        }
        Expr::Unary(op, inner) => {
            let val = eval_expr(inner, regs);
            match op {
                UnOp::Neg => -val,
                UnOp::Not => val.logical_not(),
                UnOp::BitNot => val.bitwise_not(),
            }
        }
    }
}

/// Parses a string read from stdin into a [`Value`], using the same
/// numeric-inference chain as the assembler: U8 → U32 → U128 → I32 →
/// I128 → F32 → F64. Recognizes "true"/"false" as Bool. Falls back to
/// String for anything else.
/// 中文：将从标准输入读取的字符串解析为 [`Value`]，使用与汇编器相同的数字推断链：U8 → U32 → U128 → I32 → I128 → F32 → F64。识别 "true"/"false" 为 Bool。其他情况回退到 String。
fn parse_stdin_value(s: &str) -> Value {
    let s = s.trim();
    if s.is_empty() {
        return Value::String(String::new());
    }
    if s == "true" {
        return Value::Bool(true);
    }
    if s == "false" {
        return Value::Bool(false);
    }
    if let Ok(v) = s.parse::<u8>() {
        return Value::U8(v);
    }
    if let Ok(v) = s.parse::<u32>() {
        return Value::U32(v);
    }
    if let Ok(v) = s.parse::<u128>() {
        return Value::U128(v);
    }
    if let Ok(v) = s.parse::<i32>() {
        return Value::I32(v);
    }
    if let Ok(v) = s.parse::<i128>() {
        return Value::I128(v);
    }
    if let Ok(v) = s.parse::<f32>() {
        return Value::F32(v);
    }
    if let Ok(v) = s.parse::<f64>() {
        return Value::F64(v);
    }
    Value::String(s.to_string())
}

/// Executes a sequence of opcodes against a register file.
/// 中文：对寄存器文件执行一系列操作码。
///
/// Arithmetic instructions resolve both operands, perform the operation,
/// and write the result into the destination register. Print instructions
/// resolve and write directly to stdout. Blocks recurse with the same
/// register context. Expressions evaluate against the register file.
/// 中文：算术指令解析两个操作数，执行运算，并将结果写入目标寄存器。打印指令解析后直接写入标准输出。块以相同的寄存器上下文递归执行。表达式根据寄存器文件进行计算。
///
/// # Panics
///
/// Arithmetic operators on [`Value`] panic on type mismatches (e.g.
/// `I32 + String`). This kills the VM — there is no error recovery.
/// 中文：[`Value`] 上的算术运算符在类型不匹配时会 panic（例如 `I32 + String`）。这会终止虚拟机 — 没有错误恢复。
pub fn runner(cmds: &[OpCode], regs: &mut Registers) -> Result<()> {
    for cmd in cmds {
        match cmd {
            OpCode::Add(a, b, dest) => {
                let result = resolve(a, regs) + resolve(b, regs);
                regs.write(*dest, result);
            }
            OpCode::Sub(a, b, dest) => {
                let result = resolve(a, regs) - resolve(b, regs);
                regs.write(*dest, result);
            }
            OpCode::Mul(a, b, dest) => {
                let result = resolve(a, regs) * resolve(b, regs);
                regs.write(*dest, result);
            }
            OpCode::Div(a, b, dest) => {
                let result = resolve(a, regs) / resolve(b, regs);
                regs.write(*dest, result);
            }
            OpCode::Println(val) => {
                println!("{}", resolve(val, regs));
            }
            OpCode::Print(val) => {
                print!("{}", resolve(val, regs));
                io::stdout().flush().unwrap();
            }
            OpCode::Block(inner) => {
                runner(inner, regs)?;
            }
            OpCode::Expr(expr, dest) => {
                let result = eval_expr(expr, regs);
                regs.write(*dest, result);
            }
            OpCode::If(cond, then_body, else_body) => {
                let cond_val = eval_expr(cond, regs);
                match cond_val {
                    Value::Bool(true) => runner(then_body, regs)?,
                    Value::Bool(false) => {
                        if let Some(else_b) = else_body {
                            runner(else_b, regs)?;
                        }
                    }
                    other => panic!("if condition must be Bool, got {other:?}"),
                }
            }
            OpCode::IO(op) => match op {
                // Unified I/O dispatch.
                // 中文：统一的 I/O 分派。
                IoOp::Write(val) => {
                    // Print operand without trailing newline, flush stdout.
                    // 中文：打印操作数，末尾不换行，刷新标准输出。
                    print!("{}", resolve(val, regs));
                    io::stdout().flush().unwrap();
                }
                IoOp::Writeln(val) => {
                    // Print operand followed by a newline.
                    // 中文：打印操作数并换行。
                    println!("{}", resolve(val, regs));
                }
                IoOp::Read(reg) => {
                    // Read stdin line, parse as Value via type-inference chain.
                    // 中文：从标准输入读取一行，通过类型推断链解析为 Value。
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    let val = parse_stdin_value(&input);
                    regs.write(*reg, val);
                }
                IoOp::Readln(reg) => {
                    // Read stdin line, store as String, strip trailing newline.
                    // 中文：从标准输入读取一行，作为 String 存入，去除尾部换行符。
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    if input.ends_with('\n') {
                        input.pop();
                        if input.ends_with('\r') {
                            input.pop();
                        }
                    }
                    regs.write(*reg, Value::String(input));
                }
            },
            OpCode::While(expr, cmds) => loop {
                let value = eval_expr(expr, regs);
                match value {
                    Value::Bool(true) => {
                        runner(cmds, regs)?;
                    }
                    Value::Bool(false) => {
                        break;
                    }
                    error => panic!("{:?}", error),
                }
            },
        }
    }
    Ok(())
}
