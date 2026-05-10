//! Expression AST types for the ralr VM.
//! 中文：ralr 虚拟机的表达式 AST 类型。

use serde::{Deserialize, Serialize};

use crate::{register::Register, value::Value};

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
    Literal(Value),
    Register(Register),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
}
