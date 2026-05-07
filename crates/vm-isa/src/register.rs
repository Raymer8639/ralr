//! Register identifiers and the runtime register file.
//! 中文：寄存器标识符和运行时寄存器文件。
//! 日本語：レジスタ識別子とランタイムレジスタファイル。
//! Русский: Идентификаторы регистров и файл регистров времени выполнения.
//!
//! `Register` is a lightweight discriminant enum (no data, `Copy`).
//! 中文：`Register` 是轻量级的判别式枚举（无数据，`Copy`）。
//! 日本語：`Register` は軽量な判別式 enum です（データなし、`Copy`）。
//! Русский: `Register` — это легковесное перечисление-дискриминант (без данных, `Copy`).
//!
//! `Registers` holds the live values at runtime in a flat array for O(1) access.
//! 中文：`Registers` 在运行时将活动值保存在扁平数组中，以实现 O(1) 访问。
//! 日本語：`Registers` は実行時にフラット配列でライブ値を保持し、O(1) アクセスを実現します。
//! Русский: `Registers` хранит текущие значения во время выполнения в плоском массиве для доступа за O(1).

use crate::value::Value;
use serde::{Deserialize, Serialize};

/// One of five general-purpose registers (A1–A5).
/// 中文：五个通用寄存器之一（A1–A5）。
/// 日本語：5 つの汎用レジスタの 1 つ（A1–A5）。
/// Русский: Один из пяти регистров общего назначения (A1–A5).
///
/// Each variant maps to an array index via [`Register::index`].
/// 中文：每个变体通过 [`Register::index`] 映射到数组索引。
/// 日本語：各バリアントは [`Register::index`] を介して配列インデックスにマッピングされます。
/// Русский: Каждый вариант отображается на индекс массива через [`Register::index`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Register {
    A1,
    A2,
    A3,
    A4,
    A5,
}

impl Register {
    /// Maps the register discriminant to its position in the register file.
    /// 中文：将寄存器判别式映射到其在寄存器文件中的位置。
    /// 日本語：レジスタの判別式をレジスタファイル内の位置にマッピングします。
    /// Русский: Отображает дискриминант регистра на его позицию в файле регистров.
    pub fn index(self) -> usize {
        match self {
            Register::A1 => 0,
            Register::A2 => 1,
            Register::A3 => 2,
            Register::A4 => 3,
            Register::A5 => 4,
        }
    }
}

/// Runtime register file backed by a `[Value; 5]` array.
/// 中文：由 `[Value; 5]` 数组支持的运行时寄存器文件。
/// 日本語：`[Value; 5]` 配列でバックアップされたランタイムレジスタファイル。
/// Русский: Файл регистров времени выполнения на основе массива `[Value; 5]`.
///
/// Replaces the old `AllRegister` struct with named fields. Array indexing
/// gives constant-time lookup without matching on register variants.
/// 中文：替代了旧的带命名字段的 `AllRegister` 结构体。数组索引提供常量时间查找，无需匹配寄存器变体。
/// 日本語：名前付きフィールドを持つ古い `AllRegister` 構造体を置き換えます。配列インデックスにより、レジスタバリアントでのマッチングなしで定数時間の検索が可能です。
/// Русский: Заменяет старую структуру `AllRegister` с именованными полями. Индексация массива
/// обеспечивает поиск за константное время без сопоставления вариантов регистров.
#[derive(Debug)]
pub struct Registers {
    inner: [Value; 5],
}

impl Registers {
    /// Creates a new register file with all registers set to [`Value::None`].
    /// 中文：创建一个新的寄存器文件，所有寄存器均设置为 [`Value::None`]。
    /// 日本語：すべてのレジスタが [`Value::None`] に設定された新しいレジスタファイルを作成します。
    /// Русский: Создаёт новый файл регистров, все регистры установлены в [`Value::None`].
    pub fn new() -> Self {
        Self {
            inner: [
                Value::None,
                Value::None,
                Value::None,
                Value::None,
                Value::None,
            ],
        }
    }

    /// Returns a shared reference to the value stored in a register.
    /// 中文：返回寄存器中存储的值的共享引用。
    /// 日本語：レジスタに格納されている値への共有参照を返します。
    /// Русский: Возвращает общую ссылку на значение, хранящееся в регистре.
    pub fn read(&self, reg: Register) -> &Value {
        &self.inner[reg.index()]
    }

    /// Writes a value into a register.
    /// 中文：将值写入寄存器。
    /// 日本語：レジスタに値を書き込みます。
    /// Русский: Записывает значение в регистр.
    pub fn write(&mut self, reg: Register, value: Value) {
        self.inner[reg.index()] = value;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}
