use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

// config.json
static CHEAT_ENABLED: OnceLock<bool> = OnceLock::new();
pub fn cheat_enabled() -> bool { *CHEAT_ENABLED.get().unwrap() }

// Environment Variables
static AUTH_KEY: OnceLock<Vec<u8>> = OnceLock::new();
pub fn auth_key() -> &'static [u8] { (*AUTH_KEY.get().unwrap()).as_slice() }


pub fn init() {
    init_from_json();
    init_from_env();
}

fn init_from_json() {
    #[derive(Deserialize)]
    struct ConfigJson {
        cheat_enabled: bool
    }

    let path = Path::new(env!("OUT_DIR")).join("config.json");
    println!("Initializing config from {} ...", path.display());

    let json = serde_json::from_str::<ConfigJson>(&read_from_file(&path)).unwrap();
    CHEAT_ENABLED.set(json.cheat_enabled).unwrap();
}

fn init_from_env() {
    AUTH_KEY.set(read_from_file(Path::new(env!("SPIRE_AUTH_KEY_FILE"))).into_bytes()).unwrap();
}

fn read_from_file(path: &Path) -> String {
    let mut f = File::open(path).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    buf
}