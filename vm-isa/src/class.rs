//! Class definitions and runtime object instances.
//! 中文：类定义和运行时对象实例。
//!
//! A [`ClassDef`] is the stored blueprint of a class: ordered field
//! initializers (evaluated at construction time) and a table of methods.
//! An [`Instance`] is a live object — a class name plus the current value
//! of each field.
//! 中文：[`ClassDef`] 是类的存储蓝图：有序的字段初始化器（在构造时求值）和方法表。
//! [`Instance`] 是活动对象 — 类名加上每个字段的当前值。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{expr::Expr, function::FnDef, value::Value};

/// Stored representation of a user-defined class.
/// 中文：用户自定义类的存储表示。
///
/// Fields keep declaration order so construction initializes them
/// deterministically; methods are keyed by name for O(1) dispatch. By
/// convention a method's first parameter is `self`, bound to the receiver.
/// 中文：字段保留声明顺序以便构造时确定性地初始化；方法按名称索引以实现 O(1) 分派。
/// 按约定，方法的第一个参数为 `self`，绑定到接收者。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub fields: Vec<(String, Expr)>,
    pub methods: BTreeMap<String, FnDef>,
}

/// A live object instance.
/// 中文：活动对象实例。
///
/// `class` records the originating class name (used to look up methods on
/// a call). `fields` stores the current value of every field; a
/// `BTreeMap` keeps iteration order stable for `Display` and tests.
/// 中文：`class` 记录来源类名（调用时用于查找方法）。`fields` 存储每个字段的当前值；
/// `BTreeMap` 使迭代顺序稳定，便于 `Display` 和测试。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Instance {
    pub class: String,
    pub fields: BTreeMap<String, Value>,
}
