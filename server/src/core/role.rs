use std::fmt;

#[derive(Debug)]
pub enum Role {
    Player { account_id: u64, character_id: u64 },
    CheatPlayer { account_id: u64, character_id: u64 },
    Admin,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Player {account_id, character_id} => {
                write!(f, "Role(Player) account_id={}, character_id={}", account_id, character_id)
            },
            Role::CheatPlayer {account_id, character_id} => {
                write!(f, "Role(CheatPlayer) account_id={}, character_id={}", account_id, character_id)
            },
            Role::Admin => write!(f, "Role(Admin)"),
        }
    }
}
