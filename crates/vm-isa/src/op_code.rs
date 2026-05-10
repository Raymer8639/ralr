//! Instruction definitions: [`OpCode`] and [`Operand`].
//! 中文：指令定义：[`OpCode`] 和 [`Operand`]。
//!
//! Each instruction is serialized via bincode into `.abin` files by the
//! assembler and deserialized back by the VM at runtime.
//! 中文：每条指令由汇编器通过 bincode 序列化为 `.abin` 文件，并在运行时由虚拟机反序列化。

use serde::{Deserialize, Serialize};

pub use crate::expr::{BinOp, Expr, UnOp};
use crate::{register::Register, value::Value};

/// An operand to an instruction — either an immediate literal value or
/// a register reference.
/// 中文：指令的操作数 — 立即数字面值或寄存器引用。
///
/// This separation at the type level eliminates the `Arc<Register>`
/// indirection that existed when `Value::Register` carried payloads.
/// 中文：此类型层面的分离消除了当 `Value::Register` 携带数据时存在的 `Arc<Register>` 间接引用。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Operand {
    Literal(Value),
    Register(Register),
}

/// I/O operation kind.
/// 中文：I/O 操作类型。
///
/// Unified I/O sub-operations: output with/without newline, and input
/// with type inference or raw line reading.
/// 中文：统一的 I/O 子操作：带/不带换行的输出，以及带类型推断或原始行读取的输入。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum IoOp {
    /// Print operand without trailing newline (flushes stdout).
    /// 中文：打印操作数，末尾不换行（刷新标准输出）。
    Write(Operand),
    /// Print operand followed by a newline.
    /// 中文：打印操作数并换行。
    Writeln(Operand),
    /// Read a line from stdin, parse as a Value, store in register.
    /// 中文：从标准输入读取一行，解析为 Value，存入寄存器。
    Read(Register),
    /// Read a line from stdin, store as String in register.
    /// 中文：从标准输入读取一行，作为 String 存入寄存器。
    Readln(Register),
}

/// A single VM instruction.
/// 中文：单条虚拟机指令。
///
/// Arithmetic instructions take two source operands and a destination
/// register. Print instructions take one operand and write it to stdout.
/// 中文：算术指令接受两个源操作数和一个目标寄存器。打印指令接受一个操作数并将其写入标准输出。
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum OpCode {
    Add(Operand, Operand, Register),
    Sub(Operand, Operand, Register),
    Mul(Operand, Operand, Register),
    Div(Operand, Operand, Register),
    /// Prints the operand followed by a newline.
    /// 中文：打印操作数并换行。
    Println(Operand),
    /// Prints the operand without a trailing newline (flushes stdout).
    /// 中文：打印操作数，末尾不换行（刷新标准输出）。
    Print(Operand),
    /// A nested sequence of instructions executed within the enclosing
    /// register context. Supports arbitrary nesting depth.
    /// 中文：在封闭寄存器上下文中执行的嵌套指令序列。支持任意嵌套深度。
    Block(Vec<OpCode>),
    /// Evaluates an expression tree and writes the result into the
    /// destination register.
    /// 中文：计算表达式树并将结果写入目标寄存器。
    Expr(Expr, Register),
    /// Conditional branch. Evaluates the expression; if `Bool(true)`,
    /// executes the then-body. If `Bool(false)`, executes the optional
    /// else-body. Panics on non-Bool conditions.
    /// 中文：条件分支。计算表达式；若 `Bool(true)` 则执行 then 体，若 `Bool(false)` 则执行可选的 else 体。非 Bool 条件会 panic。
    If(Expr, Vec<OpCode>, Option<Vec<OpCode>>),
    /// Unified I/O operation — write, writeln, read, readln.
    /// 中文：统一的 I/O 操作 — write、writeln、read、readln。
    IO(IoOp),
    /// A loop statement that evaluates an expression; if the result is `Bool(true)`, the loop body is executed once,
    /// and this process repeats until the result becomes `Bool(false)`, at which point it terminates.
    /// 中文： 循环语句，计算表达式；若`Bool(true)`则执行一次本体，一直重复，直到`Bool(false)`时停止
    While(Expr, Vec<OpCode>),
}
