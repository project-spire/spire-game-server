use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;

    // Copy `config.json`
    println!("cargo:rerun-if-changed=config.json");
    fs::copy("config.json", Path::new(&out_dir).join("config.json"))?;

    // Compile protocols
    println!("cargo:rerun-if-changed=protocol/");
    let proto_files: Vec<PathBuf> = glob("protocol/**/*.proto")?
        .filter_map(Result::ok)
        .collect();
    prost_build::compile_protos(&proto_files, &["protocol/"])?;

    Ok(())
}
