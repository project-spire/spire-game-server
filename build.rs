use std::path::PathBuf;
use glob::glob;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files: Vec<PathBuf> = glob("protocol/msg/**/*.proto")?
        .filter_map(Result::ok)
        .collect();

    prost_build::compile_protos(&proto_files, &["protocol/msg"])?;
    Ok(())
}