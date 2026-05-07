//! Tests for [`Register`] discriminant indices and [`Registers`] read/write.
//! 中文：[`Register`] 判别式索引和 [`Registers`] 读写测试。
//! 日本語：[`Register`] 判別式インデックスと [`Registers`] 読み書きのテスト。
//! Русский: Тесты дискриминантных индексов [`Register`] и чтения/записи [`Registers`].

use vm_isa::register::{Register, Registers};
use vm_isa::value::Value;

#[test]
fn registers_new_initializes_all_to_none() {
    let regs = Registers::new();
    assert!(matches!(regs.read(Register::A1), Value::None));
    assert!(matches!(regs.read(Register::A2), Value::None));
    assert!(matches!(regs.read(Register::A3), Value::None));
    assert!(matches!(regs.read(Register::A4), Value::None));
    assert!(matches!(regs.read(Register::A5), Value::None));
}

#[test]
fn registers_read_write_roundtrip() {
    let mut regs = Registers::new();
    regs.write(Register::A1, Value::I32(42));
    assert!(matches!(regs.read(Register::A1), Value::I32(42)));
    // Other registers unchanged
    // 中文：其他寄存器保持不变
    // 日本語：他のレジスタは変更なし
    // Русский: Остальные регистры не изменились
    assert!(matches!(regs.read(Register::A2), Value::None));
}

/// Register indices are the contract between the enum and the array layout.
/// 中文：寄存器索引是枚举与数组布局之间的契约。
/// 日本語：レジスタインデックスは列挙型と配列レイアウトの間の契約です。
/// Русский: Индексы регистров — это контракт между перечислением и макетом массива.
#[test]
fn register_index_is_stable() {
    assert_eq!(Register::A1.index(), 0);
    assert_eq!(Register::A2.index(), 1);
    assert_eq!(Register::A3.index(), 2);
    assert_eq!(Register::A4.index(), 3);
    assert_eq!(Register::A5.index(), 4);
}

#[test]
fn registers_default_equals_new() {
    let new_regs = Registers::new();
    let default_regs = Registers::default();
    for reg in [
        Register::A1,
        Register::A2,
        Register::A3,
        Register::A4,
        Register::A5,
    ] {
        assert!(matches!(
            (new_regs.read(reg), default_regs.read(reg)),
            (Value::None, Value::None)
        ));
    }
}
