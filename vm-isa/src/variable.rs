use serde::{Deserialize, Serialize};

use crate::value::Value;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Variable {
    pub is_mut: bool,
    pub value: Value,
}
