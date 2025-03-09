use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Config {
    pub cheat_enabled: bool,

    pub game_listen_port: u16,
    pub admin_listen_port: u16,
    pub auth_key: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
pub fn config() -> &'static Config {
    &CONFIG.get().unwrap()
}

#[derive(Debug, Deserialize)]
struct ConfigJson {
    pub cheat_enabled: bool,
}

pub fn init() {
    let path = Path::new(env!("OUT_DIR")).join("config.json");
    println!("Initializing config from {}...", path.display());
    let mut json = serde_json::from_str::<ConfigJson>(&read_from_file(&path)).unwrap();

    println!("Initializing config from env variables...");
    let game_listen_port = u16::from_str(env!("SPIRE_GAME_LISTEN_PORT")).unwrap();
    let admin_listen_port = u16::from_str(env!("SPIRE_ADMIN_LISTEN_PORT")).unwrap();
    let auth_key = read_from_file(Path::new(env!("SPIRE_AUTH_KEY_FILE")));

    let config = Config {
        cheat_enabled: json.cheat_enabled,

        game_listen_port,
        admin_listen_port,
        auth_key,
    };

    CONFIG.set(config).unwrap();
    println!("Initializing config done!");
}

fn read_from_file(path: &Path) -> String {
    let mut f = File::open(path).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    buf
}
