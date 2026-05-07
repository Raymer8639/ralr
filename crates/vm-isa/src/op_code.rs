//! Instruction definitions: [`OpCode`] and [`Operand`].
//! 中文：指令定义：[`OpCode`] 和 [`Operand`]。
//! 日本語：命令定義：[`OpCode`] と [`Operand`]。
//! Русский: Определения инструкций: [`OpCode`] и [`Operand`].
//!
//! Each instruction is serialized via bincode into `.abin` files by the
//! assembler and deserialized back by the VM at runtime.
//! 中文：每条指令由汇编器通过 bincode 序列化为 `.abin` 文件，并在运行时由虚拟机反序列化。
//! 日本語：各命令はアセンブラによって bincode で `.abin` ファイルにシリアライズされ、VM によって実行時にデシリアライズされます。
//! Русский: Каждая инструкция сериализуется через bincode в файлы `.abin` ассемблером
//! и десериализуется обратно виртуальной машиной во время выполнения.

use serde::{Deserialize, Serialize};

pub use crate::expr::{BinOp, Expr, UnOp};
use crate::{register::Register, value::Value};

/// An operand to an instruction — either an immediate literal value or
/// a register reference.
/// 中文：指令的操作数 — 立即数字面值或寄存器引用。
/// 日本語：命令のオペランド — 即値リテラルまたはレジスタ参照。
/// Русский: Операнд инструкции — либо непосредственное литеральное значение, либо ссылка на регистр.
///
/// This separation at the type level eliminates the `Arc<Register>`
/// indirection that existed when `Value::Register` carried payloads.
/// 中文：此类型层面的分离消除了当 `Value::Register` 携带数据时存在的 `Arc<Register>` 间接引用。
/// 日本語：この型レベルでの分離により、`Value::Register` がペイロードを持っていた時代の `Arc<Register>` の間接参照が排除されます。
/// Русский: Это разделение на уровне типов устраняет косвенность `Arc<Register>`,
/// которая существовала, когда `Value::Register` содержал полезную нагрузку.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Operand {
    Literal(Value),
    Register(Register),
}

/// A single VM instruction.
/// 中文：单条虚拟机指令。
/// 日本語：単一の VM 命令。
/// Русский: Одна инструкция виртуальной машины.
///
/// Arithmetic instructions take two source operands and a destination
/// register. Print instructions take one operand and write it to stdout.
/// 中文：算术指令接受两个源操作数和一个目标寄存器。打印指令接受一个操作数并将其写入标准输出。
/// 日本語：算術命令は 2 つのソースオペランドと宛先レジスタを取ります。印刷命令は 1 つのオペランドを取り、標準出力に書き込みます。
/// Русский: Арифметические инструкции принимают два исходных операнда и целевой регистр.
/// Инструкции печати принимают один операнд и выводят его в stdout.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum OpCode {
    Add(Operand, Operand, Register),
    Sub(Operand, Operand, Register),
    Mul(Operand, Operand, Register),
    Div(Operand, Operand, Register),
    /// Prints the operand followed by a newline.
    /// 中文：打印操作数并换行。
    /// 日本語：オペランドを出力し、改行します。
    /// Русский: Выводит операнд с последующим переводом строки.
    Println(Operand),
    /// Prints the operand without a trailing newline (flushes stdout).
    /// 中文：打印操作数，末尾不换行（刷新标准输出）。
    /// 日本語：オペランドを出力し、末尾の改行なし（標準出力をフラッシュ）。
    /// Русский: Выводит операнд без завершающего перевода строки (сбрасывает stdout).
    Print(Operand),
    /// A nested sequence of instructions executed within the enclosing
    /// register context. Supports arbitrary nesting depth.
    /// 中文：在封闭寄存器上下文中执行的嵌套指令序列。支持任意嵌套深度。
    /// 日本語：囲まれたレジスタコンテキスト内で実行されるネストされた命令シーケンス。任意のネスト深度をサポートします。
    /// Русский: Вложенная последовательность инструкций, выполняемая в охватывающем
    /// контексте регистров. Поддерживает произвольную глубину вложенности.
    Block(Vec<OpCode>),
    /// Evaluates an expression tree and writes the result into the
    /// destination register.
    /// 中文：计算表达式树并将结果写入目标寄存器。
    /// 日本語：式ツリーを評価し、結果を宛先レジスタに書き込みます。
    /// Русский: Вычисляет дерево выражения и записывает результат в целевой регистр.
    Expr(Expr, Register),
}
