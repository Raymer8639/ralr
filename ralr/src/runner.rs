//! Synchronous instruction execution loop.
//! 中文：同步指令执行循环。
//!
//! Resolves operands (literals or register references) against the
//! register file and dispatches each opcode.
//! 中文：根据寄存器文件解析操作数（字面量或寄存器引用）并分派每条操作码。

use std::io::{self, Write};

use ahash::AHashMap;
use anyhow::Result;
use vm_isa::{
    function::FnDef,
    op_code::{IoOp, OpCode, Operand},
    register::{Register, Registers},
    value::Value,
    variable::Variable,
};

/// Resolves an operand to a concrete value.
/// 中文：将操作数解析为具体值。
///
/// Literals are cloned; register references are looked up from `regs`;
/// variable references are looked up from the variable hashmap.
/// 中文：字面量被克隆；寄存器引用从 `regs` 中查找；变量引用从变量哈希表中查找。
fn resolve(operand: &Operand, regs: &Registers, variables: &AHashMap<String, Variable>) -> Value {
    match operand {
        Operand::Literal(v) => v.clone(),
        Operand::Register(r) => regs.read(*r).clone(),
        Operand::Variable(name, _) => {
            let var = variables
                .get(name)
                .unwrap_or_else(|| panic!("variable not found: {name}"));
            var.value.clone()
        }
    }
}

/// Resolves an operand to a shared reference without cloning.
/// 中文：将操作数解析为共享引用，不进行克隆。
///
/// Used in I/O paths where only a `&Value` is needed (e.g., `Display`).
/// 中文：用于仅需要 `&Value` 的 I/O 路径（例如 `Display`）。
fn resolve_ref<'a>(
    operand: &'a Operand,
    regs: &'a Registers,
    variables: &'a AHashMap<String, Variable>,
) -> &'a Value {
    match operand {
        Operand::Literal(v) => v,
        Operand::Register(r) => regs.read(*r),
        Operand::Variable(name, _) => {
            let var = variables
                .get(name)
                .unwrap_or_else(|| panic!("variable not found: {name}"));
            &var.value
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
/// Executes a sequence of opcodes against a register file, variable
/// hashmap, and function table.
/// 中文：根据寄存器文件、变量哈希表和函数表执行一系列操作码。
pub fn runner(
    cmds: &[OpCode],
    regs: &mut Registers,
    variables: &mut AHashMap<String, Variable>,
    functions: &mut AHashMap<String, FnDef>,
) -> Result<()> {
    for cmd in cmds {
        match cmd {
            OpCode::Add(a, b, dest) => {
                let result = resolve(a, regs, variables) + resolve(b, regs, variables);
                regs.write(*dest, result);
            }
            OpCode::Sub(a, b, dest) => {
                let result = resolve(a, regs, variables) - resolve(b, regs, variables);
                regs.write(*dest, result);
            }
            OpCode::Mul(a, b, dest) => {
                let result = resolve(a, regs, variables) * resolve(b, regs, variables);
                regs.write(*dest, result);
            }
            OpCode::Div(a, b, dest) => {
                let result = resolve(a, regs, variables) / resolve(b, regs, variables);
                regs.write(*dest, result);
            }
            OpCode::Println(val) => {
                println!("{}", resolve_ref(val, regs, variables));
            }
            OpCode::Print(val) => {
                print!("{}", resolve_ref(val, regs, variables));
                io::stdout().flush().unwrap();
            }
            OpCode::Block(inner) => {
                runner(inner, regs, variables, functions)?;
            }
            OpCode::Expr(expr, dest) => {
                let result = expr.eval_expr(expr, regs, variables);
                match dest {
                    Operand::Register(reg) => {
                        regs.write(*reg, result);
                    }
                    Operand::Variable(name, _) => {
                        // Variable assignment — update the existing variable's
                        // value in the hashmap. Panics if the variable is
                        // immutable or does not exist.
                        // 中文：变量赋值 — 更新哈希表中现有变量的值。若变量不可变或不存在则 panic。
                        let var = variables
                            .get_mut(name)
                            .unwrap_or_else(|| panic!("variable not found: {name}"));
                        if !var.is_mut {
                            panic!("cannot assign to immutable variable: {name}");
                        }
                        var.value = result;
                    }
                    Operand::Literal(_) => {
                        panic!("cannot assign expression result to a literal");
                    }
                }
            }
            OpCode::If(cond, then_body, else_body) => {
                let cond_val = cond.eval_expr(cond, regs, variables);
                match cond_val {
                    Value::Bool(true) => runner(then_body, regs, variables, functions)?,
                    Value::Bool(false) => {
                        if let Some(else_b) = else_body {
                            runner(else_b, regs, variables, functions)?;
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
                    print!("{}", resolve_ref(val, regs, variables));
                    io::stdout().flush().unwrap();
                }
                IoOp::Writeln(val) => {
                    // Print operand followed by a newline.
                    // 中文：打印操作数并换行。
                    println!("{}", resolve_ref(val, regs, variables));
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
                let value = expr.eval_expr(expr, regs, variables);
                match value {
                    Value::Bool(true) => {
                        runner(cmds, regs, variables, functions)?;
                    }
                    Value::Bool(false) => {
                        break;
                    }
                    other => panic!("while condition must be Bool, got {other:?}"),
                }
            },
            OpCode::Variable(var_name, var) => {
                // take() avoids cloning the SystemVarBuffer value.
                // 中文：take() 避免克隆 SystemVarBuffer 的值。
                let val = regs.take(Register::SystemVarBuffer);
                variables.insert(
                    (*var_name).clone(),
                    Variable {
                        is_mut: var.is_mut,
                        value: val,
                    },
                );
            }
            OpCode::FnDef { name, params, body } => {
                // Register the function definition in the function table.
                // 中文：在函数表中注册函数定义。
                functions.insert(
                    name.clone(),
                    FnDef {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }
            OpCode::Call { name, args, dest } => {
                // Look up the function and verify parameter count.
                // Clone to release the immutable borrow before the recursive
                // runner call takes a mutable borrow.
                // 中文：查找函数并验证参数数量。克隆以释放不可变借用。
                let fn_def = functions
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("function not found: {name}"));
                if fn_def.params.len() != args.len() {
                    panic!(
                        "function '{name}' expects {} arguments, got {}",
                        fn_def.params.len(),
                        args.len()
                    );
                }
                // Evaluate arguments first — they may reference variables
                // that share names with parameters, and removing then
                // re-evaluating would lose the original values.
                // 中文：首先计算参数 — 参数可能引用与形参同名的变量，
                // 先移除再重新计算会丢失原始值。
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|arg_expr| arg_expr.eval_expr(arg_expr, regs, variables))
                    .collect();
                // Save variables that would be shadowed by parameters.
                // 中文：保存会被参数遮蔽的变量。
                let saved: Vec<(String, Option<Variable>)> = fn_def
                    .params
                    .iter()
                    .map(|p| (p.clone(), variables.remove(p)))
                    .collect();
                // Bind pre-evaluated argument values to parameter names.
                // 中文：将预计算的参数值绑定到形参名称。
                for (param_name, arg_val) in fn_def.params.iter().zip(arg_vals) {
                    variables.insert(
                        param_name.clone(),
                        Variable {
                            is_mut: false,
                            value: arg_val,
                        },
                    );
                }
                // Execute the function body.
                // 中文：执行函数体。
                runner(&fn_def.body, regs, variables, functions)?;
                // Retrieve the return value from SystemVarBuffer
                // via take() to avoid cloning the Value.
                // 中文：通过 take() 获取返回值，避免克隆 Value。
                let ret_val = regs.take(Register::SystemVarBuffer);
                // Restore shadowed variables.
                // 中文：恢复被遮蔽的变量。
                for (param_name, old_var) in saved {
                    if let Some(v) = old_var {
                        variables.insert(param_name, v);
                    } else {
                        variables.remove(&param_name);
                    }
                }
                // Write the return value to the destination.
                // 中文：将返回值写入目标位置。
                match dest {
                    Operand::Register(reg) => {
                        regs.write(*reg, ret_val);
                    }
                    Operand::Variable(name, _) => {
                        let var = variables
                            .get_mut(name)
                            .unwrap_or_else(|| panic!("variable not found: {name}"));
                        if !var.is_mut {
                            panic!("cannot assign to immutable variable: {name}");
                        }
                        var.value = ret_val;
                    }
                    Operand::Literal(_) => {
                        panic!("cannot assign call result to a literal");
                    }
                }
            }
            OpCode::Return(expr) => {
                // Evaluate the expression and store the result in
                // SystemVarBuffer for the caller to retrieve.
                // 中文：计算表达式并将结果存入 SystemVarBuffer，供调用者获取。
                let ret_val = expr.eval_expr(expr, regs, variables);
                regs.write(Register::SystemVarBuffer, ret_val);
            }
        }
    }
    Ok(())
}
