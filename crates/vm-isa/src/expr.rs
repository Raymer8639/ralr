//! Expression AST types for the ralr VM.
//! 中文：ralr 虚拟机的表达式 AST 类型。

use crate::{op_code::Operand, register::Registers, value::Value, variable::Variable};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

/// Binary operator for [`Expr::Binary`] nodes.
/// 中文：[`Expr::Binary`] 节点的二元运算符。
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary operator for [`Expr::Unary`] nodes.
/// 中文：[`Expr::Unary`] 节点的一元运算符。
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
}

/// An expression tree.
/// 中文：表达式树。
///
/// Leaf nodes are literals or register references. Inner nodes combine
/// sub-expressions with binary or unary operators.
/// 中文：叶节点是字面量或寄存器引用。内部节点用二元或一元运算符组合子表达式。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Expr {
    Operand(Operand),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
}
impl Expr {
    /// Recursively evaluates an expression tree against the register file.
    /// 中文：根据寄存器文件递归计算表达式树。
    ///
    /// Register references are resolved from the current register state. All
    /// arithmetic, comparison, logical, bitwise, and shift operators are
    /// dispatched here.
    /// 中文：寄存器引用从当前寄存器状态解析。所有算术、比较、逻辑、位和移位运算符都在此处分派。
    pub fn eval_expr(
        &self,
        expr: &Expr,
        regs: &Registers,
        variables: &AHashMap<String, Variable>,
    ) -> Value {
        match expr {
            Expr::Operand(Operand::Literal(value)) => (*value).clone(),
            Expr::Operand(Operand::Register(register)) => (*regs.read(*register)).clone(),
            Expr::Operand(Operand::Variable(name, _)) => {
                let value = variables
                    .get(name)
                    .expect("Error: Cannot find the variable");
                value.value.clone()
            }
            Expr::Binary(lhs, op, rhs) => {
                let lhs_val = self.eval_expr(lhs, regs, variables);
                let rhs_val = self.eval_expr(rhs, regs, variables);
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
                let val = self.eval_expr(inner, regs, variables);
                match op {
                    UnOp::Neg => -val,
                    UnOp::Not => val.logical_not(),
                    UnOp::BitNot => val.bitwise_not(),
                }
            }
        }
    }
}
