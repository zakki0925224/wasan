use crate::execution::{runtime::Runtime, wasi::WasiSnapshotPreview1};

mod binary;
mod execution;

fn main() -> anyhow::Result<()> {
    let wasi = WasiSnapshotPreview1::new();
    let wasm = include_bytes!("./fixtures/hello_world.wasm");
    let mut runtime = Runtime::instantiate_with_wasi(wasm, wasi)?;
    runtime.call("_start", vec![])?;
    Ok(())
}
