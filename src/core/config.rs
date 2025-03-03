use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    enable_cheat: bool,
}

static mut CONFIG: Config = Config {
    enable_cheat: false,
};

impl Config {
    pub fn enable_cheat() -> bool {
        unsafe { CONFIG.enable_cheat }
    }
}

pub fn init() {
    let path = Path::new(env!("OUT_DIR")).join("config.json");
    println!("Initializing config from {} ...", path.display());

    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    unsafe {
        CONFIG = serde_json::from_str::<Config>(&contents).unwrap();
    }
}