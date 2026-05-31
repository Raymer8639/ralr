//! Assembler entry point. Reads `.ralr` source files and writes a
//! bincode-serialized `.abin` binary.
//! 中文：汇编器入口点。读取 `.ralr` 源文件并写入 bincode 序列化的 `.abin` 二进制文件。

use ralr_asm::reader;

use anyhow::Result;
use clap::Parser;
use std::{
    fs::{self, File},
    io::BufWriter,
};

#[derive(Parser)]
struct Args {
    #[arg(required = true, num_args = 1..)]
    files: Vec<String>,
    /// Output file name (default: output.abin)
    /// 中文：输出文件名（默认值：output.abin）
    #[arg(short, long)]
    output_name: Option<String>,
}

fn main() -> Result<()> {
    let mut cmds = vec![];
    let args = Args::parse();

    // Parse each input file (full-file reads support multi-line blocks).
    // 中文：解析每个输入文件（全文件读取以支持多行块）。
    for file in &args.files {
        let content = fs::read_to_string(file)?;
        let mut parsed = reader::parse(&content)?;
        cmds.append(&mut parsed);
    }

    // Serialize the collected opcodes into the output binary.
    // 中文：将收集到的操作码序列化为输出二进制文件。
    let file = File::create(args.output_name.unwrap_or(String::from("output.abin")))?;
    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, &cmds)?;

    Ok(())
}
