//! Shared instruction set architecture for the ralr register-based VM.
//! 中文：ralr 基于寄存器的虚拟机的共享指令集架构。
//! 日本語：ralr レジスタベース VM の共有命令セットアーキテクチャ。
//! Русский: Общая архитектура набора инструкций для регистровой виртуальной машины ralr.
//!
//! Defines the core types used by both the assembler (`ralr-asm`) and the
//! runtime (`ralr`): instructions, operands, values, and registers.
//! 中文：定义汇编器（`ralr-asm`）和运行时（`ralr`）共同使用的核心类型：指令、操作数、值和寄存器。
//! 日本語：アセンブラ（`ralr-asm`）とランタイム（`ralr`）の両方で使用されるコア型（命令、オペランド、値、レジスタ）を定義します。
//! Русский: Определяет основные типы, используемые как ассемблером (`ralr-asm`), так и
//! средой выполнения (`ralr`): инструкции, операнды, значения и регистры.

pub mod expr;
pub mod op_code;
pub mod register;
pub mod value;
