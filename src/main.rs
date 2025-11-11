use crate::execution::{runtime::Runtime, wasi::WasiSnapshotPreview1};
use clap::Parser;
use std::{fs::File, io::Read};

mod binary;
mod execution;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    wasm_file_path: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let wasm_file_path = match args.wasm_file_path {
        Some(path) => path,
        None => "./src/fixtures/hello_world.wasm".to_string(),
    };

    let mut wasm_file = File::open(&wasm_file_path)?;
    let mut wasm = Vec::new();
    let _ = wasm_file.read_to_end(&mut wasm)?;

    let wasi = WasiSnapshotPreview1::new();
    let mut runtime = Runtime::instantiate_with_wasi(wasm, wasi)?;
    println!("{:?}", runtime.store);
    println!("{:?}", runtime.wasi);

    runtime.call("_start", vec![])?;
    Ok(())
}
