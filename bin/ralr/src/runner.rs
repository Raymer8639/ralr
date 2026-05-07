//! Synchronous instruction execution loop.
//! 中文：同步指令执行循环。
//! 日本語：同期命令実行ループ。
//! Русский: Синхронный цикл выполнения инструкций.
//!
//! Resolves operands (literals or register references) against the
//! register file and dispatches each opcode.
//! 中文：根据寄存器文件解析操作数（字面量或寄存器引用）并分派每条操作码。
//! 日本語：レジスタファイルに対してオペランド（リテラルまたはレジスタ参照）を解決し、各オペコードをディスパッチします。
//! Русский: Разрешает операнды (литералы или ссылки на регистры) относительно
//! файла регистров и диспетчеризует каждый опкод.

use std::io::{self, Write};

use anyhow::Result;
use vm_isa::{
    op_code::{BinOp, Expr, OpCode, Operand, UnOp},
    register::Registers,
    value::Value,
};

/// Resolves an operand to a concrete value.
/// 中文：将操作数解析为具体值。
/// 日本語：オペランドを具体的な値に解決します。
/// Русский: Разрешает операнд в конкретное значение.
///
/// Literals are cloned; register references are looked up from `regs`.
/// 中文：字面量被克隆；寄存器引用从 `regs` 中查找。
/// 日本語：リテラルはクローンされ、レジスタ参照は `regs` から検索されます。
/// Русский: Литералы клонируются; ссылки на регистры извлекаются из `regs`.
fn resolve(operand: &Operand, regs: &Registers) -> Value {
    match operand {
        Operand::Literal(v) => v.clone(),
        Operand::Register(r) => regs.read(*r).clone(),
    }
}

/// Recursively evaluates an expression tree against the register file.
/// 中文：根据寄存器文件递归计算表达式树。
/// 日本語：レジスタファイルに対して式ツリーを再帰的に評価します。
/// Русский: Рекурсивно вычисляет дерево выражения относительно файла регистров.
///
/// Register references are resolved from the current register state. All
/// arithmetic, comparison, logical, bitwise, and shift operators are
/// dispatched here.
/// 中文：寄存器引用从当前寄存器状态解析。所有算术、比较、逻辑、位和移位运算符都在此处分派。
/// 日本語：レジスタ参照は現在のレジスタ状態から解決されます。すべての算術、比較、論理、ビット、シフト演算子がここでディスパッチされます。
/// Русский: Ссылки на регистры разрешаются из текущего состояния регистров. Все
/// арифметические, сравнительные, логические, побитовые и сдвиговые операторы диспетчеризуются здесь.
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

/// Executes a sequence of opcodes against a register file.
/// 中文：对寄存器文件执行一系列操作码。
/// 日本語：レジスタファイルに対して一連のオペコードを実行します。
/// Русский: Выполняет последовательность опкодов на файле регистров.
///
/// Arithmetic instructions resolve both operands, perform the operation,
/// and write the result into the destination register. Print instructions
/// resolve and write directly to stdout. Blocks recurse with the same
/// register context. Expressions evaluate against the register file.
/// 中文：算术指令解析两个操作数，执行运算，并将结果写入目标寄存器。打印指令解析后直接写入标准输出。块以相同的寄存器上下文递归执行。表达式根据寄存器文件进行计算。
/// 日本語：算術命令は両方のオペランドを解決し、演算を実行し、結果を宛先レジスタに書き込みます。印刷命令は解決して直接標準出力に書き込みます。ブロックは同じレジスタコンテキストで再帰します。式はレジスタファイルに対して評価されます。
/// Русский: Арифметические инструкции разрешают оба операнда, выполняют операцию и
/// записывают результат в целевой регистр. Инструкции печати разрешают и выводят
/// непосредственно в stdout. Блоки рекурсивно выполняются с тем же контекстом регистров.
/// Выражения вычисляются относительно файла регистров.
///
/// # Panics
///
/// Arithmetic operators on [`Value`] panic on type mismatches (e.g.
/// `I32 + String`). This kills the VM — there is no error recovery.
/// 中文：[`Value`] 上的算术运算符在类型不匹配时会 panic（例如 `I32 + String`）。这会终止虚拟机 — 没有错误恢复。
/// 日本語：[`Value`] の算術演算子は型の不一致でパニックします（例：`I32 + String`）。これにより VM は停止します — エラー回復はありません。
/// Русский: Арифметические операторы на [`Value`] паникуют при несовпадении типов
/// (например, `I32 + String`). Это убивает ВМ — восстановление после ошибок отсутствует.
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
        }
    }
    Ok(())
}
