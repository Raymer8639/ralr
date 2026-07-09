//! Tests for object-oriented syntax: classes, fields, methods, `new`.
//! 中文：面向对象语法测试：类、字段、方法、`new`。
//! 日本語：オブジェクト指向構文のテスト：クラス、フィールド、メソッド、`new`。
//! Русский: Тесты объектно-ориентированного синтаксиса: классы, поля, методы, `new`.

use std::collections::BTreeMap;

use ralr_asm::reader;
use vm_isa::class::Instance;
use vm_isa::op_code::{Expr, OpCode, Operand};
use vm_isa::register::{Register, Registers};
use vm_isa::value::Value;

// ── Helpers ──────────────────────────────────────────────────────────
// 中文：辅助函数

/// Parse + execute, returning both the register file and the variable map.
/// 中文：解析 + 执行，返回寄存器文件和变量哈希表。
fn run(
    source: &str,
) -> (
    Registers,
    ahash::AHashMap<String, vm_isa::variable::Variable>,
) {
    let ops = reader::parse(source).unwrap();
    let mut regs = Registers::new();
    let mut variables: ahash::AHashMap<String, vm_isa::variable::Variable> = ahash::AHashMap::new();
    let mut functions: ahash::AHashMap<String, vm_isa::function::FnDef> = ahash::AHashMap::new();
    let mut classes: ahash::AHashMap<String, vm_isa::class::ClassDef> = ahash::AHashMap::new();
    ralr::runner::runner(
        &ops,
        &mut regs,
        &mut variables,
        &mut functions,
        &mut classes,
    )
    .unwrap();
    (regs, variables)
}

/// Reads a field of an object stored in a named variable.
/// 中文：读取存储在命名变量中的对象的字段。
fn field(vars: &ahash::AHashMap<String, vm_isa::variable::Variable>, var: &str, f: &str) -> Value {
    match &vars.get(var).expect("variable not found").value {
        Value::Object(inst) => inst.fields.get(f).expect("field not found").clone(),
        other => panic!("not an object: {other:?}"),
    }
}

// ── Parsing tests ────────────────────────────────────────────────────
// 中文：解析测试

#[test]
fn parse_class_fields_and_methods() {
    let ops = reader::parse(
        "class Point { let x = 0; let y = 0; fn sum(self) { return self.x + self.y; } }",
    )
    .unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::ClassDef { name, def } => {
            assert_eq!(name, "Point");
            assert_eq!(def.fields.len(), 2);
            assert_eq!(def.fields[0].0, "x");
            assert_eq!(def.fields[1].0, "y");
            assert!(def.methods.contains_key("sum"));
            assert_eq!(def.methods["sum"].params, vec!["self".to_string()]);
        }
        other => panic!("expected ClassDef, got {other:?}"),
    }
}

#[test]
fn parse_new_assignment() {
    let ops = reader::parse("$a1 = new Point(3, 4);").unwrap();
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        OpCode::New { class, args, dest } => {
            assert_eq!(class, "Point");
            assert_eq!(args.len(), 2);
            assert_eq!(*dest, Operand::Register(Register::A1));
        }
        other => panic!("expected New, got {other:?}"),
    }
}

#[test]
fn parse_method_call_assignment() {
    let ops = reader::parse("$a1 = p.sum();").unwrap();
    match &ops[0] {
        OpCode::MethodCall {
            recv,
            method,
            args,
            dest,
        } => {
            assert_eq!(recv, "p");
            assert_eq!(method, "sum");
            assert!(args.is_empty());
            assert_eq!(*dest, Operand::Register(Register::A1));
        }
        other => panic!("expected MethodCall, got {other:?}"),
    }
}

#[test]
fn parse_bare_method_call() {
    let ops = reader::parse("p.shift(1, 2);").unwrap();
    match &ops[0] {
        OpCode::MethodCall {
            recv,
            method,
            args,
            dest,
        } => {
            assert_eq!(recv, "p");
            assert_eq!(method, "shift");
            assert_eq!(args.len(), 2);
            assert_eq!(*dest, Operand::Register(Register::SystemVarBuffer));
        }
        other => panic!("expected MethodCall, got {other:?}"),
    }
}

#[test]
fn parse_field_assignment() {
    let ops = reader::parse("p.x = 42;").unwrap();
    match &ops[0] {
        OpCode::SetField { base, path, value } => {
            assert_eq!(base, "p");
            assert_eq!(path, &vec!["x".to_string()]);
            assert_eq!(*value, Expr::Operand(Operand::Literal(Value::U8(42))));
        }
        other => panic!("expected SetField, got {other:?}"),
    }
}

#[test]
fn parse_field_access_in_expression() {
    let ops = reader::parse("$a1 = p.x + p.y;").unwrap();
    match &ops[0] {
        OpCode::Expr(Expr::Binary(lhs, _, rhs), _) => {
            assert!(matches!(**lhs, Expr::Field(_, ref f) if f == "x"));
            assert!(matches!(**rhs, Expr::Field(_, ref f) if f == "y"));
        }
        other => panic!("expected Expr(Binary), got {other:?}"),
    }
}

#[test]
fn float_literal_is_not_field_access() {
    // `2.5` must parse as a float literal, not a field access on `2`.
    // 中文：`2.5` 必须解析为浮点字面量，而非对 `2` 的字段访问。
    let ops = reader::parse("$a1 = 2.5;").unwrap();
    assert_eq!(
        ops[0],
        OpCode::Expr(
            Expr::Operand(Operand::Literal(Value::F32(2.5))),
            Operand::Register(Register::A1),
        )
    );
}

// ── Serialization tests ──────────────────────────────────────────────
// 中文：序列化测试

#[test]
fn classdef_bincode_roundtrip() {
    let ops = reader::parse("class C { let v = 1; fn get(self) { return self.v; } }").unwrap();
    let bytes = bincode::serialize(&ops).unwrap();
    let back: Vec<OpCode> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(ops, back);
}

#[test]
fn object_value_bincode_roundtrip() {
    let mut fields = BTreeMap::new();
    fields.insert("x".to_string(), Value::U8(3));
    fields.insert("y".to_string(), Value::U8(4));
    let v = Value::Object(Box::new(Instance {
        class: "Point".to_string(),
        fields,
    }));
    let bytes = bincode::serialize(&v).unwrap();
    let back: Value = bincode::deserialize(&bytes).unwrap();
    assert_eq!(v, back);
}

#[test]
fn object_display() {
    let mut fields = BTreeMap::new();
    fields.insert("x".to_string(), Value::U8(1));
    fields.insert("y".to_string(), Value::U8(2));
    let v = Value::Object(Box::new(Instance {
        class: "Point".to_string(),
        fields,
    }));
    assert_eq!(format!("{v}"), "Point { x: 1, y: 2 }");
}

#[test]
fn empty_object_display() {
    let v = Value::Object(Box::new(Instance {
        class: "Empty".to_string(),
        fields: BTreeMap::new(),
    }));
    assert_eq!(format!("{v}"), "Empty {}");
}

// ── Integration tests ────────────────────────────────────────────────
// 中文：集成测试

#[test]
fn construct_and_read_fields() {
    let (_, vars) = run(
        "class Point { let x = 0; let y = 0; fn init(self, x, y) { self.x = x; self.y = y; } } \
         let p = new Point(3, 4);",
    );
    assert_eq!(field(&vars, "p", "x"), Value::U8(3));
    assert_eq!(field(&vars, "p", "y"), Value::U8(4));
}

#[test]
fn method_returns_value() {
    let (regs, _) = run(
        "class Point { let x = 0; let y = 0; fn init(self, x, y) { self.x = x; self.y = y; } \
         fn sum(self) { return self.x + self.y; } } \
         let p = new Point(3, 4); $a1 = p.sum();",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(7));
}

#[test]
fn method_mutation_persists() {
    let (regs, vars) = run(
        "class Point { let x = 0; let y = 0; fn init(self, x, y) { self.x = x; self.y = y; } \
         fn shift(self, dx, dy) { self.x = self.x + dx; self.y = self.y + dy; } \
         fn sum(self) { return self.x + self.y; } } \
         let mut p = new Point(3, 4); p.shift(10, 20); $a1 = p.sum();",
    );
    assert_eq!(field(&vars, "p", "x"), Value::U8(13));
    assert_eq!(field(&vars, "p", "y"), Value::U8(24));
    assert_eq!(regs.read(Register::A1), &Value::U8(37));
}

#[test]
fn direct_field_assignment() {
    let (_, vars) = run("class Point { let x = 0; let y = 0; } let mut p = new Point(); p.x = 99;");
    assert_eq!(field(&vars, "p", "x"), Value::U8(99));
    assert_eq!(field(&vars, "p", "y"), Value::U8(0));
}

#[test]
fn class_without_init_uses_defaults() {
    let (regs, _) = run(
        "class Counter { let n = 5; fn get(self) { return self.n; } } \
         let c = new Counter(); $a1 = c.get();",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(5));
}

#[test]
fn method_mutates_then_reads_via_field_access() {
    let (regs, _) = run(
        "class Counter { let n = 0; fn bump(self) { self.n = self.n + 1; } } \
         let mut c = new Counter(); c.bump(); c.bump(); c.bump(); $a1 = c.n;",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(3));
}

#[test]
fn field_used_in_condition() {
    let (regs, _) = run("class Box { let v = 0; fn init(self, v) { self.v = v; } } \
         let b = new Box(10); if b.v > 5 { $a1 = 1; } else { $a1 = 0; }");
    assert_eq!(regs.read(Register::A1), &Value::U8(1));
}

#[test]
fn method_returning_new_object() {
    // A factory method that constructs and returns an object.
    // 中文：构造并返回对象的工厂方法。
    let (regs, vars) = run(
        "class Wrapper { let inner = 0; fn init(self, v) { self.inner = v; } } \
         fn make(v) { return new Wrapper(v); } \
         let w = call make(7); $a1 = w.inner;",
    );
    assert_eq!(regs.read(Register::A1), &Value::U8(7));
    assert_eq!(field(&vars, "w", "inner"), Value::U8(7));
}

