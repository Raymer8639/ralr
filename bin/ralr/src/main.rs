use anyhow::Result;
use clap::Parser;
use tokio::fs;
use tracing::{Instrument, Level, info, span};
use vm_isa::{op_code::OpCode, register::Registers};

pub mod runner;

#[derive(Parser)]
#[command(version, about = "Raymer's vm")]
struct Args {
    file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let registers = Registers::new();
    // 读取和解析
    let reader_span = span!(Level::TRACE, "Reader");
    let op_code: Vec<OpCode> = async {
        info!("读取中...");
        let file = fs::read(args.file).await?;
        info!("解析中...");
        let op_code = bincode::deserialize::<Vec<OpCode>>(&file)?;
        Ok::<Vec<OpCode>, anyhow::Error>(op_code)
    }
    .instrument(reader_span)
    .await?;
    // 运行
    runner::runner(op_code, registers)?;
    Ok(())
}
