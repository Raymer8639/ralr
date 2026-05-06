use std::io::Write;
use std::sync::Arc;

use anyhow::Result;
use vm_isa::{
    op_code::OpCode,
    register::{AllRegister, Register},
    value::Value,
};
// 简化重复代码的宏 op_assgin(Value1, Value2, 目标寄存器, ar 寄存器, 符号, [目标寄存器的位置], [ ar 寄存器 要替换的位置])
// 通过输入的符号( $op ) 对 Value1 和 Value2 进行运算，再通过目标寄存器的位置 , 将运算的结果存入对应位置的 ar 寄存器中
macro_rules! op_assign {
    ($fir: expr, $sec: expr, $out: expr, $ar: expr, $op: tt, [$($register: ident),*], [$($ar_register: ident),*]) => {
        match $out {
            $(
                Register::$register(_) => $ar.$ar_register = Register::$register($fir $op $sec),
            )*
        }
    };
}
// 简化重复代码的宏 update_register_from_allregister(Value1, Value2, ar 寄存器, [目标要替换的寄存器], [要用来替换的ar寄存器])
// 用于当 Value1 和 Value2 的其中一个或者两个全部都是寄存器时，通过 ar 寄存器(及时同步过的寄存器) 同步从编译器得到的寄存器(值永远为Value::None)
macro_rules! update_register_from_allreister {
    ($fir: expr, $sec: expr, $ar: expr, [$($register: ident),*], [$($ar_register: ident),*]) => {
        {
            let fir = match $fir {
                Value::Register(value) => match *value {
                    $(
                        Register::$register(_) => Value::Register(Arc::new($ar.$ar_register.clone())),
                    )*
                }
                _ => $fir,
            };
            let sec = match $sec {
                Value::Register(value) => match *value {
                    $(
                        Register::$register(_) => Value::Register(Arc::new($ar.$ar_register.clone())),
                    )*
                }
                _ => $sec,
            };
            (fir, sec)
        }
    };
}

macro_rules! resolve_value {
    ($val: expr, $ar: expr, [$($register: ident),*], [$($ar_register: ident),*]) => {
        match $val {
            Value::Register(reg) => {
                let ar_reg = match *reg {
                    $(
                        Register::$register(_) => &$ar.$ar_register,
                    )*
                };
                match ar_reg {
                    $(
                        Register::$register(v) => v.clone(),
                    )*
                }
            }
            other => other,
        }
    };
}

pub fn runner(cmds: Vec<OpCode>, mut ar: AllRegister) -> Result<()> {
    for cmd in cmds {
        match cmd {
            OpCode::Add(fir, sec, out) => {
                let (fir, sec) = update_register_from_allreister!(
                    fir,
                    sec,
                    ar,
                    [A1, A2, A3, A4, A5],
                    [a1, a2, a3, a4, a5]
                );
                op_assign!(fir, sec, out, ar, +, [A1, A2, A3, A4, A5], [a1, a2, a3, a4, a5])
            }
            OpCode::Sub(fir, sec, out) => {
                let (fir, sec) = update_register_from_allreister!(
                    fir,
                    sec,
                    ar,
                    [A1, A2, A3, A4, A5],
                    [a1, a2, a3, a4, a5]
                );
                op_assign!(fir, sec, out, ar, -, [A1, A2, A3, A4, A5], [a1, a2, a3, a4, a5])
            }
            OpCode::Mul(fir, sec, out) => {
                let (fir, sec) = update_register_from_allreister!(
                    fir,
                    sec,
                    ar,
                    [A1, A2, A3, A4, A5],
                    [a1, a2, a3, a4, a5]
                );

                op_assign!(fir, sec, out, ar, *, [A1, A2, A3, A4, A5], [a1, a2, a3, a4, a5])
            }
            OpCode::Div(fir, sec, out) => {
                let (fir, sec) = update_register_from_allreister!(
                    fir,
                    sec,
                    ar,
                    [A1, A2, A3, A4, A5],
                    [a1, a2, a3, a4, a5]
                );
                op_assign!(fir, sec, out, ar, /, [A1, A2, A3, A4, A5], [a1, a2, a3, a4, a5])
            }
            OpCode::Println(val) => {
                let resolved = resolve_value!(val, ar, [A1, A2, A3, A4, A5], [a1, a2, a3, a4, a5]);
                println!("{resolved}");
            }
            OpCode::Print(val) => {
                let resolved = resolve_value!(val, ar, [A1, A2, A3, A4, A5], [a1, a2, a3, a4, a5]);
                print!("{resolved}");
                std::io::stdout().flush().unwrap();
            }
        }
    }
    Ok(())
}
