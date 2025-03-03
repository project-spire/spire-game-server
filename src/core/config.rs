use serde::Deserialize;
use std::fs::File;
use std::io::{Bytes, Read};
use std::path::Path;

// config.json
static mut CHEAT_ENABLED: bool = false;
pub fn cheat_enabled() -> bool { unsafe { CHEAT_ENABLED } }

// Environment Variables
static mut AUTH_KEY: Vec<u8> = Vec::new();
pub fn auth_key() -> &'static [u8] { unsafe { AUTH_KEY.as_slice() } }


#[derive(Deserialize)]
struct ConfigJson {
    cheat_enabled: bool
}

pub fn init() {
    init_from_json();
    init_from_env();
}

fn init_from_json() {
    let path = Path::new(env!("OUT_DIR")).join("config.json");
    println!("Initializing config from {} ...", path.display());

    let json = serde_json::from_str::<ConfigJson>(&read_from_file(&path)).unwrap();
    
    unsafe {
        CHEAT_ENABLED = json.cheat_enabled;
    }
}

fn init_from_env() {
    unsafe {
        AUTH_KEY = read_from_file(Path::new(env!("SPIRE_AUTH_KEY_FILE"))).into_bytes();
    }
}

fn read_from_file(path: &Path) -> String {
    let mut f = File::open(path).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    buf
}