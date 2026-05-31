//! Shared instruction set architecture for the ralr register-based VM.
//! 中文：ralr 基于寄存器的虚拟机的共享指令集架构。
//!
//! Defines the core types used by both the assembler (`ralr-asm`) and the
//! runtime (`ralr`): instructions, operands, values, and registers.
//! 中文：定义汇编器（`ralr-asm`）和运行时（`ralr`）共同使用的核心类型：指令、操作数、值和寄存器。

pub mod expr;
pub mod function;
pub mod op_code;
pub mod register;
pub mod value;
pub mod variable;
