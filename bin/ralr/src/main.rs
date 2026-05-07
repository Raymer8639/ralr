//! VM entry point. Reads a `.abin` file, deserializes instructions,
//! and hands them off to the synchronous runner.
//! 中文：虚拟机入口点。读取 `.abin` 文件，反序列化指令，并将其传递给同步执行器。
//! 日本語：VM エントリポイント。`.abin` ファイルを読み取り、命令をデシリアライズし、同期ランナーに渡します。
//! Русский: Точка входа ВМ. Читает файл `.abin`, десериализует инструкции и
//! передаёт их синхронному исполнителю.

use anyhow::Result;
use clap::Parser;
use std::fs;
use tracing::{Level, info, span};
use vm_isa::{op_code::OpCode, register::Registers};

use ralr::runner;

#[derive(Parser)]
#[command(version, about = "Raymer's vm")]
struct Args {
    file: String,
}

fn main() -> Result<()> {
    // Initialize structured logging (filter via RUST_LOG env var).
    // 中文：初始化结构化日志（通过 RUST_LOG 环境变量过滤）。
    // 日本語：構造化ロギングを初期化します（RUST_LOG 環境変数でフィルタ）。
    // Русский: Инициализировать структурированное логирование (фильтр через переменную окружения RUST_LOG).
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let mut registers = Registers::new();

    // Read the binary file and deserialize into opcodes.
    // 中文：读取二进制文件并反序列化为操作码。
    // 日本語：バイナリファイルを読み取り、オペコードにデシリアライズします。
    // Русский: Прочитать бинарный файл и десериализовать в опкоды.
    let reader_span = span!(Level::TRACE, "Reader");
    let op_code: Vec<OpCode> = {
        let _enter = reader_span.enter();
        info!("Reading...");
        let file = fs::read(args.file)?;
        info!("Deserializing...");
        let op_code = bincode::deserialize::<Vec<OpCode>>(&file)?;
        Ok::<Vec<OpCode>, anyhow::Error>(op_code)
    }?;
    // Execute instructions against a fresh register file.
    // 中文：在全新的寄存器文件上执行指令。
    // 日本語：新しいレジスタファイルに対して命令を実行します。
    // Русский: Выполнить инструкции на свежем файле регистров.
    runner::runner(&op_code, &mut registers)?;
    Ok(())
}
