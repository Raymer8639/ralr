use serde::{Deserialize, Serialize};

use crate::op_code::OpCode;

/// Stored representation of a user-defined function.
/// 中文：用户自定义函数的存储表示。
///
/// Parameter names and body opcodes are registered at definition time
/// and replayed on each call.
/// 中文：参数名和体操作码在定义时注册，在每次调用时重放。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FnDef {
    pub params: Vec<String>,
    pub body: Vec<OpCode>,
}
