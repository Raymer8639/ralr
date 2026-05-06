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
    assert!(matches!(regs.read(Register::A2), Value::None));
}

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
