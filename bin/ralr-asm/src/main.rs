use anyhow::Result;
use clap::Parser;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter},
};

mod reader;

#[derive(Parser)]
struct Args {
    #[arg(required = true, num_args = 1..)]
    files: Vec<String>,
    #[arg(short, long)]
    output_name: Option<String>,
}

fn main() -> Result<()> {
    let mut cmds = vec![];
    let args = Args::parse();

    // 读取文件
    for file in args.files {
        let file = File::open(file)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            reader::reader(line?, &mut cmds)?;
        }
    }

    // 写入文件
    let file = File::create(args.output_name.unwrap_or(String::from("output.abin")))?;
    let reader = BufWriter::new(file);
    bincode::serialize_into(reader, &cmds)?;

    Ok(())
}
