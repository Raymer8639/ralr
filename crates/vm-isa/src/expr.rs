//! Expression AST types for the ralr VM.
//! 中文：ralr 虚拟机的表达式 AST 类型。
//! 日本語：ralr VM の式 AST 型。
//! Русский: Типы AST выражений для ВМ ralr.

use serde::{Deserialize, Serialize};

use crate::{register::Register, value::Value};

/// Binary operator for [`Expr::Binary`] nodes.
/// 中文：[`Expr::Binary`] 节点的二元运算符。
/// 日本語：[`Expr::Binary`] ノードの二項演算子。
/// Русский: Бинарный оператор для узлов [`Expr::Binary`].
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
/// 日本語：[`Expr::Unary`] ノードの単項演算子。
/// Русский: Унарный оператор для узлов [`Expr::Unary`].
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
}

/// An expression tree.
/// 中文：表达式树。
/// 日本語：式ツリー。
/// Русский: Дерево выражений.
///
/// Leaf nodes are literals or register references. Inner nodes combine
/// sub-expressions with binary or unary operators.
/// 中文：叶节点是字面量或寄存器引用。内部节点用二元或一元运算符组合子表达式。
/// 日本語：葉ノードはリテラルまたはレジスタ参照です。内部ノードは部分式を二項または単項演算子で結合します。
/// Русский: Листовые узлы — литералы или ссылки на регистры. Внутренние узлы
/// комбинируют подвыражения бинарными или унарными операторами.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value),
    Register(Register),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
}
