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
    class::{ClassDef, Instance},
    function::FnDef,
    op_code::{Expr, IoOp, OpCode, Operand},
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
/// Writes `val` to a destination operand (register or mutable variable).
/// 中文：将 `val` 写入目标操作数（寄存器或可变变量）。
///
/// Panics if the destination is an immutable variable or a literal.
/// 中文：若目标是不可变变量或字面量则 panic。
fn write_dest(
    dest: &Operand,
    val: Value,
    regs: &mut Registers,
    variables: &mut AHashMap<String, Variable>,
) {
    match dest {
        Operand::Register(reg) => regs.write(*reg, val),
        Operand::Variable(name, _) => {
            let var = variables
                .get_mut(name)
                .unwrap_or_else(|| panic!("variable not found: {name}"));
            if !var.is_mut {
                panic!("cannot assign to immutable variable: {name}");
            }
            var.value = val;
        }
        Operand::Literal(_) => panic!("cannot assign to a literal"),
    }
}

/// Invokes a method body with `receiver` bound to its first parameter
/// (`self`) and `args` bound to the remaining parameters.
/// 中文：调用方法体，将 `receiver` 绑定到第一个参数（`self`），`args` 绑定到其余参数。
///
/// Returns the (possibly mutated) receiver instance so the caller can
/// persist field changes. The method's return value is left in
/// `SystemVarBuffer` for the caller to read.
/// 中文：返回（可能已修改的）接收者实例，以便调用者持久化字段更改。
/// 方法的返回值留在 `SystemVarBuffer` 中供调用者读取。
fn call_method(
    method: &FnDef,
    receiver: Instance,
    args: &[Expr],
    regs: &mut Registers,
    variables: &mut AHashMap<String, Variable>,
    functions: &mut AHashMap<String, FnDef>,
    classes: &mut AHashMap<String, ClassDef>,
) -> Result<Instance> {
    let self_param = method
        .params
        .first()
        .unwrap_or_else(|| panic!("method must take 'self' as its first parameter"));
    let rest_params = &method.params[1..];
    if rest_params.len() != args.len() {
        panic!(
            "method expects {} argument(s), got {}",
            rest_params.len(),
            args.len()
        );
    }
    // Evaluate arguments in the caller scope before binding parameters.
    // 中文：在绑定参数之前，于调用者作用域中计算参数。
    let arg_vals: Vec<Value> = args
        .iter()
        .map(|a| a.eval_expr(a, regs, variables))
        .collect();
    // Save variables shadowed by `self` and the parameters.
    // 中文：保存被 `self` 和参数遮蔽的变量。
    let mut saved: Vec<(String, Option<Variable>)> =
        vec![(self_param.clone(), variables.remove(self_param))];
    for p in rest_params {
        saved.push((p.clone(), variables.remove(p)));
    }
    // Bind `self` (mutable so the body may update fields) and parameters.
    // 中文：绑定 `self`（可变，使方法体可更新字段）和参数。
    variables.insert(
        self_param.clone(),
        Variable {
            is_mut: true,
            value: Value::Object(Box::new(receiver)),
        },
    );
    for (p, v) in rest_params.iter().zip(arg_vals) {
        variables.insert(
            p.clone(),
            Variable {
                is_mut: false,
                value: v,
            },
        );
    }
    // Execute the method body, then reclaim the (mutated) receiver.
    // 中文：执行方法体，然后回收（已修改的）接收者。
    runner(&method.body, regs, variables, functions, classes)?;
    let self_var = variables
        .remove(self_param)
        .expect("'self' binding disappeared during method execution");
    let new_receiver = match self_var.value {
        Value::Object(inst) => *inst,
        other => panic!("'self' was reassigned to a non-object: {other:?}"),
    };
    // Restore shadowed variables.
    // 中文：恢复被遮蔽的变量。
    for (name, old) in saved {
        match old {
            Some(v) => {
                variables.insert(name, v);
            }
            None => {
                variables.remove(&name);
            }
        }
    }
    Ok(new_receiver)
}

/// Executes a sequence of opcodes against a register file, variable
/// hashmap, function table, and class table.
/// 中文：根据寄存器文件、变量哈希表、函数表和类表执行一系列操作码。
pub fn runner(
    cmds: &[OpCode],
    regs: &mut Registers,
    variables: &mut AHashMap<String, Variable>,
    functions: &mut AHashMap<String, FnDef>,
    classes: &mut AHashMap<String, ClassDef>,
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
                runner(inner, regs, variables, functions, classes)?;
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
                    Value::Bool(true) => runner(then_body, regs, variables, functions, classes)?,
                    Value::Bool(false) => {
                        if let Some(else_b) = else_body {
                            runner(else_b, regs, variables, functions, classes)?;
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
                        runner(cmds, regs, variables, functions, classes)?;
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
                runner(&fn_def.body, regs, variables, functions, classes)?;
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
                write_dest(dest, ret_val, regs, variables);
            }
            OpCode::Return(expr) => {
                // Evaluate the expression and store the result in
                // SystemVarBuffer for the caller to retrieve.
                // 中文：计算表达式并将结果存入 SystemVarBuffer，供调用者获取。
                let ret_val = expr.eval_expr(expr, regs, variables);
                regs.write(Register::SystemVarBuffer, ret_val);
            }
            OpCode::ClassDef { name, def } => {
                // Register the class definition in the class table.
                // 中文：在类表中注册类定义。
                classes.insert(name.clone(), def.clone());
            }
            OpCode::New { class, args, dest } => {
                // Look up the class and initialize its fields in order.
                // 中文：查找类并按顺序初始化其字段。
                let cls = classes
                    .get(class)
                    .cloned()
                    .unwrap_or_else(|| panic!("class not found: {class}"));
                let mut fields = std::collections::BTreeMap::new();
                for (field_name, init) in &cls.fields {
                    let v = init.eval_expr(init, regs, variables);
                    fields.insert(field_name.clone(), v);
                }
                let mut instance = Instance {
                    class: class.clone(),
                    fields,
                };
                // If an `init` method exists, run it as a constructor.
                // 中文：如果存在 `init` 方法，则将其作为构造函数运行。
                if let Some(init_fn) = cls.methods.get("init") {
                    instance =
                        call_method(init_fn, instance, args, regs, variables, functions, classes)?;
                    // Discard any return value left by the constructor.
                    // 中文：丢弃构造函数留下的任何返回值。
                    regs.take(Register::SystemVarBuffer);
                } else if !args.is_empty() {
                    panic!(
                        "class '{class}' has no 'init' method but was constructed with {} argument(s)",
                        args.len()
                    );
                }
                write_dest(dest, Value::Object(Box::new(instance)), regs, variables);
            }
            OpCode::MethodCall {
                recv,
                method,
                args,
                dest,
            } => {
                // Take the receiver object out of its variable.
                // 中文：从其变量中取出接收者对象。
                let recv_var = variables
                    .get(recv)
                    .unwrap_or_else(|| panic!("variable not found: {recv}"));
                let instance = match &recv_var.value {
                    Value::Object(inst) => (**inst).clone(),
                    other => panic!("'{recv}' is not an object: {other:?}"),
                };
                let cls = classes
                    .get(&instance.class)
                    .cloned()
                    .unwrap_or_else(|| panic!("class not found: {}", instance.class));
                let m = cls.methods.get(method).cloned().unwrap_or_else(|| {
                    panic!("method '{method}' not found on class '{}'", instance.class)
                });
                let new_instance =
                    call_method(&m, instance, args, regs, variables, functions, classes)?;
                // Persist field mutations back into the receiver variable
                // (objects have reference-like mutation semantics).
                // 中文：将字段修改持久化回接收者变量（对象具有类似引用的修改语义）。
                if let Some(v) = variables.get_mut(recv) {
                    v.value = Value::Object(Box::new(new_instance));
                }
                // Capture the method's return value and write it to dest.
                // 中文：捕获方法的返回值并写入目标位置。
                let ret_val = regs.take(Register::SystemVarBuffer);
                write_dest(dest, ret_val, regs, variables);
            }
            OpCode::SetField { base, path, value } => {
                // Evaluate the new value before borrowing the object mutably.
                // 中文：在可变借用对象之前计算新值。
                let new_val = value.eval_expr(value, regs, variables);
                let var = variables
                    .get_mut(base)
                    .unwrap_or_else(|| panic!("variable not found: {base}"));
                // Navigate intermediate fields, then set the final one.
                // 中文：导航中间字段，然后设置最终字段。
                let mut target = &mut var.value;
                for seg in &path[..path.len() - 1] {
                    target = match target {
                        Value::Object(inst) => inst
                            .fields
                            .get_mut(seg)
                            .unwrap_or_else(|| panic!("field not found: {seg}")),
                        other => panic!("cannot access field '{seg}' on non-object: {other:?}"),
                    };
                }
                let last = path.last().expect("field path must be non-empty");
                match target {
                    Value::Object(inst) => {
                        inst.fields.insert(last.clone(), new_val);
                    }
                    other => panic!("cannot set field '{last}' on non-object: {other:?}"),
                }
            }
        }
    }
    Ok(())
}
